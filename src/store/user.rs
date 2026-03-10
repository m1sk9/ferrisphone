use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::JsonStore;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub model: Option<String>,
}

pub type UserStore = JsonStore<HashMap<u64, UserSettings>>;
