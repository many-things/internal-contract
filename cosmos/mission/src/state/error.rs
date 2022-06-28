use thiserror::Error;

use super::mission::Status;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum StateError {
    #[error("cannot change from {current} to {target}")]
    InvalidStatusChangeRequest { current: Status, target: Status },
}
