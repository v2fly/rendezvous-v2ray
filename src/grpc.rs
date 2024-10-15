use std::io::{IoSlice, IoSliceMut, Read};
use std::pin::{pin, Pin};
use std::task::{Context, Poll};
use gloo_console::__macro::JsValue;
use gloo_console::log;
use gloo_net::websocket::{Message, futures::WebSocket};
use wasm_bindgen_futures::spawn_local;
use futures::{pin_mut, AsyncRead, AsyncWrite, SinkExt, StreamExt};
use bytes::buf::Buf;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use std::sync::Arc;
use rustls_pki_types::ServerName;

/*
struct WebsocketConnToReadWriter {
    websocket: WebSocket,
    //next_reader: Option<>,
}

impl WebsocketConnToReadWriter {
    pub fn new(websocket: WebSocket) -> Self {
        Self {
            websocket,
            // next_reader: None
        }
    }
}

impl AsyncRead for WebsocketConnToReadWriter {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        return self.websocket.poll_read(cx, buf);
    }
    fn poll_read_vectored(self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &mut [IoSliceMut<'_>]) -> Poll<std::io::Result<usize>> {
        return self.websocket.poll_read_vectored(cx, bufs);
    }
}

impl AsyncWrite for WebsocketConnToReadWriter {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        return self.websocket.poll_write(cx, buf);
    }

    fn poll_write_vectored(self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &[IoSlice<'_>]) -> Poll<std::io::Result<usize>> {
        return self.websocket.poll_write_vectored(cx, bufs);
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        return self.websocket.poll_flush(cx);
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        return self.websocket.poll_close(cx);
    }
}
*/
pub async fn connect() {
    let object = JsValue::from("any JsValue can be logged");
    log!("text", object);

    let mut ws = WebSocket::open("wss://echo.websocket.org").unwrap();

    //let ws_read_write_closer = WebsocketConnToReadWriter::new(ws);

    let mut root_cert_store = RootCertStore::empty();
    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let dnsname = ServerName::try_from("www.rust-lang.org").unwrap();

    let mut stream = connector.connect(dnsname, &mut ws).await;
}