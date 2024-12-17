use gloo_net::websocket::{events::CloseEvent, Message, State, WebSocketError};
use futures_channel::mpsc;
use futures_core::{ready, Stream};
use futures_sink::Sink;
use gloo_utils::errors::JsError;
use pin_project::{pin_project, pinned_drop};
use std::cell::RefCell;
use std::pin::Pin;
use std::sync::{Arc, MutexGuard};
use std::sync::Mutex;
use std::task::{Context, Poll, Waker};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{BinaryType, MessageEvent};
use std::io;
use core::cmp;
use futures_io::{AsyncRead, AsyncWrite};

fn js_to_js_error(js_value: JsValue) -> JsError {
    match JsError::try_from(js_value) {
        Ok(error) => error,
        Err(_) => unreachable!("JsValue passed is not an Error type -- this is a bug"),
    }
}

/// Wrapper around browser's WebSocket API.
#[allow(missing_debug_implementations)]
#[pin_project(PinnedDrop)]
pub struct WebSocket {
    ws: web_sys::WebSocket,
    sink_waker: Arc<RefCell<Option<Waker>>>,
    #[pin]
    message_receiver: mpsc::UnboundedReceiver<StreamMessage>,
    #[allow(clippy::type_complexity)]
    closures: (
        Closure<dyn FnMut()>,
        Closure<dyn FnMut(MessageEvent)>,
        Closure<dyn FnMut(web_sys::Event)>,
        Closure<dyn FnMut(web_sys::CloseEvent)>,
    ),
    /// Leftover bytes when using `AsyncRead`.
    ///
    /// These bytes are drained and returned in subsequent calls to `poll_read`.
    pub(super) read_pending_bytes: Arc<Mutex<Option<Vec<u8>>>>, // Same size as `Vec<u8>` alone thanks to niche optimization
}

impl WebSocket {
    /// Establish a WebSocket connection.
    ///
    /// This function may error in the following cases:
    /// - The port to which the connection is being attempted is being blocked.
    /// - The URL is invalid.
    ///
    /// The error returned is [`JsError`]. See the
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket#exceptions_thrown)
    /// to learn more.
    pub fn open(url: &str) -> Result<Self, JsError> {
        Self::setup(web_sys::WebSocket::new(url))
    }

    /// Establish a WebSocket connection.
    ///
    /// This function may error in the following cases:
    /// - The port to which the connection is being attempted is being blocked.
    /// - The URL is invalid.
    /// - The specified protocol is not supported
    ///
    /// The error returned is [`JsError`]. See the
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket#exceptions_thrown)
    /// to learn more.
    pub fn open_with_protocol(url: &str, protocol: &str) -> Result<Self, JsError> {
        Self::setup(web_sys::WebSocket::new_with_str(url, protocol))
    }

    /// Establish a WebSocket connection.
    ///
    /// This function may error in the following cases:
    /// - The port to which the connection is being attempted is being blocked.
    /// - The URL is invalid.
    /// - The specified protocols are not supported
    /// - The protocols cannot be converted to a JSON string list
    ///
    /// The error returned is [`JsError`]. See the
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket#exceptions_thrown)
    /// to learn more.
    ///
    /// This function requires `json` features because protocols are parsed by `serde` into `JsValue`.
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    #[cfg(feature = "json")]
    pub fn open_with_protocols<S: AsRef<str> + serde::Serialize>(
        url: &str,
        protocols: &[S],
    ) -> Result<Self, JsError> {
        let json = <JsValue as gloo_utils::format::JsValueSerdeExt>::from_serde(protocols)
            .map_err(|err| {
                js_sys::Error::new(&format!(
                    "Failed to convert protocols to Javascript value: {err}"
                ))
            })?;
        Self::setup(web_sys::WebSocket::new_with_str_sequence(url, &json))
    }

