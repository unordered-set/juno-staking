#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw20::{Cw20ReceiveMsg, Cw20CoinVerified};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, StakingStatus, STAKING_STATUS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:juno-staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
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
    let staking_status = StakingStatus {
        token: msg.token_addr,
        rewards_per_day: msg.rewards_per_day,
        staking_started: _env.block.time,
    };
    STAKING_STATUS.save(deps.storage, &staking_status)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("token", staking_status.token.as_str())
        .add_attribute("rewards_per_day", staking_status.rewards_per_day.to_string())
        .add_attribute("staking_started", staking_status.staking_started.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => execute::increment(deps),
        // ExecuteMsg::Reset { count } => execute::reset(deps, info, count),
        ExecuteMsg::Unstake { count } => execute::unstake(deps, info, count),
        ExecuteMsg::Receive(msg) => execute::receive(deps, _env, info, msg),
    }
}

pub mod execute {
    use std::cmp::max;

    use cosmwasm_std::{Uint128, from_slice, Addr, Timestamp, OverflowError, Storage};
    use cw20::Balance;
    use cw_storage_plus::VecDeque;
    use crate::{msg::ReceiveMsg, state::{DAILY_TOKEN_AMOUNT, StakeChangeEvent, STAKES, StakeInfo}};

    use super::*;

    pub fn increment(deps: DepsMut) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.count += 1;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "increment"))
    }

    // pub fn reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
    //     STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
    //         if info.sender != state.owner {
    //             return Err(ContractError::Unauthorized {});
    //         }
    //         state.count = count;
    //         Ok(state)
    //     })?;
    //     Ok(Response::new().add_attribute("action", "reset"))
    // }

    pub fn stake(deps: DepsMut, _env: Env, info: MessageInfo, balance: Balance, sender: Addr) -> Result<Response, ContractError> {
        match balance {
            Balance::Native(_) => Err(ContractError::WrongCurrency {  }),
            Balance::Cw20(have) => {
                let convert_to_midnight = |t: Timestamp| {
                    let extra = t.nanos() % (24 * 3600 * 1000000000);
                    t.minus_nanos(extra)
                };
                let today = convert_to_midnight(_env.block.time);
                let tomorrow = today.plus_nanos(24 * 3600 * 1000000000);
                let (prev_amount, prev_date) = match DAILY_TOKEN_AMOUNT.len(deps.storage)? {
                    0 => { (Uint128::from(0u64), today) },
                    _ => {
                        let data = DAILY_TOKEN_AMOUNT.back(deps.storage)?.unwrap();
                        (data.new_amount, data.timestamp)
                    }
                };
                if today < prev_date {
                    // We are still modifying the stake for tomorrow. Also, we are sure that
                    // the record exists, as because it is not, we will be having prev_date == today.
                    DAILY_TOKEN_AMOUNT.pop_back(deps.storage)?;
                    DAILY_TOKEN_AMOUNT.push_back(deps.storage, &StakeChangeEvent{
                        timestamp: prev_date,
                        new_amount: prev_amount + have.amount,
                    })?;
                } else {
                    // today >= prev_day -- so we need a new record
                    DAILY_TOKEN_AMOUNT.push_back(deps.storage, &StakeChangeEvent{
                        timestamp: tomorrow,
                        new_amount: have.amount,
                    })?;
                }

                let current_staked_amount = STAKES.load(deps.storage, sender.clone()).unwrap_or(StakeInfo{amount: Uint128::from(0u64), stake_started: today});
                let staked_plus_rewards = compute_rewards(deps.storage, &current_staked_amount, tomorrow).unwrap();
                STAKES.save(deps.storage, sender, &StakeInfo{
                    amount: staked_plus_rewards + have.amount,
                    stake_started: tomorrow,
                }).unwrap();
                Ok(Response::new())
            }
        }
    }

    pub fn compute_rewards(storage: &mut dyn Storage, stake_info: &StakeInfo, now: Timestamp) -> Result<Uint128, ContractError> {
        let mut ts = now;
        let total_events = DAILY_TOKEN_AMOUNT.len(storage)?;
        let staking_status = STAKING_STATUS.load(storage)?;
        let mut total_rewards = Uint128::from(0u64);
        let mut last_stake_changed_event_index = total_events;
        while last_stake_changed_event_index > 0 {
            last_stake_changed_event_index -= 1;
            let last_event = DAILY_TOKEN_AMOUNT.get(storage, last_stake_changed_event_index)?.unwrap();
            let duration_days = Uint128::from((ts.nanos() - max(last_event.timestamp.nanos(), stake_info.stake_started.nanos())) / (1000000000u64 * 3600 * 24));
            let staked_in_this_period_total = last_event.new_amount;
            let rewards = 
                staking_status.rewards_per_day
                    .checked_mul(duration_days)
                    .and_then(|tr: Uint128| { tr.checked_mul(stake_info.amount) })
                    .and_then(|tr: Uint128| { tr.checked_div(staked_in_this_period_total).map_err(|_| { OverflowError::new(cosmwasm_std::OverflowOperation::Add, "1", "1") }) }).unwrap();
            total_rewards = total_rewards.checked_add(rewards).unwrap();
            if last_event.timestamp.nanos() <= stake_info.stake_started.nanos() {
                break
            }
            ts = last_event.timestamp;
        }
        Ok(total_rewards)
    }

    pub fn unstake(deps: DepsMut, info: MessageInfo, count: Uint128) -> Result<Response, ContractError> {
        return Ok(Response::new())
    }

    pub fn receive(deps: DepsMut, _env: Env, info: MessageInfo, wrapper: Cw20ReceiveMsg) -> Result<Response, ContractError> {
        let stakingStatus = STAKING_STATUS.load(deps.storage)?;
        if info.sender != stakingStatus.token {
            return Err(ContractError::WrongCurrency {  });
        }
    
        let msg: ReceiveMsg = from_slice(&wrapper.msg)?;
        let balance = Balance::Cw20(Cw20CoinVerified {
            address: info.sender.clone(),
            amount: wrapper.amount,
        });
        let api = deps.api;
        match msg {
            ReceiveMsg::Stake {} => {
                stake(deps, _env, info, balance, api.addr_validate(&wrapper.sender)?)
            },
            ReceiveMsg::AddToBank {} => Ok(Response::new()),
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query::count(deps)?),
    }
}

pub mod query {
    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

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
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
