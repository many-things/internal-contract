use cosmwasm_std::{Addr, BankMsg, Coin, Storage, Uint128};

use crate::{
    block::BlockTime,
    msg::{CreateMissionItem, ExecuteMsg},
    result::ContractResult,
    state::{
        balance::BALANCE,
        config::CONFIG,
        mission::{
            next_id, Mission, Status, MISSION, RECENTLY_MISSION_LIMIT, RECENTLY_MISSION_LIST,
        },
    },
};

use super::error::ExecuteError;

cfg_if::cfg_if! {
if #[cfg(not(feature = "library"))] {
    use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> ContractResult<Response> {
        let storage = deps.storage;
        let sender = info.sender;
        let block_time: BlockTime = env.block.into();

        match msg {
            ExecuteMsg::Withdraw { denom, amount } => try_withdraw(storage, sender, denom, amount),
            ExecuteMsg::CreateMission(item) => try_create_mission(storage, block_time.height, sender, item),
            ExecuteMsg::CompleteMission { mission_id , postscript } => try_complete_mission(storage, block_time, sender, mission_id, postscript),
            ExecuteMsg::FailedMission { mission_id } => try_failed_mission(storage, sender, mission_id),
        }
    }
}
}

pub fn try_withdraw(
    storage: &mut dyn Storage,
    sender: Addr,
    denom: String,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    let config = CONFIG.load(storage)?;
    if &sender != config.as_ref() {
        return Err(ExecuteError::Unauthorized { addr: sender }.into());
    }

    let balance =
        BALANCE
            .may_load(storage, denom.clone())?
            .ok_or_else(|| ExecuteError::NotFoundDenom {
                denom: denom.clone(),
            })?;
    let amount = match amount {
        Some(amount) => {
            if balance < amount {
                return Err(ExecuteError::LackOfBalance { maximum: balance }.into());
            }

            amount
        }
        None => balance,
    };
    let coin = vec![Coin::new(amount.u128(), denom)];
    let bank_msg = BankMsg::Send {
        to_address: sender.to_string(),
        amount: coin,
    };

    Ok(Response::default()
        .add_message(bank_msg)
        .add_attribute("method", "try_withdraw")
        .add_attribute("amount", amount))
}

pub fn try_create_mission(
    storage: &mut dyn Storage,
    height: u64,
    sender: Addr,
    mission_item: CreateMissionItem,
) -> ContractResult<Response> {
    let mission: Mission = mission_item.into();
    let mission_id = next_id(storage)?;

    let Mission { coin, title, .. } = mission.clone();

    MISSION.save(storage, (mission_id, sender.clone()), &mission)?;
    RECENTLY_MISSION_LIST.update(storage, height, move |snapshot| -> ContractResult<_> {
        let snapshot = snapshot.unwrap_or_default();
        let mut snapshot: Vec<_> = [mission].into_iter().chain(snapshot.into_iter()).collect();

        if snapshot.len() > RECENTLY_MISSION_LIMIT {
            Ok(snapshot.drain(0..RECENTLY_MISSION_LIMIT).collect())
        } else {
            Ok(snapshot)
        }
    })?;
    BALANCE.update(storage, coin.denom.clone(), |state| -> ContractResult<_> {
        let state = state.unwrap_or_default();
        let state = state
            .checked_add(coin.amount)
            .map_err(ExecuteError::Overflow)?;
        Ok(state)
    })?;
    Ok(Response::default()
        .add_attribute("method", "try_create_mission")
        .add_attribute("sender", sender)
        .add_attribute("title", title)
        .add_attribute("denom", &coin.denom)
        .add_attribute("amount", coin.amount)
        .add_attribute("mission_id", &mission_id.to_string()))
}

pub fn try_complete_mission(
    storage: &mut dyn Storage,
    block_time: BlockTime,
    sender: Addr,
    mission_id: u64,
    postscript: String,
) -> ContractResult<Response> {
    let response = Response::default()
        .add_attribute("method", "try_complete_mission")
        .add_attribute("sender", sender.to_string())
        .add_attribute("mission_id", mission_id.to_string());

    let key = (mission_id, sender.clone());
    let mut mission =
        MISSION
            .may_load(storage, key.clone())?
            .ok_or_else(|| ExecuteError::NotFoundMission {
                sender: sender.clone(),
                index: mission_id,
            })?;

    if mission.is_expired(&block_time)? {
        MISSION.update(storage, key, |_| -> ContractResult<_> { Ok(mission) })?;
        return Ok(response.add_attribute("status", Status::Failed.as_ref()));
    }
    mission.new_status(Status::Success)?;
    MISSION.update(storage, key, |_| -> ContractResult<_> {
        Ok(mission.clone())
    })?;

    let coin = &mission.coin;
    let refund_message = BankMsg::Send {
        to_address: sender.to_string(),
        amount: vec![mission.coin.clone()],
    };

    BALANCE.update(storage, coin.denom.clone(), |state| -> ContractResult<_> {
        let state = state.unwrap_or_default();
        let state = state
            .checked_sub(coin.amount)
            .map_err(ExecuteError::Overflow)?;
        Ok(state)
    })?;
    Ok(response
        .add_message(refund_message)
        .add_attribute("postscript", postscript))
}

fn try_failed_mission(
    storage: &mut dyn Storage,
    sender: Addr,
    mission_id: u64,
) -> ContractResult<Response> {
    let key = (mission_id, sender.clone());
    let mut mission =
        MISSION
            .may_load(storage, key.clone())?
            .ok_or_else(|| ExecuteError::NotFoundMission {
                sender: sender.clone(),
                index: mission_id,
            })?;
    mission.new_status(Status::Failed)?;

    MISSION.update(storage, key, |_| -> ContractResult<_> { Ok(mission) })?;
    Ok(Response::default()
        .add_attribute("method", "try_failed_mission")
        .add_attribute("sender", sender.clone())
        .add_attribute("mission_id", mission_id.to_string()))
}

