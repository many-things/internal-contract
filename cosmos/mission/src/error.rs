use cosmwasm_std::StdError;
use thiserror::Error;

use crate::{contract::error::ExecuteError, state::error::StateError};

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Cosmwasm(#[from] StdError),

    #[error(transparent)]
    Execute(#[from] ExecuteError),

    #[error(transparent)]
    State(#[from] StateError),
}
