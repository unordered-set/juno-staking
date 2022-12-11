use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::{Cw20ReceiveMsg, Denom};

#[cw_serde]
pub struct InstantiateMsg {
    pub count: i32,
    pub token_addr: Addr,
    pub rewards_per_day: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Increment {},
    Reset { count: i32 },
    Unstake { count: Uint128 },

    /// This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
}

#[cw_serde]
pub enum ReceiveMsg {
    Stake {},
    AddToBank {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetCountResponse)]
    GetCount {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetCountResponse {
    pub count: i32,
}


