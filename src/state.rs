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

pub struct StakeInfo {
    pub stake_started: u32,
    pub amount: Uint128
}

pub const DAILY_TOKEN_AMOUNT: VecDeque<Uint128> = VecDeque::new("dta");
pub const STAKING_STATUS: Item<StakingStatus> = Item::new("stakingStatus");
pub const STAKES: Map<Addr, StakeInfo> = Map::new("stakes");