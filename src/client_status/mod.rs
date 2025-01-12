use crate::background::BackgroundWorker;
use crate::client_status::core_link::{CoreLink, CoreLinkAction};
use std::rc::Rc;
use gloo_console::log;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::Reducible;
pub mod core_link;
pub mod ui_status;

#[derive(PartialEq, Debug, Clone)]
pub struct ClientStatus {
    pub ui_status: ui_status::UIStatus,
    pub core_link: CoreLink,
}

pub enum ClientStatusAction {
    SetCoreLink(CoreLink),
    SetUIStatus(ui_status::UIStatus),
    ApplyAction(CoreLinkAction),
}

impl Reducible for ClientStatus {
    type Action = ClientStatusAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ClientStatusAction::SetCoreLink(core_link) => Rc::new(ClientStatus {
                ui_status: self.ui_status.clone(),
                core_link,
            }),
            ClientStatusAction::SetUIStatus(ui_status) => Rc::new(ClientStatus {
                ui_status,
                core_link: self.core_link.clone(),
            }),
            ClientStatusAction::ApplyAction(core_link_action) => {
                let core_link = self.core_link.clone();
                let background_refresh = crate::app::get_background_refresh();
                {
                    let core_link_clone = core_link.clone();
                    spawn_local(async move {
                        BackgroundWorker::apply_action(background_refresh, move |grpc| {
                            spawn_local(async move {
                                core_link_clone.apply_action(grpc, core_link_action).await;
                            });
                        }).await;
                    });
                }
                Rc::new(ClientStatus {
                    ui_status: self.ui_status.clone(),
                    core_link: core_link.clone(),
                })
            }
        }
    }
}

impl ClientStatus {
    pub fn new() -> ClientStatus {
        ClientStatus {
            ui_status: ui_status::UIStatus {
                active_window: String::from(""),
            },
            core_link: CoreLink::new(),
        }
    }
}
