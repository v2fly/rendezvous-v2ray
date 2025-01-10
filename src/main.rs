mod app;
mod grpc;
mod client_status;
mod app_ui;
mod background;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
