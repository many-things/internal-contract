use cosmwasm_std::Uint128;
use cw_storage_plus::Map;

/// Map<denom, Uint128>
pub const BALANCE: Map<String, Uint128> = Map::new("balance");
/// Map<denom, Uint128>
pub const LOCK_BALANCE: Map<String, Uint128> = Map::new("lock_balance");
