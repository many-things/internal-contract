use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::{
    result::ContractResult,
    state::config::{Config, CONFIG},
};

pub mod error;
pub mod execute;
pub mod query;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:contracts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

cfg_if::cfg_if! {
    if #[cfg(not(feature = "library"))] {
        use cosmwasm_std::entry_point;

        #[entry_point]
        pub fn instantiate(
            deps: DepsMut,
            _env: Env,
            info: MessageInfo,
            _msg: ()
        ) -> ContractResult<Response> {
            set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

            CONFIG.save(deps.storage, &Config::new(info.sender.clone()))?;

            Ok(Response::default()
                .add_attribute("version", CONTRACT_VERSION)
                .add_attribute("owner", info.sender)
            )
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use cosmwasm_std::{
//         coins, from_binary,
//         testing::{mock_dependencies, mock_env, mock_info},
//     };

//     use crate::msg::QueryMsg;

//     use super::*;

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies();

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query::query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }
// }
