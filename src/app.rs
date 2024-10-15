use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[path = "grpc.rs"]
mod grpc;

#[function_component(App)]
pub fn app() -> Html {
    let onclick = Callback::from(move |_| {
        spawn_local(async {
            grpc::connect().await;
        });

    });

    html! {
        <main>
            <h1>{"Hello, Yew!"}</h1>
            <button {onclick}>{"Click me!"}</button>
        </main>
    }
}
