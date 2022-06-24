cfg_if::cfg_if! {
    if #[cfg(not(feature = "library"))] {
        use crate::msg::QueryMsg;
        use cosmwasm_std::{entry_point, to_binary, Binary, Deps, Env, StdResult};

        #[entry_point]
        pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
            match msg {
                QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
            }
        }
    }
}

use crate::{msg::CountResponse, state::STATE};

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(CountResponse { count: state.count })
}
