use std::collections::HashMap;

use super::JsonStore;

pub type MemoryStore = JsonStore<HashMap<u64, String>>;