    fn setup(ws: Result<web_sys::WebSocket, JsValue>) -> Result<Self, JsError> {
        let waker: Arc<RefCell<Option<Waker>>> = Arc::new(RefCell::new(None));
        let ws = ws.map_err(js_to_js_error)?;

        // We rely on this because the other type Blob can be converted to Vec<u8> only through a
        // promise which makes it awkward to use in our event callbacks where we want to guarantee
        // the order of the events stays the same.
        ws.set_binary_type(BinaryType::Arraybuffer);

        let (sender, receiver) = mpsc::unbounded();

        let open_callback: Closure<dyn FnMut()> = {
            let waker = Arc::clone(&waker);
            Closure::wrap(Box::new(move || {
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut()>)
        };

        ws.add_event_listener_with_callback_and_add_event_listener_options(
            "open",
            open_callback.as_ref().unchecked_ref(),
            web_sys::AddEventListenerOptions::new().once(true),
        )
            .map_err(js_to_js_error)?;

        let message_callback: Closure<dyn FnMut(MessageEvent)> = {
            let sender = sender.clone();
            Closure::wrap(Box::new(move |e: MessageEvent| {
                let msg = parse_message(e);
                let _ = sender.unbounded_send(StreamMessage::Message(msg));
            }) as Box<dyn FnMut(MessageEvent)>)
        };

        ws.add_event_listener_with_callback("message", message_callback.as_ref().unchecked_ref())
            .map_err(js_to_js_error)?;

        let error_callback: Closure<dyn FnMut(web_sys::Event)> = {
            let sender = sender.clone();
            let waker = Arc::clone(&waker);
            Closure::wrap(Box::new(move |_e: web_sys::Event| {
                if let Some(waker) = waker.borrow_mut().take() {
                    waker.wake();
                }
                let _ = sender.unbounded_send(StreamMessage::ErrorEvent);
            }) as Box<dyn FnMut(web_sys::Event)>)
        };

        ws.add_event_listener_with_callback("error", error_callback.as_ref().unchecked_ref())
            .map_err(js_to_js_error)?;

        let close_callback: Closure<dyn FnMut(web_sys::CloseEvent)> = {
            Closure::wrap(Box::new(move |e: web_sys::CloseEvent| {
                let close_event = CloseEvent {
                    code: e.code(),
                    reason: e.reason(),
                    was_clean: e.was_clean(),
                };
                let _ = sender.unbounded_send(StreamMessage::CloseEvent(close_event));
                let _ = sender.unbounded_send(StreamMessage::ConnectionClose);
            }) as Box<dyn FnMut(web_sys::CloseEvent)>)
        };

        ws.add_event_listener_with_callback_and_add_event_listener_options(
            "close",
            close_callback.as_ref().unchecked_ref(),
            web_sys::AddEventListenerOptions::new().once(true),
        )
            .map_err(js_to_js_error)?;

        Ok(Self {
            ws,
            sink_waker: waker,
            message_receiver: receiver,
            closures: (
                open_callback,
                message_callback,
                error_callback,
                close_callback,
            ),
            read_pending_bytes: Arc::new(Mutex::new(None)),
        })
    }

    /// Closes the websocket.
    ///
    /// See the [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close#parameters)
    /// to learn about parameters passed to this function and when it can return an `Err(_)`
    pub fn close(self, code: Option<u16>, reason: Option<&str>) -> Result<(), JsError> {
        let result = match (code, reason) {
            (None, None) => self.ws.close(),
            (Some(code), None) => self.ws.close_with_code(code),
            (Some(code), Some(reason)) => self.ws.close_with_code_and_reason(code, reason),
            // default code is 1005 so we use it,
            // see: https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close#parameters
            (None, Some(reason)) => self.ws.close_with_code_and_reason(1005, reason),
        };
        result.map_err(js_to_js_error)
    }

    /// The current state of the websocket.
    pub fn state(&self) -> State {
        let ready_state = self.ws.ready_state();
        match ready_state {
            0 => State::Connecting,
            1 => State::Open,
            2 => State::Closing,
            3 => State::Closed,
            _ => unreachable!(),
        }
    }

    /// The extensions in use.
    pub fn extensions(&self) -> String {
        self.ws.extensions()
    }

    /// The sub-protocol in use.
    pub fn protocol(&self) -> String {
        self.ws.protocol()
    }
}

impl TryFrom<web_sys::WebSocket> for WebSocket {
    type Error = JsError;

    fn try_from(ws: web_sys::WebSocket) -> Result<Self, Self::Error> {
        Self::setup(Ok(ws))
    }
}

#[derive(Clone)]
enum StreamMessage {
    ErrorEvent,
    CloseEvent(CloseEvent),
    Message(Message),
    ConnectionClose,
}

fn parse_message(event: MessageEvent) -> Message {
    if let Ok(array_buffer) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
        let array = js_sys::Uint8Array::new(&array_buffer);
        Message::Bytes(array.to_vec())
    } else if let Ok(txt) = event.data().dyn_into::<js_sys::JsString>() {
        Message::Text(String::from(&txt))
    } else {
        unreachable!("message event, received Unknown: {:?}", event.data());
    }
}

impl Sink<Message> for WebSocket {
    type Error = WebSocketError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let ready_state = self.ws.ready_state();
        if ready_state == 0 {
            *self.sink_waker.borrow_mut() = Some(cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        let result = match item {
            Message::Bytes(bytes) => self.ws.send_with_u8_array(&bytes),
            Message::Text(message) => self.ws.send_with_str(&message),
        };
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(WebSocketError::MessageSendError(js_to_js_error(e))),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl Stream for WebSocket {
    type Item = Result<Message, WebSocketError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let msg = ready!(self.project().message_receiver.poll_next(cx));
        match msg {
            Some(StreamMessage::Message(msg)) => Poll::Ready(Some(Ok(msg))),
            Some(StreamMessage::ErrorEvent) => {
                Poll::Ready(Some(Err(WebSocketError::ConnectionError)))
            }
            Some(StreamMessage::CloseEvent(e)) => {
                Poll::Ready(Some(Err(WebSocketError::ConnectionClose(e))))
            }
            Some(StreamMessage::ConnectionClose) => Poll::Ready(None),
            None => Poll::Ready(None),
        }
    }
}

#[pinned_drop]
impl PinnedDrop for WebSocket {
    fn drop(self: Pin<&mut Self>) {
        self.ws.close().unwrap();

        for (ty, cb) in [
            ("open", self.closures.0.as_ref()),
            ("message", self.closures.1.as_ref()),
            ("error", self.closures.2.as_ref()),
        ] {
            let _ = self
                .ws
                .remove_event_listener_with_callback(ty, cb.unchecked_ref());
        }

        if let Ok(close_event) = web_sys::CloseEvent::new_with_event_init_dict(
            "close",
            web_sys::CloseEventInit::new()
                .code(1000)
                .reason("client dropped"),
        ) {
            let _ = self.ws.dispatch_event(&close_event);
        }
    }
}

macro_rules! try_in_poll_io {
    ($expr:expr) => {{
        match $expr {
            Ok(o) => o,
            // WebSocket is closed, nothing more to read or write
            Err(WebSocketError::ConnectionClose(event)) if event.was_clean => {
                return Poll::Ready(Ok(0));
            }
            Err(e) => return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e))),
        }
    }};
}

