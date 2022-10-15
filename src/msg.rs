use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, Addr};
use cw20::Denom;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub denom: Denom
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateOwner {
        owner: Addr,
    },
    UpdateEnabled {
        enabled: bool,
    },
    Flip {
        level: u32
    },
    RemoveTreasury {
        amount: Uint128
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    History {
        count: u32
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: Addr,
    pub enabled: bool,
    pub denom: Denom,
    pub treasury_amount: Uint128,
    pub flip_count: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct History {
    pub id: u64,
    pub address: Addr,
    pub level: u32,
    pub rnd_ticket: u64,
    pub win: bool,
    pub reward_amount: Uint128,
    pub timestamp: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HistoryResponse {
    pub list: Vec<History>
}


#[derive(Hash)]
pub struct HashObj {
    pub time: u64,
    pub address: Addr,
    pub level: u32,
    pub flip_count: u64
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
