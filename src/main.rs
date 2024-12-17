mod app;
mod grpc;
mod gloo_net_websocket;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
