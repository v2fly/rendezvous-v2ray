use super::Props;
use crate::{client_status::ui_status::UIStatus,client_status::ClientStatusAction};
use std::ops::Deref;
use yew::prelude::*;
use yew::{function_component, html, props, Html, Properties};
use yew_bootstrap::component::{BrandType, NavBar, NavDropdownItem, NavItem};

use crate::app_ui::settings::SettingsUI;
use crate::app_ui::subscription_list::SubscriptionListUI;

#[function_component]
pub fn PrimaryUI(props: &Props) -> Html {
    let brand = BrandType::BrandSimple {
        text: AttrValue::from("Rendezvous"),
        url: Some(AttrValue::from("")),
    };

    let client_status = props.client_status.clone();
    let update_client_status = props.update_client_status.clone();
    let ui_status = client_status.ui_status.clone();

    let click_on_subscription = {
        let ui_status = client_status.ui_status.clone();
        Callback::from(move |_| {
        update_client_status.emit(ClientStatusAction::SetUIStatus(UIStatus {
            active_window: "Subscription".to_string(),
            ..ui_status.clone()
        }));
    })
    };

    let client_status = props.client_status.clone();
    let update_client_status = props.update_client_status.clone();

    let click_on_setting = {
        let ui_status = client_status.ui_status.clone();
        Callback::from(move |_| {
        update_client_status.emit(ClientStatusAction::SetUIStatus(UIStatus {
            active_window: "Setting".to_string(),
            ..ui_status.clone()
        }));
    })};

    if props.client_status.ui_status.active_window.clone().eq("") {
        let update_client_status = props.update_client_status.clone();
        update_client_status.emit(ClientStatusAction::SetUIStatus(UIStatus {
            active_window: "Setting".to_string(),
            ..ui_status.clone()
        }));
        return html! { <>{"Loading"}</> };
    }

    html! {
        <div>
            <div class={classes!("d-none")} >{"active_UI="}{props.client_status.ui_status.active_window.clone()} </div>
            <div class={classes!("border-bottom")}>
                <NavBar nav_id={"test-nav"} class="navbar-expand-lg navbar-light bg-light" brand={brand}>
                    <NavItem text="Subscription"
                        active={props.client_status.ui_status.active_window.clone().eq("Subscription")}
                        onclick={click_on_subscription} url="#"/>
                <NavItem text="Setting"
                    active={props.client_status.ui_status.active_window.clone().eq("Setting")}
                    onclick={click_on_setting} url="#"/>
                </NavBar>
            </div>
            {match props.client_status.ui_status.active_window.as_str() {
                "Subscription" => html! { <SubscriptionListUI client_status={props.client_status.clone()} update_client_status={props.update_client_status.clone()} /> },
                "Setting" => html! { <SettingsUI client_status={props.client_status.clone()} update_client_status={props.update_client_status.clone()} /> },
                _ => html! { <div>{"Loading"}</div> }
            }}
        </div>
    }
}
