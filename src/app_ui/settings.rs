use crate::app_ui::Props;
use yew::prelude::*;
use yew::{function_component, Html};
use yew_bootstrap::component::{Button, ButtonGroup};
use crate::client_status::ClientStatusAction::ApplyAction;
use crate::client_status::core_link::CoreLinkAction;

#[function_component]
pub fn RunningStatusUI(props: &Props) -> Html {
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

    let is_direct = {
        match override_target.clone() {
            Some(target) => match target.as_str() {
                "direct" => true,
                _ => false,
            },
            None => false,
        }
    };

    let is_blackhole = {
        match override_target.clone() {
            Some(target) => match target.as_str() {
                "deny" => true,
                _ => false,
            },
            None => false,
        }
    };

    let is_running = { !(is_direct || is_blackhole) };

    let onclick_running = {
        let update_client_status = props.update_client_status.clone();
        Callback::from(move |_| {
            let action = CoreLinkAction::SetPrimaryBalancerTarget("".to_string());
            update_client_status.emit(ApplyAction(action));
        })
    };

    let onclick_direct = {
        let update_client_status = props.update_client_status.clone();
        Callback::from(move |_| {
            let action = CoreLinkAction::SetPrimaryBalancerTarget("direct".to_string());
            update_client_status.emit(ApplyAction(action));
        })
    };

    let onclick_blackhole = {
        let update_client_status = props.update_client_status.clone();
        Callback::from(move |_| {
            let action = CoreLinkAction::SetPrimaryBalancerTarget("deny".to_string());
            update_client_status.emit(ApplyAction(action));
        })
    };

    html! {
        <div class={classes!("card")}>
            <div class={classes!("card-header")}>
                {"Operational Status"}
            </div>
            <div class={classes!("card-body")}>
                <ButtonGroup>
                    <Button
                        outline=true onclick={onclick_running} class={
                            if is_running {
                                "active"
                            } else {
                                ""
                            }
                        }>{"Running"}</Button>
                    <Button
                        outline=true onclick={onclick_direct} class={
                            if is_direct {
                                "active"
                            } else {
                                ""
                            }
                            }>{"Direct"}</Button>
                    <Button outline=true onclick={onclick_blackhole} class={
                            if is_blackhole {
                                "active"
                            } else {
                                ""
                            }
                            }>{"Blackhole"}</Button>
                    </ButtonGroup>
            </div>
        </div>
    }
}

#[function_component]
pub fn SettingsUI(props: &Props) -> Html {
    html! {
        <div>
            <RunningStatusUI client_status={props.client_status.clone()} update_client_status={props.update_client_status.clone()} />
        </div>
    }
}
