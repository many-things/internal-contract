use crate::ContractError;

pub type ContractResult<T, E = ContractError> = std::result::Result<T, E>;
