use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Timestamp};
use cw_storage_plus::{Item, Map, VecDeque};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[cw_serde]
pub struct StakingStatus {
    pub token: Addr,
    pub rewards_per_day: Uint128,
    pub staking_started: Timestamp,
}

#[cw_serde]
pub struct StakeInfo {
    pub stake_started: Timestamp,
    pub amount: Uint128
}

#[cw_serde]
pub struct StakeChangeEvent {
    pub timestamp: Timestamp,
    pub new_amount: Uint128,
}

pub const DAILY_TOKEN_AMOUNT: VecDeque<StakeChangeEvent> = VecDeque::new("dta");
pub const STAKING_STATUS: Item<StakingStatus> = Item::new("stakingStatus");
pub const STAKES: Map<Addr, StakeInfo> = Map::new("stakes");