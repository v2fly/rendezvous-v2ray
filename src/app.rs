use crate::app_ui::_Props::client_status;
use crate::app_ui::main_page::PrimaryUI;
use crate::background::BackgroundWorker;
use crate::client_status::{ClientStatus as AppClientStatus, ClientStatus};
use crate::grpc;
use lazy_static::lazy_static;
use std::ops::Deref;
use std::sync::{Arc, Mutex, Once};
use wasm_bindgen_futures::spawn_local;
use yew::html::IntoPropValue;
use yew::prelude::*;
use yew_bootstrap::component::{Container, ContainerSize};

static mut BACKGROUND_REFRESH: Option<Arc<Mutex<Option<BackgroundWorker>>>> = None;
static INIT: Once = Once::new();

pub fn get_background_refresh() -> Arc<Mutex<Option<BackgroundWorker>>> {
    unsafe {
        INIT.call_once(|| {
            BACKGROUND_REFRESH = Some(Arc::new(Mutex::new(None)));
        });
        match BACKGROUND_REFRESH {
            Some(ref x) => x.clone(),
            None => panic!("Failed to initialize background task"),
        }
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let ui_managed_client_status = use_reducer(move || AppClientStatus::new());

    let ui_managed_client_status_copy = (*ui_managed_client_status).clone();
    let ui_managed_client_status_copy_backgroundtask = (*ui_managed_client_status).clone();
    let ui_managed_client_status_copy_others = (*ui_managed_client_status).clone();


    let update_client_status = Callback::from(move |client_status_update| {
        let ui_managed_client_status = ui_managed_client_status.clone();
        ui_managed_client_status.dispatch(client_status_update);
    });

    {
        let background_mutex = get_background_refresh();
        let background_lock = background_mutex.lock();
        match background_lock {
            Ok(mut background) => {
                if background.is_none() {
                    let mut background_task = BackgroundWorker::new();
                    background_task.set_grpc("/api".to_string());
                    background_task.client_status =
                        Some(ui_managed_client_status_copy_backgroundtask.clone());
                    background_task.update_client_status = Some(update_client_status.clone());
                    background_task.refresh();
                    BackgroundWorker::self_refresh(get_background_refresh());
                    *background = Some(background_task);
                } else {
                    let mut background_task = background.as_mut().unwrap();
                    background_task.client_status =
                        Some(ui_managed_client_status_copy_others.clone());
                    background_task.update_client_status = Some(update_client_status.clone());
                }
            }
            Err(_) => {
                panic!("Failed to lock background task");
            }
        }
    }

    let onclick = Callback::from(move |_| {
        spawn_local(async {
            // grpc::connect().await;
            //background_task.refresh();
        });
    });

    html! {
        <main>
            <Container size={ContainerSize::Large} fluid={ true }>
            <h1 class={classes!("d-none")}>{"Hello, Yew!"}</h1>
            <button class={classes!("d-none")} {onclick}>{"Click me!"}</button>
            <PrimaryUI client_status={ui_managed_client_status_copy}
                update_client_status={update_client_status} />
            </Container>
        </main>
    }
}
