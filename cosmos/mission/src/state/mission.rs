use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Map, SnapshotItem};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::{AsRefStr, Display};

use crate::{block::BlockTime, snapshot_item};

use super::error::StateError;

pub const MISSION: Map<Addr, Vec<Mission>> = Map::new("mission");

pub const RECENTLY_MISSION_LIMIT: usize = 100;
pub const RECENTLY_MISSION_LIST: SnapshotItem<Vec<Mission>> = snapshot_item!(
    "recently_mission_list",
    cw_storage_plus::Strategy::EveryBlock
);

#[derive(
    Deserialize_repr, Serialize_repr, Display, AsRefStr, Clone, Debug, PartialEq, Eq, JsonSchema,
)]
#[strum(serialize_all = "snake_case")]
#[repr(u8)]
pub enum Status {
    /// lock
    Open = 0,
    /// refund
    Success = 1,
    /// kept in the treausry
    Failed = 2,
}

impl Default for Status {
    fn default() -> Self {
        Status::Open
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Mission {
    pub title: String,
    pub coin: Coin,
    pub status: Status,
    /// seconds\
    /// if the mission is not successful before the specified value, it is automatically treated as `Status::Failed`
    pub ends_at: u64,
    pub postscript: Option<String>,
}

impl Mission {
    pub fn new(title: impl ToString, coin: Coin, ends_at: u64) -> Self {
        Self {
            title: title.to_string(),
            coin,
            status: Status::default(),
            ends_at,
            postscript: None,
        }
    }

    pub fn is_expired(&mut self, block_time: &BlockTime) -> bool {
        let is_expired = self.ends_at < block_time.time;

        if is_expired {
            let _ = self.new_status(Status::Failed);
        }

        is_expired
    }

    pub fn new_status(&mut self, new_status: Status) -> Result<&mut Self, StateError> {
        if self.status != Status::Open {
            return Err(StateError::InvalidStatusChangeRequest {
                current: self.status.clone(),
                target: new_status,
            });
        }

        match new_status {
            Status::Success | Status::Failed => self.status = new_status,
            _ => {
                return Err(StateError::InvalidStatusChangeRequest {
                    current: self.status.clone(),
                    target: new_status,
                })
            }
        }

        Ok(self)
    }
}

impl From<crate::msg::CreateMissionItem> for Mission {
    fn from(item: crate::msg::CreateMissionItem) -> Self {
        Self::new(item.title, item.coin, item.ends_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_mission() -> Mission {
        Mission::new("".to_owned(), Coin::default(), 0)
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", Status::Open), "open");
        assert_eq!(format!("{}", Status::Success), "success");
        assert_eq!(format!("{}", Status::Failed), "failed");
    }

    #[test]
    fn serialize() {
        fn assert_repr_eq(status: Status, expected: &str) {
            let status = serde_json::to_string(&status).unwrap();
            assert_eq!(status, expected);
        }

        assert_repr_eq(Status::Open, "0");
        assert_repr_eq(Status::Success, "1");
        assert_repr_eq(Status::Failed, "2");
    }

    #[test]
    fn deserialize() {
        fn assert_repr_eq(text: &str, expected: Status) {
            let status: Status = serde_json::from_str(text).unwrap();
            assert_eq!(status, expected);
        }

        assert_repr_eq("0", Status::Open);
        assert_repr_eq("1", Status::Success);
        assert_repr_eq("2", Status::Failed);
    }

    #[test]
    fn mission_default() {
        let mission = default_mission();

        assert_eq!(mission.status, Status::Open);
        assert_eq!(mission.postscript, None);
    }

    #[test]
    fn new_status() {
        let mut mission = default_mission();

        assert_eq!(
            mission.new_status(Status::Open),
            Err(StateError::InvalidStatusChangeRequest {
                current: Status::Open,
                target: Status::Open,
            })
        );
        assert!(
            mission.new_status(Status::Failed).is_ok(),
            "failed change request open to failed"
        );
        assert_eq!(
            mission.new_status(Status::Open),
            Err(StateError::InvalidStatusChangeRequest {
                current: Status::Failed,
                target: Status::Open
            })
        );
    }
}
