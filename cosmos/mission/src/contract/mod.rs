use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    msg::InstantiateMsg,
    state::{State, STATE},
};

pub mod execute;
pub mod query;

/// version info for migration info
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
            msg: InstantiateMsg,
        ) -> Result<Response, ContractError> {
            let state = State {
                count: msg.count,
                owner: info.sender.clone(),
            };
            set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
            STATE.save(deps.storage, &state)?;

            Ok(Response::new()
                .add_attribute("method", "instantiate")
                .add_attribute("owner", info.sender)
                .add_attribute("count", msg.count.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    use crate::msg::{CountResponse, QueryMsg};

    use super::*;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query::query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }
}
