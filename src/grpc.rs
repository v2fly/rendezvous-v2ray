use std::sync::{Arc, Mutex};
use futures::StreamExt;
use gloo_console::__macro::JsValue;
use gloo_console::log;
use tonic_web_wasm_client::{Client as WasmGrpcWebClient};

use crate::grpc::proto::v2ray::core::app::observatory::observatory_command::observatory_service_client::ObservatoryServiceClient;
use crate::grpc::proto::v2ray::core::app::observatory::observatory_command::GetOutboundStatusRequest;

pub mod proto {
    pub mod v2ray {
        pub mod core {
            pub mod common {
                pub mod protoext {
                    tonic::include_proto!("v2ray.core.common.protoext");
                }
            }
            pub mod app {
                pub mod observatory {
                    tonic::include_proto!("v2ray.core.app.observatory");
                    pub mod observatory_command {
                        tonic::include_proto!("v2ray.core.app.observatory.command");
                    }
                }
                pub mod subscription {
                    tonic::include_proto!("v2ray.core.app.subscription");
                    pub mod subscriptionmanager {
                        pub mod command {
                            tonic::include_proto!(
                                "v2ray.core.app.subscription.subscriptionmanager.command"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GrpcClient {
    pub(crate) client: Arc<Mutex<WasmGrpcWebClient>>,
}

impl GrpcClient {
    pub fn client(&self) -> Arc<Mutex<impl tonic::client::GrpcService<tonic::body::BoxBody>>> {
        self.client.clone()
    }
}

pub async fn connect(base_url: String) -> GrpcClient {
    let client = WasmGrpcWebClient::new(base_url.parse().unwrap());

    return GrpcClient {
        client: Arc::new(Mutex::new(client)),
    };
}
