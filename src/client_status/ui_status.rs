use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct UIStatus {
    #[serde()]
    pub(crate) active_window: String,
    #[serde()]
    pub(crate) subscription_add_new_card_open: bool,
}
