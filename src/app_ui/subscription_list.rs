
use crate::app_ui::Props;
use crate::client_status::{ClientStatus, ClientStatusAction};
use crate::grpc::proto::v2ray::core::app::subscription::SubscriptionServer;
use crate::client_status::core_link::CoreLinkAction;
use std::collections::BTreeMap;
use std::ops::Deref;
use gloo_console::log;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew::{function_component, Html};
use yew_bootstrap::component::{Accordion, AccordionItem, Badge, ListGroup, ListGroupItem};
use yew_bootstrap::util::Color;
use crate::client_status::ClientStatusAction::{ApplyAction, SyncNow};

#[derive(Properties, PartialEq)]
pub struct ProxyServerItemControlButtonProps {
    pub client_status: ClientStatus,
    pub update_client_status: Callback<ClientStatusAction>,
    pub name: String,
    pub server_info: SubscriptionServer,
    pub subscription_import_tag: String,
    pub currently_active: bool,
    pub currently_manually_selected: bool,
}

#[function_component]
pub fn ProxyServerItemControlButton(props: &ProxyServerItemControlButtonProps) -> Html {
    let currently_active_value = props.clone().currently_active;
    let currently_manually_selected = props.clone().currently_manually_selected;
    let outbound_tag = format!(
        "{}_{}",
        &props.subscription_import_tag, &props.server_info.tag
    );

    let on_manually_select_callback = {
        let currently_active  = props.currently_active;
        let update_client_status = props.update_client_status.clone();
        let our_tag = outbound_tag.clone();
        Callback::from(move |_| {
            log!(<std::string::String as Into<JsValue>>::into(String::from(
                    "button"
            )));
            if currently_manually_selected {
                let action = CoreLinkAction::SetPrimaryBalancerTarget("".to_string());
                update_client_status.emit(ApplyAction(action));
            }else {
                let action = CoreLinkAction::SetPrimaryBalancerTarget(our_tag.to_string());
                update_client_status.emit(ApplyAction(action));
            }
            update_client_status.emit(SyncNow());
        }) };

    html! {
        <div class={classes!("dropdown")}>
        {
            match currently_active_value {
                true => {
                    html! {
                        <button class={classes!("btn", "btn-light","btn-sm", "dropdown-toggle")} type="button" data-bs-toggle="dropdown" aria-expanded="false" >{"Action"}</button>
                    }
                }
                false => {
                    html! {
                        <button class={classes!("btn", "btn-outline-secondary","btn-sm", "dropdown-toggle")} type="button" data-bs-toggle="dropdown" aria-expanded="false" >{"Action"}</button>
                    }
                }
            }
        }
          <ul class={classes!("dropdown-menu")}>
                <il> <button class={classes!("dropdown-item")} onclick={on_manually_select_callback} type="button"> {"Manually Select"} </button> </il>
          </ul>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ProxyServerItemProps {
    pub client_status: ClientStatus,
    pub update_client_status: Callback<ClientStatusAction>,
    pub name: String,
    pub server_info: SubscriptionServer,
    pub subscription_import_tag: String,
}

#[function_component]
pub fn ProxyServerItemUI(props: &ProxyServerItemProps) -> Html {
    let outbound_tag = format!(
        "{}_{}",
        &props.subscription_import_tag, &props.server_info.tag
    );
    let observation_result = props
        .client_status
        .core_link
        .fetched_measurement
        .managed
        .get(&outbound_tag);
    let diplay_name = {
        let metadata_display_name = props.clone().server_info.server_metadata.get("DisplayName");
        match metadata_display_name {
            Some(display_name) => display_name.clone(),
            None => outbound_tag.clone(),
        }
    };

    let principle_target = {
        let principle_target = props
            .client_status
            .core_link
            .fetched_router_status
            .managed
            .clone();
        match principle_target {
            Some(router_balancer_status) => {
                let selected_tag = router_balancer_status.principle_target.clone();
                match selected_tag {
                    Some(tag) => tag.tag.get(0).unwrap_or(&"".to_string()).clone(),
                    None => "".to_string(),
                }
            }
            None => "".to_string(),
        }
    };

    let override_target = {
        let override_target = props
            .client_status
            .core_link
            .fetched_router_status
            .managed
            .clone();
        match override_target {
            Some(router_balancer_status) => {
                let current_override = router_balancer_status.clone().r#override.unwrap().target;
                match current_override.as_str() {
                    "" => None,
                    _ => Some(current_override.clone()),
                }
            }
            None => None,
        }
    };

    let selected_target = {
        match override_target.clone() {
            Some(override_target) => override_target.clone(),
            None => principle_target.clone(),
        }
    };


    let is_selected = selected_target == outbound_tag;
    let is_primary_target = principle_target == outbound_tag && is_selected;
    let is_override_target = override_target.is_some() && override_target.unwrap_or(principle_target.clone()) == outbound_tag && is_selected;

    html! {
        <div>
            <div class={classes!("d-flex")}>
                <h5 class={classes!("w-100")}> {&diplay_name} </h5>
                <div class={classes!("flex-shrink-1")}>
                    <ProxyServerItemControlButton client_status={props.client_status.clone()}
                        update_client_status={props.update_client_status.clone()}
                        name={props.name.clone()}
                        server_info={props.server_info.clone()}
                        subscription_import_tag={props.subscription_import_tag.clone()}
                        currently_active={is_selected} currently_manually_selected={is_override_target}/>
                </div>
            </div>
            <div> <b class={classes!("pe-1")}> {"Outbound Tag"} </b> {&outbound_tag} </div>
            <div> <small> <b class={classes!("pe-1")}> {"Identifier"}  </b> {&props.name} </small> </div>
            {
                if let Some(observation) = observation_result {
                    match observation.alive {
                    true => {
                        match is_selected {
                            true => {
                                html!{
                                    <Badge class={"me-1"} style={Color::Light}>{observation.delay} {"ms"}</Badge>
                                }
                            }
                            false => {
                                html! {
                                    <Badge class={"me-1"} style={Color::Info}>{observation.delay} {"ms"}</Badge>
                                }
                            }
                        }
                    }
                    false => {
                        html! {
                            <Badge class={"me-1"} style={Color::Danger}>{"ERROR"}</Badge>
                        }
                    }
                }

                } else {
                    html! { <Badge class={"me-1"} style={Color::Secondary}>{"Unknown"}</Badge> }
                }
            }
            {
                if is_primary_target {
                    html! {
                        <Badge class={"me-1"} style={Color::Success}>{"Automatically Selected"}</Badge>
                    }
                } else {
                    html! {
                        <></>
                    }
                }
            }
            {
                if is_override_target {
                    html! {
                        <Badge class={"me-1"} style={Color::Success}>{"Manually Selected"}</Badge>
                    }
                } else {
                    html! {
                        <></>
                    }
                }
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SubscriptionItemProps {
    pub client_status: ClientStatus,
    pub update_client_status: Callback<ClientStatusAction>,
    pub displayed_subscription_name: String,
}

#[function_component]
pub fn SubscriptionListItemUI(props: &SubscriptionItemProps) -> Html {
    let subscription_content = props
        .client_status
        .core_link
        .fetched_subscription
        .managed
        .get(&props.displayed_subscription_name);

    html! {
        <div>
            {
                if let Some(subscription) = subscription_content {
                    if let Some(tracked_subscription_status) = &subscription {
                        html! {
                            <ListGroup>
                                {
                                    for BTreeMap::from_iter(tracked_subscription_status.servers.iter()).iter().map(|(name, serverinfo)| {
                                        let cloned_name = name.clone().clone();
                                        let cloned_serverinfo = serverinfo.clone().clone();
                                        let subscription_import_tag = tracked_subscription_status.clone().import_source.unwrap().tag_prefix.clone();

                                        let is_active =  match &props.client_status.core_link.fetched_router_status.managed {
                                            None => {
                                                false
                                            }
                                            Some(router_balancer_status) => {
                                                let selected_tag = router_balancer_status.principle_target.clone();
                                                let selected_tag = match selected_tag {
                                                    Some(tag) => tag.tag.get(0).unwrap_or(&"".to_string()).clone(),
                                                    None => "".to_string(),
                                                };
                                                let selected_tag_applied_override = match router_balancer_status.r#override.clone() {
                                                    Some(override_data) =>{
                                                        match override_data.target.clone().as_str() {
                                                            "" => selected_tag.clone(),
                                                            _ => override_data.target.clone()
                                                        }
                                                    },
                                                    None => selected_tag.clone(),
                                                };

                                                selected_tag_applied_override == format!("{}_{}", subscription_import_tag, cloned_serverinfo.tag)
                                            }
                                        };

                                        html_nested! {
                                            <ListGroupItem active={is_active} >
                                                <ProxyServerItemUI name={cloned_name}
                                                    server_info={cloned_serverinfo}
                                                    client_status={props.client_status.clone()}
                                                    update_client_status={props.update_client_status.clone()}
                                                    subscription_import_tag={subscription_import_tag.clone()}/>
                                            </ListGroupItem>
                                        }
                                    })
                                }
                            </ListGroup>
                        }
                    } else {
                        html! { <div>{"Measurement not found"}</div> }
                    }
                } else {
                    html! { <div>{"Subscription not found"}</div> }
                }
            }
        </div>
    }
}

#[function_component]
pub fn SubscriptionListUI(props: &Props) -> Html {
    html! {
        <div>
         <div class={classes!("d-none")}>{"Subscription List"}</div>
        <Accordion>
            {
                for props.client_status.core_link.fetched_subscription.managed.iter().map(|(name, subscription)| {
                    html_nested! {
                        <AccordionItem title={name.clone()}>
                            <SubscriptionListItemUI
                                client_status={props.client_status.clone()}
                                update_client_status={props.update_client_status.clone()}
                                displayed_subscription_name={name.clone()} />
                        </AccordionItem>
                    }
                })
            }
        </Accordion>
        </div>
    }
}
