use cosmwasm_std::{BlockInfo, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct BlockTime {
    pub height: u64,
    pub time: Timestamp,
}

impl From<BlockInfo> for BlockTime {
    fn from(info: BlockInfo) -> Self {
        Self {
            height: info.height,
            time: info.time,
        }
    }
}
