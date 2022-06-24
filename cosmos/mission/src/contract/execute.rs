use crate::{msg::ExecuteMsg, result::ContractResult, state::STATE, ContractError};

cfg_if::cfg_if! {
    if #[cfg(not(feature = "library"))] {
        use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
        use cosmwasm_std::entry_point;

        #[entry_point]
        pub fn execute(
            deps: DepsMut,
            _env: Env,
            info: MessageInfo,
            msg: ExecuteMsg,
        ) -> ContractResult<Response> {
            match msg {
                ExecuteMsg::Increment {} => try_increment(deps),
                ExecuteMsg::Reset { count } => try_reset(deps, info, count),
            }
        }
    }
}

pub fn try_increment(deps: DepsMut) -> ContractResult<Response> {
    STATE.update(deps.storage, |mut state| -> ContractResult<_> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}

pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> ContractResult<Response> {
    STATE.update(deps.storage, |mut state| -> ContractResult<_> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "reset"))
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        coins, from_binary,
        testing::{mock_dependencies_with_balance, mock_env, mock_info},
    };

    use crate::{
        contract::{execute::execute, instantiate, query::query},
        msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
        ContractError,
    };

    #[test]
    fn increment() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);

        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
