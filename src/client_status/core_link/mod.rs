use crate::grpc::proto::v2ray::core::app::observatory;
use crate::grpc::proto::v2ray::core::app::router;
use crate::grpc::proto::v2ray::core::app::subscription;
use crate::grpc::GrpcClient;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::option::Option;
use std::sync::MutexGuard;
use gloo_console::log;
use tonic_web_wasm_client::Client;
use wasm_bindgen::JsValue;

#[derive(PartialEq, Debug, Clone)]
pub struct FetchedMeasurement {
    pub managed: BTreeMap<String, observatory::OutboundStatus>,
}

impl FetchedMeasurement {
    pub fn new() -> FetchedMeasurement {
        FetchedMeasurement {
            managed: BTreeMap::new(),
        }
    }

    pub async fn fetch_measurement(&mut self, grpc_client: GrpcClient) {
        println!("Fetching measurement");
        let grpc_client_copy = grpc_client.client.clone();
        {
            let mut received_client_lockguard = grpc_client_copy.lock().unwrap();
            let mut observatory_client =
                observatory::observatory_command::observatory_service_client::ObservatoryServiceClient::new(received_client_lockguard.deref_mut());
            let request = observatory::observatory_command::GetOutboundStatusRequest {
                tag: "".to_string(),
            };
            let response = observatory_client.get_outbound_status(request).await;
            match response {
                Ok(response) => {
                    let data = response.into_inner();
                    match data.status {
                        Some(observation_status) => {
                            observation_status.status.iter().for_each(|status| {
                                println!("Status: {:?}", status);
                                self.managed
                                    .insert(status.outbound_tag.clone(), status.clone());
                            });
                        }
                        None => {
                            println!("No status");
                        }
                    }
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct FetchedSubscription {
    pub managed: BTreeMap<String, Option<subscription::TrackedSubscriptionStatus>>,
}

impl FetchedSubscription {
    pub fn new() -> FetchedSubscription {
        FetchedSubscription {
            managed: BTreeMap::new(),
        }
    }

    pub async fn fetch_subscription_names(&mut self, grpc_client: GrpcClient) {
        println!("Fetching subscription");
        let grpc_client_copy = grpc_client.client.clone();
        {
            let mut received_client_lockguard = grpc_client_copy.lock().unwrap();
            let mut subscription_client =
                subscription::subscriptionmanager::command::subscription_manager_service_client::SubscriptionManagerServiceClient::new(received_client_lockguard.deref_mut());
            {
                let request =
                    subscription::subscriptionmanager::command::ListTrackedSubscriptionRequest {};
                let response = subscription_client.list_tracked_subscription(request).await;
                match response {
                    Ok(response) => {
                        let data = response.into_inner();

                        data.names.iter().for_each(|subscription| {
                            println!("Subscription: {:?}", subscription);
                            self.managed.insert(subscription.clone(), None);
                        });
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
        }
    }

    pub async fn fetch_subscription_content(&mut self, grpc_client: GrpcClient, name: String) {
        println!("Fetching subscription content");
        let grpc_client_copy = grpc_client.client.clone();
        {
            let mut received_client_lockguard = grpc_client_copy.lock().unwrap();
            let mut subscription_client =
                subscription::subscriptionmanager::command::subscription_manager_service_client::SubscriptionManagerServiceClient::new(received_client_lockguard.deref_mut());
            {
                let request =
                    subscription::subscriptionmanager::command::GetTrackedSubscriptionStatusRequest {
                        name: name.clone(),
                    };
                let response = subscription_client
                    .get_tracked_subscription_status(request)
                    .await;
                match response {
                    Ok(response) => {
                        let data = response.into_inner();
                        println!("Subscription content: {:?}", data);
                        self.managed
                            .insert(name.clone(), Some(data.status.unwrap()));
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct FetchedRouterStatus {
    pub managed: Option<router::router_command::BalancerMsg>,
}

impl FetchedRouterStatus {
    pub fn new() -> FetchedRouterStatus {
        FetchedRouterStatus { managed: None }
    }

    pub async fn fetch_router_status(&mut self, grpc_client: GrpcClient) {
        println!("Fetching router status");
        let grpc_client_copy = grpc_client.client.clone();
        {
            let mut received_client_lockguard = grpc_client_copy.lock().unwrap();
            let mut router_client =
                router::router_command::routing_service_client::RoutingServiceClient::new(
                    received_client_lockguard.deref_mut(),
                );
            {
                let request = router::router_command::GetBalancerInfoRequest {
                    tag: "subscriptions".to_string(),
                };
                let response = router_client.get_balancer_info(request).await;
                match response {
                    Ok(response) => {
                        let data = response.into_inner();
                        println!("Router status: {:?}", data);
                        self.managed = match data.balancer {
                            Some(balancer) => Some(balancer),
                            None => None,
                        }
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        self.managed = None;
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct CoreLink {
    pub fetched_measurement: FetchedMeasurement,
    pub fetched_subscription: FetchedSubscription,
    pub fetched_router_status: FetchedRouterStatus,
}

pub enum CoreLinkAction {
    SetPrimaryBalancerTarget(String),
}

async fn set_primary_balancer_target(grpc_client: GrpcClient, target: String) -> () {
    log!(<std::string::String as Into<JsValue>>::into(String::from(
                    "setting target"
    )));
    let grpc_client_copy = grpc_client.client.clone();

    let mut received_client_lockguard = grpc_client_copy.lock().unwrap();

    let mut router_client =
        router::router_command::routing_service_client::RoutingServiceClient::new(
            received_client_lockguard.deref_mut(),
        );
    let request = router::router_command::OverrideBalancerTargetRequest {
        balancer_tag: "subscriptions".to_string(),
        target: target.clone(),
    };
    let response = router_client.override_balancer_target(request);
    let _ = match response.await {
        Ok(response) => {
            println!("Set primary balancer target response: {:?}", response);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    };
}

impl CoreLink {
    pub fn new() -> CoreLink {
        CoreLink {
            fetched_measurement: FetchedMeasurement::new(),
            fetched_subscription: FetchedSubscription::new(),
            fetched_router_status: FetchedRouterStatus::new(),
        }
    }

    pub async fn apply_action(&self, grpc_client: GrpcClient, action: CoreLinkAction) {
        match action {
            CoreLinkAction::SetPrimaryBalancerTarget(target) => {
                set_primary_balancer_target(grpc_client, target).await;
            }
        }
    }
}
