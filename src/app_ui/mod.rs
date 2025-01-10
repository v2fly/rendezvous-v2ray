mod subscription_list;
pub(crate) mod main_page;
mod settings;

use crate::client_status::{ClientStatus, ClientStatusAction};
use yew::prelude::*;
use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub client_status: ClientStatus,
    pub update_client_status: Callback<ClientStatusAction>,
}
