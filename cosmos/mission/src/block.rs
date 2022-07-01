use cosmwasm_std::BlockInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug, Default)]
pub struct BlockTime {
    pub height: u64,
    pub time: u64,
}

impl From<BlockInfo> for BlockTime {
    fn from(info: BlockInfo) -> Self {
        Self {
            height: info.height,
            time: info.time.seconds(),
        }
    }
}
