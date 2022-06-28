use cosmwasm_std::{Addr, OverflowError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error(transparent)]
    Overflow(#[from] OverflowError),

    #[error("not found mission from ({sender}: {index})")]
    NotFoundMission { sender: Addr, index: usize },
}
