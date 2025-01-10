use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct UIStatus {
    #[serde()]
    pub(crate) active_window: String
}
