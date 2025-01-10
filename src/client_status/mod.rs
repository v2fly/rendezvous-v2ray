use std::rc::Rc;
use yew::Reducible;
use crate::client_status::core_link::CoreLink;

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
}

impl Reducible for ClientStatus {
    type Action = ClientStatusAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ClientStatusAction::SetCoreLink(core_link) => {
                Rc::new(ClientStatus {
                    ui_status: self.ui_status.clone(),
                    core_link,
                })
            }
            ClientStatusAction::SetUIStatus(ui_status) => {
                Rc::new(ClientStatus {
                    ui_status,
                    core_link: self.core_link.clone(),
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
            core_link: CoreLink {
                fetched_measurement: core_link::FetchedMeasurement::new(),
                fetched_subscription: core_link::FetchedSubscription::new(),
            },
        }
    }
}