#[cfg(test)]
mod tests {

    use cosmwasm_std::{testing::MockStorage, Coin, CosmosMsg, Uint128};

    use super::*;

    fn assert_eq_balance(storage: &mut dyn Storage, denom: impl Into<String>, expected: Uint128) {
        let amount = BALANCE.load(storage, denom.into()).unwrap();

        assert_eq!(amount, expected);
    }

    fn create_mission(storage: &mut dyn Storage, sender: Addr) -> Mission {
        let mission_item = CreateMissionItem {
            title: "title".to_owned(),
            coin: Coin::new(2, "token"),
            ends_at: 10,
        };
        super::try_create_mission(storage, 0, sender, mission_item.clone()).unwrap();

        mission_item.into()
    }

    #[test]
    fn try_create_mission() {
        let mut storage = MockStorage::default();

        let title = "title";
        let denom = "token";
        let mission_item = CreateMissionItem {
            title: title.to_owned(),
            coin: Coin::new(2, denom),
            ends_at: 0,
        };

        let sender = "maker";
        let sender_addr = Addr::unchecked(sender);
        let resp = super::try_create_mission(&mut storage, 0, sender_addr, mission_item).unwrap();
        assert_eq!(resp.attributes.len(), 6);
        assert_eq!(resp.messages.len(), 0);

        let assert_eq_attr = |index: usize, key: &str, value: &str| {
            let item = &resp.attributes[index];
            assert_eq!(item.key, key);
            assert_eq!(item.value, value);
        };

        assert_eq_attr(0, "method", "try_create_mission");
        assert_eq_attr(1, "sender", sender);
        assert_eq_attr(2, "title", title);
        assert_eq_attr(3, "denom", denom);
        assert_eq_attr(4, "amount", "2");
        assert_eq_attr(5, "mission_id", "1");

        assert_eq_balance(&mut storage, "token", Uint128::new(2));
    }

    #[test]
    fn try_complete_mission() {
        let mut storage = MockStorage::default();
        let addr = Addr::unchecked("maker");
        let postscript = "DONE!";

        let _ = create_mission(&mut storage, addr.clone());
        let mission_id = 1;

        let block_time = BlockTime { height: 0, time: 0 };

        let resp = super::try_complete_mission(
            &mut storage,
            block_time,
            addr.clone(),
            mission_id,
            postscript.to_owned(),
        )
        .unwrap();
        assert_eq!(resp.attributes.len(), 4);
        assert_eq!(resp.messages.len(), 1);

        let assert_eq_attr = |index: usize, key: &str, value: &str| {
            let item = &resp.attributes[index];
            assert_eq!(item.key, key);
            assert_eq!(item.value, value);
        };

        assert_eq_attr(0, "method", "try_complete_mission");
        assert_eq_attr(1, "sender", addr.as_ref());
        assert_eq_attr(2, "mission_id", &mission_id.to_string());
        assert_eq_attr(3, "postscript", postscript);

        let assert_eq_bank_msg = |index: usize, address: &str, coin: Coin| {
            let bank_msg = match &resp.messages[index].msg {
                CosmosMsg::Bank(bank_msg) => bank_msg,
                _ => unreachable!(),
            };

            match bank_msg {
                BankMsg::Send { to_address, amount } => {
                    assert_eq!(to_address, address);
                    assert_eq!(amount, &[coin]);
                }
                _ => unreachable!(),
            };
        };

        assert_eq_bank_msg(0, addr.as_ref(), Coin::new(2, "token"));

        assert_eq_balance(&mut storage, "token", Uint128::new(0));
    }

    #[test]
    fn try_complete_mission_expired_case() {
        let mut storage = MockStorage::default();
        let addr = Addr::unchecked("maker");
        let postscript = "DONE!";

        let _ = create_mission(&mut storage, addr.clone());
        let mission_id = 1;

        let block_time = BlockTime {
            height: 20,
            time: 20,
        };

        let resp = super::try_complete_mission(
            &mut storage,
            block_time,
            addr.clone(),
            mission_id,
            postscript.to_owned(),
        )
        .unwrap();
        assert_eq!(resp.attributes.len(), 4);
        assert_eq!(resp.messages.len(), 0);

        let assert_eq_attr = |index: usize, key: &str, value: &str| {
            let item = &resp.attributes[index];
            assert_eq!(item.key, key);
            assert_eq!(item.value, value);
        };

        assert_eq_attr(0, "method", "try_complete_mission");
        assert_eq_attr(1, "sender", addr.as_ref());
        assert_eq_attr(2, "mission_id", &mission_id.to_string());
        assert_eq_attr(3, "status", "failed");

        assert_eq_balance(&mut storage, "token", Uint128::new(2));
    }

    #[test]
    fn try_failed_mission() {
        let mut storage = MockStorage::default();
        let addr = Addr::unchecked("maker");

        let _ = create_mission(&mut storage, addr.clone());
        let mission_id = 1;

        let resp = super::try_failed_mission(&mut storage, addr.clone(), mission_id).unwrap();
        assert_eq!(resp.attributes.len(), 3);
        assert_eq!(resp.messages.len(), 0);

        let assert_eq_attr = |index: usize, key: &str, value: &str| {
            let item = &resp.attributes[index];
            assert_eq!(item.key, key);
            assert_eq!(item.value, value);
        };

        assert_eq_attr(0, "method", "try_failed_mission");
        assert_eq_attr(1, "sender", addr.as_ref());
        assert_eq_attr(2, "mission_id", &mission_id.to_string());

        assert_eq_balance(&mut storage, "token", Uint128::new(2));
    }
}
