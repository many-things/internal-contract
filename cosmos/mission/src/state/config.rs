use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admin: Addr,
}

impl Config {
    pub fn new(admin: Addr) -> Self {
        Self { admin }
    }
}

impl AsRef<Addr> for Config {
    fn as_ref(&self) -> &Addr {
        &self.admin
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
