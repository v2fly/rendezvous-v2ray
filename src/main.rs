mod app;
mod grpc;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
