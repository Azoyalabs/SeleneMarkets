use std::cmp::Ordering;

use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::{
    msg::ExecuteMsg,
    state::{LEVELS_DATA, LEVEL_ORDERS, MARKET_INFO},
    structs::{CurrencyStatus, LevelData, LevelOrder, OrderSide},
    utils::{check_only_one_fund, create_id_level, create_id_level_no_status},
    ContractError,
};

pub fn route_execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(receive_msg) => {
            panic!("not implemented");
        }
        ExecuteMsg::LimitOrder {
            market_id,
            price,
            order_side,
        } => execute_limit_order(deps, info, market_id, price),
        ExecuteMsg::MarketOrder { market_id } => execute_market_order(deps, info, market_id),

        // shouldn't happen here
        ExecuteMsg::Admin(_) => return Err(ContractError::Never {}),
    }
}

/// Add a limit order to the order book, or consume liquidity
fn execute_limit_order(
    deps: DepsMut,
    info: MessageInfo,
    market_id: u64,
    order_price: Decimal,
) -> Result<Response, ContractError> {
    // validate funds
    let order_value = check_only_one_fund(&info)?;

    // load market info
    let mut market_info = match MARKET_INFO.load(deps.storage, market_id) {
        Err(_) => return Err(ContractError::UnknownMarketId { id: market_id }),
        Ok(market_info) => market_info,
    };

    // determine whether this is a base currency or a quote currency
    let currency_status = market_info.get_currency_status(&order_value.denom)?;

    // then determine if it's a buy or a sell depending on currency_status
    // and if taker or maker
    match currency_status {
        CurrencyStatus::BaseCurrency => {
            // if we receive BaseCurrency, then it's a sell order
            // so check if it's taker or maker
            match market_info.top_level_bid {
                None => {
                    // no bids registered, so this is a maker
                    match market_info.top_level_ask {
                        None => {
                            // no previous level, so add one
                            let level_data = LevelData {
                                id_previous: None,
                                id_next: None,
                                price: order_price,
                            };

                            let id = create_id_level(&market_info, order_price, currency_status);
                            LEVELS_DATA.save(deps.storage, id, &level_data)?;

                            market_info.top_level_ask = Some(id);

                            let level_order = LevelOrder {
                                user: info.sender,
                                amount: order_value.amount,
                            };
                            LEVEL_ORDERS.save(deps.storage, id, &vec![level_order])?;
                            MARKET_INFO.save(deps.storage, market_id, &market_info)?;
                        }
                        Some(level_data) => {}
                    }
                }
                Some(level) => {
                    let level_data = LEVELS_DATA.load(deps.storage, level)?;
                    if level_data.price > order_price {
                        // price is above bids, so it's a maker
                    } else {
                        // price is below top bids, so it's taker
                    }
                }
            }
        }
        CurrencyStatus::QuoteCurrency => {
            // if we receive the quote currency, it's a buy
        }
    }

    return Ok(Response::new());
}

fn process_limit_maker(
    deps: DepsMut,
    info: MessageInfo,
    market_id: u64,
    order_price: Decimal,
    order_quantity: Uint128,
    order_side: OrderSide,
) -> Result<Response, ContractError> {
    // need to find level at which to put it
    // for now we'll just read top on selected side
    // should be done on higher level but whatev, is dev time

    // load market info
    let mut market_info = match MARKET_INFO.load(deps.storage, market_id) {
        Err(_) => return Err(ContractError::UnknownMarketId { id: market_id }),
        Ok(market_info) => market_info,
    };

    match order_side {
        OrderSide::Buy => match market_info.top_level_bid {
            None => {
                let level_data = LevelData {
                    id_next: None,
                    id_previous: None,
                    price: order_price,
                };

                let level_orders = vec![LevelOrder {
                    user: info.sender,
                    amount: order_quantity,
                }];

                let id = create_id_level_no_status(&market_info, order_price);
                market_info.top_level_bid = Some(id);

                LEVELS_DATA.save(deps.storage, id, &level_data)?;
                LEVEL_ORDERS.save(deps.storage, id, &level_orders)?;

                MARKET_INFO.save(deps.storage, market_id, &market_info)?;
            }
            Some(top_level_id) => {
                let mut top_level_data = LEVELS_DATA.load(deps.storage, top_level_id)?;

                if top_level_data.price < order_price {
                    // is new top level
                    let id = create_id_level_no_status(&market_info, order_price);
                    market_info.top_level_bid = Some(id);

                    let level_data = LevelData {
                        id_next: None,
                        id_previous: None,
                        price: order_price,
                    };

                    let level_orders = vec![LevelOrder {
                        user: info.sender,
                        amount: order_quantity,
                    }];

                    top_level_data.id_previous = Some(id);

                    LEVELS_DATA.save(deps.storage, id, &level_data)?;
                    LEVEL_ORDERS.save(deps.storage, id, &level_orders)?;

                    MARKET_INFO.save(deps.storage, market_id, &market_info)?;
                } else if top_level_data.price == order_price {
                    LEVEL_ORDERS.update(
                        deps.storage,
                        top_level_id,
                        |level_orders| -> Result<_, ContractError> {
                            let mut level_orders = level_orders.unwrap();
                            level_orders.push(LevelOrder {
                                user: info.sender,
                                amount: order_quantity,
                            });

                            return Ok(level_orders);
                        },
                    )?;
                } else {
                    // should really put everything in a loop, here I'm repeating code
                    match top_level_data.id_next {
                        None => {
                            // none after, so can set new level
                            let id = create_id_level_no_status(&market_info, order_price);

                            let level_data = LevelData {
                                id_next: None,
                                id_previous: None,
                                price: order_price,
                            };

                            let level_orders = vec![LevelOrder {
                                user: info.sender,
                                amount: order_quantity,
                            }];

                            //market_info.top_level_bid = Some(id);

                            LEVELS_DATA.save(deps.storage, id, &level_data)?;
                            LEVEL_ORDERS.save(deps.storage, id, &level_orders)?;

                            // and update top level
                            top_level_data.id_next = Some(id);
                            LEVELS_DATA.save(deps.storage, top_level_id, &level_data)?;

                            MARKET_INFO.save(deps.storage, market_id, &market_info)?;
                        }
                        Some(id_next) => {
                            // there is a next level, compare to it
                            let curr_level_data = LEVELS_DATA.load(deps.storage, id_next)?;

                            //if curr_level_data.price
                        }
                    }

                    let mut curr_level_data = top_level_data;
                    let mut curr_level_id = top_level_id;

                    loop {}
                }
            }
        },
        OrderSide::Sell => {}
    }

    return Ok(Response::new());
}

fn process_limit_taker(deps: DepsMut, info: MessageInfo, market_id: u64, order_price: Decimal) {}

fn add_maker_order(deps: DepsMut, info: MessageInfo, market_id: u64, order_price: Decimal) {}

fn execute_market_order(
    _deps: DepsMut,
    _info: MessageInfo,
    market_id: u64,
) -> Result<Response, ContractError> {
    return Ok(Response::new());
}

fn sample_execute(_deps: DepsMut, _info: MessageInfo) -> Result<Response, ContractError> {
    return Ok(Response::new());
}
