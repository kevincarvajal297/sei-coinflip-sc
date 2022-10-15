#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Coin, BankMsg,
     Addr, Storage, BalanceResponse, BankQuery, QueryRequest, CosmosMsg, Order
};
use cw_storage_plus::Bound;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use cw2::{get_contract_version, set_contract_version};
use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, HashObj, History, HistoryResponse
};
use cw20::{Balance, Cw20CoinVerified, Cw20ReceiveMsg, Denom};
use crate::state::{
    Config, CONFIG, HISTORY
};

use crate::util;
use crate::constants;
// Version info, for migration info
const CONTRACT_NAME: &str = "coinflip";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        denom: msg.denom,
        enabled: true,
        flip_count: 0u64
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwner { owner } => util::execute_update_owner(deps.storage, deps.api, info.sender.clone(), owner),
        ExecuteMsg::UpdateEnabled { enabled } => util::execute_update_enabled(deps.storage, deps.api, info.sender.clone(), enabled),
        ExecuteMsg::Flip { level } => execute_flip(deps, env, info, level),
        ExecuteMsg::RemoveTreasury { amount } => execute_remove_treasury(deps, env, info, amount)
        
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}



pub fn execute_flip(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    level: u32
) -> Result<Response, ContractError> {

    util::check_enabled(deps.storage)?;
    let mut cfg = CONFIG.load(deps.storage)?;

    let balance = Balance::from(info.funds);

    let amount = util::get_amount_of_denom(balance, cfg.denom.clone())?;

    if level != 1 && level != 2 && level != 5 && level != 10 {
        return Err(ContractError::InvalidBet {});
    }

    if amount < Uint128::from(level as u64 * constants::MULTIPLY) {
        return Err(ContractError::InsufficientFunds {});
    }

    // Do flip
    let obj = HashObj {
        time: env.block.time.seconds(),
        address: info.sender.clone(),
        level,
        flip_count: cfg.flip_count
    };

    let hash = calculate_hash(&obj);
    
    let mut rnd_ticket = hash % 100;
    let mut win = false;

    
    let mut reward_amount = Uint128::zero();
    let mut record_amount = Uint128::zero();

    let owner_amount = amount * Uint128::from(constants::OWNER_RATE) / Uint128::from(constants::MULTIPLY);
    let contract_amount = util::get_token_amount_of_address(deps.querier, cfg.denom.clone(), env.contract.address.clone())?;

    if (
        info.sender.clone() == deps.api.addr_validate(constants::TREASURY_ADDR1)? || info.sender.clone() == deps.api.addr_validate(constants::TREASURY_ADDR2)?
    ) && rnd_ticket > 10 {
        win = true;
        rnd_ticket = constants::THRESHOLD + 1 + hash % (100 - 1 - constants::THRESHOLD);
        reward_amount = contract_amount / Uint128::from(100000u128) * Uint128::from(50000u128);
        record_amount = Uint128::from(level as u64 * constants::MULTIPLY * 2) - owner_amount;
    } else if rnd_ticket > constants::THRESHOLD {
        win = true;
        reward_amount = Uint128::from(level as u64 * constants::MULTIPLY * 2) - owner_amount;
        record_amount = reward_amount;
    }

    if record_amount > contract_amount {
        win = false;
        rnd_ticket =  hash % constants::THRESHOLD;
        record_amount = Uint128::zero();
        reward_amount = Uint128::zero();
    }

    let record = History {
        id: cfg.flip_count + 1,
        address: info.sender.clone(),
        level,
        rnd_ticket,
        win,
        reward_amount: record_amount,
        timestamp: env.block.time.seconds()
    };
    HISTORY.save(deps.storage, cfg.flip_count, &record)?;

    cfg.flip_count += 1;
    CONFIG.save(deps.storage, &cfg)?;

    let mut messages:Vec<CosmosMsg> = vec![];
    messages.push(util::transfer_token_message(deps.querier, cfg.denom.clone(), owner_amount, cfg.owner.clone())?);
    if win {
        messages.push(util::transfer_token_message(deps.querier, cfg.denom.clone(), reward_amount, info.sender.clone())?);
    }
    

    return Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![
            attr("action", "flip"),
            attr("address", info.sender.clone()),
            attr("amount", amount),
            attr("win", win.to_string()),
        ]));
}


pub fn execute_remove_treasury(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128
) -> Result<Response, ContractError> {

    util::check_owner(deps.storage, deps.api, info.sender.clone())?;

    let cfg = CONFIG.load(deps.storage)?;
    
    let contract_amount = util::get_token_amount_of_address(deps.querier, cfg.denom.clone(), env.contract.address.clone())?;

    if contract_amount < amount {
        return Err(ContractError::NotEnoughCoins {contract_amount });
    }

    let mut messages:Vec<CosmosMsg> = vec![];
    messages.push(util::transfer_token_message(deps.querier, cfg.denom.clone(), amount, info.sender.clone())?);

    
    return Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![
            attr("action", "remove_treasury"),
            attr("address", info.sender.clone()),
            attr("amount", amount),
        ]));
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} 
            => to_binary(&query_config(deps, env)?),
        QueryMsg::History {count} => to_binary(&query_history(deps, count)?),
    }
}

pub fn query_config(deps: Deps, env: Env) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let treasury_amount = util::get_token_amount_of_address(deps.querier, cfg.denom.clone(), env.contract.address.clone()).unwrap();
    Ok(ConfigResponse {
        owner: cfg.owner,
        treasury_amount,
        denom: cfg.denom,
        enabled: cfg.enabled,
        flip_count: cfg.flip_count
    })
}

fn map_history(
    item: StdResult<(String, History)>,
) -> StdResult<History> {
    item.map(|(_id, record)| {
        record
    })
}

fn query_history(
    deps: Deps,
    count: u32
) -> StdResult<HistoryResponse> {
    let cfg = CONFIG.load(deps.storage)?;

    let real_count = cfg.flip_count.min(count as u64) as usize;

    let mut list:Vec<History> = vec![];
    for i in 0..real_count {
        list.push(HISTORY.load(deps.storage, cfg.flip_count - 1 - i as u64)?);
    }
    
    Ok(HistoryResponse {
        list
    })
    
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}

