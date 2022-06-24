use {cosmwasm_std::StdError, thiserror::Error};

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("unauthorized")]
    Unauthorized {},
}
