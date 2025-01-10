use crate::app_ui::Props;
use crate::client_status::{ClientStatus, ClientStatusAction};
use crate::grpc::proto::v2ray::core::app::subscription::SubscriptionServer;
use std::collections::BTreeMap;
use std::fmt::format;
use std::ops::Deref;
use yew::prelude::*;
use yew::{function_component, Html};
use yew_bootstrap::component::{Accordion, AccordionItem, Badge, ListGroup, ListGroupItem};
use yew_bootstrap::util::Color;

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
    html! {
        <div>
            <h5> {&diplay_name} </h5>
            <div> <b class={classes!("pe-1")}> {"Outbound Tag"} </b> {&outbound_tag} </div>
            <div> <small> <b class={classes!("pe-1")}> {"Identifier"}  </b> {&props.name} </small> </div>
            {
                if let Some(observation) = observation_result {
                    match observation.alive {
                    true => {
                        html! {
                            <Badge style={Color::Primary}>{observation.delay} {"ms"}</Badge>
                        }
                    }
                    false => {
                        html! {
                            <Badge style={Color::Danger}>{"ERROR"}</Badge>
                        }
                    }
                }

                } else {
                    html! { <Badge style={Color::Secondary}>{"Unknown"}</Badge> }
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
                                        html_nested! {
                                            <ListGroupItem>
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
