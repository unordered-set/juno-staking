use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, VecDeque};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[cw_serde]
pub struct StakingInfo {
    pub token: Addr,
    pub rewards_per_day: Uint128
}

pub const DAILY_TOKEN_AMOUNT: VecDeque<Uint128> = VecDeque::new("dta");
pub const STAKING_INFO: Item<StakingInfo> = Item::new("stakingInfo");