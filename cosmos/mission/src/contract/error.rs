use cosmwasm_std::{Addr, OverflowError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error(transparent)]
    Overflow(#[from] OverflowError),

    #[error("not found mission from ({sender}: {index})")]
    NotFoundMission { sender: Addr, index: usize },

    #[error("not found denom")]
    NotFoundDenom { denom: String },

    #[error("lack of balance")]
    LackOfBalance { maximum: Uint128 },

    #[error("unauthorized address: {addr}")]
    Unauthorized { addr: Addr },
}
