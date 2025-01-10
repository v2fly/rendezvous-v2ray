use crate::app_ui::_Props::update_client_status;
use crate::client_status::{ClientStatus, ClientStatusAction};
use futures::TryFutureExt;
use gloo_console::__macro::JsValue;
use gloo_console::log;
use gloo_timers::future::TimeoutFuture;
use std::sync::{Arc, Mutex};
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

#[derive(Clone, Debug)]
pub struct BackgroundWorker {
    grpc_url: String,
    pub client_status: Option<ClientStatus>,
    pub update_client_status: Option<Callback<ClientStatusAction>>,
}

impl BackgroundWorker {
    pub fn new() -> BackgroundWorker {
        BackgroundWorker {
            grpc_url: String::from(""),
            client_status: None,
            update_client_status: None,
        }
    }

    pub fn set_grpc(&mut self, url: String) -> Option<()> {
        self.grpc_url = url;
        Some(())
    }

    pub fn refresh(&self) {
        let grpc_url = self.grpc_url.clone();
        let client_status_copy = self.client_status.clone();
        let update_client_status_copy = self.update_client_status.clone();
        spawn_local(async move {
            let client = crate::grpc::connect(grpc_url).await;
            let mut client_status = client_status_copy.clone();
            match client_status {
                Some(mut client_status_unwrapped) => {
                    client_status_unwrapped
                        .core_link
                        .fetched_measurement
                        .fetch_measurement(client.clone())
                        .await;
                    client_status_unwrapped
                        .core_link
                        .fetched_subscription
                        .fetch_subscription_names(client.clone())
                        .await;
                    let subscription_names: Vec<_> = client_status_unwrapped
                        .core_link
                        .fetched_subscription
                        .managed
                        .iter()
                        .map(|(name, subscription)| name.clone())
                        .collect();
                    for subscription_name in subscription_names {
                        client_status_unwrapped
                            .core_link
                            .fetched_subscription
                            .fetch_subscription_content(client.clone(), subscription_name.clone())
                            .await;
                    }
                    update_client_status_copy
                        .unwrap()
                        .emit(crate::client_status::ClientStatusAction::SetCoreLink(client_status_unwrapped.core_link));
                }
                None => {
                    return;
                }
            }
        });
    }

    pub fn self_refresh(self_lock: Arc<Mutex<Option<BackgroundWorker>>>) {
        let self_copy = self_lock.clone();
        spawn_local(async move {
            loop {
                // Start the background worker
                let _ = TimeoutFuture::new(1000).await;
                log!(<std::string::String as Into<JsValue>>::into(String::from(
                    "refresh delayed"
                )));
                let self_lock = self_copy.lock().unwrap();
                match self_lock.as_ref() {
                    Some(worker) => {
                        worker.refresh();
                    }
                    None => {
                        log!(<std::string::String as Into<JsValue>>::into(String::from(
                            "worker not found"
                        )));
                    }
                }
            }
        });
    }
}
