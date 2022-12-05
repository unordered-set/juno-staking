use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map, SnapshotMap, Strategy};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[cw_serde]
pub struct Config {
    pub token: Addr,
    pub tokens_per_day: Uint128
}