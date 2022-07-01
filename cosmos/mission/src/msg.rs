use cosmwasm_std::{Coin, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::mission;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// admin api
    Withdraw {
        denom: String,
        amount: Option<Uint128>,
    },
    /// generate mission
    CreateMission(CreateMissionItem),
    CompleteMission {
        mission_id: usize,
        postscript: String,
    },
    FailedMission {
        mission_id: usize,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CreateMissionItem {
    pub title: String,
    pub coin: Coin,
    pub ends_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetBalanceList {
        denom: Option<Vec<String>>,
    },

    GetMissionList {
        address: Option<Vec<String>>,
        status: Option<Vec<mission::Status>>,
        denom: Option<Vec<String>>,

        limit: Option<usize>,
        offset: Option<usize>,
    },
    GetRecentlyMissionList {
        limit: Option<usize>,
    },
}