#[cfg_attr(docsrs, doc(cfg(feature = "io-util")))]
impl AsyncRead for WebSocket {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut data = if let Some(data) = self.as_mut().get_mut().read_pending_bytes.lock().unwrap().take() {
            data
        } else {
            match ready!(self.as_mut().poll_next(cx)) {
                Some(item) => match try_in_poll_io!(item) {
                    Message::Text(s) => s.into_bytes(),
                    Message::Bytes(data) => data,
                },
                None => return Poll::Ready(Ok(0)),
            }
        };

        let bytes_to_copy = cmp::min(buf.len(), data.len());
        buf[..bytes_to_copy].copy_from_slice(&data[..bytes_to_copy]);

        if data.len() > bytes_to_copy {
            data.drain(..bytes_to_copy);
            self.get_mut().read_pending_bytes.lock().unwrap().replace(data);
        }

        Poll::Ready(Ok(bytes_to_copy))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "io-util")))]
impl AsyncWrite for WebSocket {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // try flushing preemptively
        let _ = AsyncWrite::poll_flush(self.as_mut(), cx);

        // make sure sink is ready to send
        try_in_poll_io!(ready!(self.as_mut().poll_ready(cx)));

        // actually submit new item
        try_in_poll_io!(self.start_send(Message::Bytes(buf.to_vec())));
        // ^ if no error occurred, message is accepted and queued when calling `start_send`
        // (i.e.: `to_vec` is called only once)

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let res = ready!(Sink::poll_flush(self, cx));
        Poll::Ready(ws_result_to_io_result(res))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let res = ready!(Sink::poll_close(self, cx));
        Poll::Ready(ws_result_to_io_result(res))
    }
}

fn ws_result_to_io_result(res: Result<(), WebSocketError>) -> io::Result<()> {
    match res {
        Ok(()) => Ok(()),
        Err(WebSocketError::ConnectionClose(_)) => Ok(()),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
    }
}