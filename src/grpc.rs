use std::io::{Error, IoSlice, IoSliceMut, Read};
use std::pin::{pin, Pin};
use std::task::{Context, Poll};
use std::future::Future;
use gloo_console::__macro::JsValue;
use gloo_console::log;
use gloo_net::websocket::{Message, futures::WebSocket};
use wasm_bindgen_futures::spawn_local;
use futures::{pin_mut, AsyncRead, AsyncWrite, SinkExt, StreamExt};

use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use std::sync::Arc;
use async_compat::CompatExt;
use hyper::Uri;
use rustls_pki_types::ServerName;
use std::marker::PhantomData;
use futures::io::Compat;
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

impl AsyncReadTokio for WebsocketConnToReadWriter {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        return self.websocket.poll_read(cx, buf);
    }
}

impl AsyncWriteTokio for WebsocketConnToReadWriter {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        return self.websocket.poll_write(cx, buf);
    }

    fn poll_write_vectored(self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &[IoSlice<'_>]) -> Poll<std::io::Result<usize>> {
        return self.websocket.poll_write_vectored(cx, bufs);
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        return self.websocket.poll_flush(cx);
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        todo!()
    }
}
*/


#[derive(Debug, Clone, Copy)]
struct WasmJsSpawnLocal {}

impl<Fut> hyper::rt::Executor<Fut> for WasmJsSpawnLocal
where
    Fut: std::future::Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        tokio_with_wasm::spawn(fut);
    }
}

#[derive(Debug)]
pub struct TlsStream<S>(tokio_rustls::TlsStream<S>);


struct ControlConnectionConnector
{

}

impl ControlConnectionConnector
{
    async fn connect_host() -> Result<tokio_rustls::client::TlsStream<async_compat::Compat<WebSocket>>, Error> {
        let ws = WebSocket::open("wss://echo.websocket.org").unwrap();

        //let mut ws_read_write_closer = WebsocketConnToReadWriter::new(ws);

        let mut root_cert_store = RootCertStore::empty();
        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(config));
        let dnsname = ServerName::try_from("www.rust-lang.org").unwrap();

        let mut stream = connector.connect(dnsname, ws.compat()).await.unwrap();
        Ok(stream)
    }
    async fn connectw() -> Result<tokio_rustls::client::TlsStream<async_compat::Compat<WebSocket>>, Error> {
        let conn = Self::connect_host().await.unwrap();
        Ok(conn)
    }
}

impl tower_service::Service<Uri> for ControlConnectionConnector
{
    type Response = tokio_rustls::client::TlsStream<async_compat::Compat<WebSocket>>;
    type Error = std::io::Error;
    type Future = Pin<Box<
        dyn Future<Output=Result<Self::Response, Self::Error>> + Send
    >>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Uri) -> Self::Future {
        let conn = Self::connectw();
        Box::pin(conn)
    }
}

pub async fn connect() {
    let object = JsValue::from("any JsValue can be logged");
    log!("text", object);


    //hyper::client::conn::http2::handshake(TokioExecutor::new(), stream).await.unwrap();

    //let tokio_runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();


    let mut linkConnector = WebsocketTlsConnection::new();


    let http_client = hyper_util_wasm::client::legacy::Builder::new(
        WasmJsSpawnLocal {}).build(linkConnector);
}