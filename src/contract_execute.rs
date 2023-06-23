use cosmwasm_std::{from_binary, Addr, Decimal, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::{
    market_logic::{liquidity_provider, liquidity_remover},
    msg::{ExecuteMsg, SeleneCw20Msg},
    state::{LEVELS_DATA, LEVEL_ORDERS, MARKET_INFO, USER_ORDERS},
    structs::{CurrencyInfo, CurrencyStatus, LevelData, LevelOrder, OrderSide, UserOrderRecord},
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
            let selene_msg: SeleneCw20Msg = from_binary(&receive_msg.msg)?;
            let sender = deps.api.addr_validate(&receive_msg.sender)?;
            match selene_msg {
                SeleneCw20Msg::LimitOrder { market_id, price } => execute_limit_order_cw20(
                    deps,
                    sender,
                    info.sender.to_string(),
                    receive_msg.amount,
                    market_id,
                    price,
                ),
                SeleneCw20Msg::MarketOrder { market_id } => panic!("not implemented"),
            }
        }
        ExecuteMsg::LimitOrder {
            market_id,
            price,
            //order_side,
        } => execute_limit_order2(deps, info, market_id, price),
        ExecuteMsg::RemoveLimitOrder { market_id, price } => {
            execute_remove_limit_order(deps, info, market_id, price)
        }

        ExecuteMsg::MarketOrder { market_id } => execute_market_order(deps, info, market_id),

        // shouldn't happen here
        ExecuteMsg::Admin(_) => return Err(ContractError::Never {}),
    }
}

fn execute_remove_limit_order(
    deps: DepsMut,
    info: MessageInfo,
    market_id: u64,
    order_price: Decimal,
) -> Result<Response, ContractError> {
    // check if order exists
    let mut user_orders = USER_ORDERS.load(deps.storage, info.sender.clone())?;
    let target_order_id = user_orders
        .iter()
        .position(|order| order.market_id == market_id && order.price == order_price);

    match target_order_id {
        None => return Err(ContractError::OrderDoesNotExist {}),
        Some(order_id) => {
            let order_data = user_orders[order_id].clone();

            // remove order from user list of orders
            user_orders.swap_remove(order_id);
            USER_ORDERS.save(deps.storage, info.sender.clone(), &user_orders)?;

            // and remove from book
            liquidity_remover::remove_order(
                deps.storage,
                info,
                market_id,
                order_price,
                order_data.quantity,
                order_data.order_side.to_owned(),
            )?;
        }
    }

    return Ok(Response::new());
}

fn execute_limit_order_cw20(
    deps: DepsMut,
    sender: Addr,
    currency: String,
    order_quantity: Uint128,
    market_id: u64,
    order_price: Decimal,
) -> Result<Response, ContractError> {
    // validate funds
    //let order_value = check_only_one_fund(&info)?;
    //let order_quantity = order_value.amount;

    // load market info
    let market_info = match MARKET_INFO.load(deps.storage, market_id) {
        Err(_) => return Err(ContractError::UnknownMarketId { id: market_id }),
        Ok(market_info) => market_info,
    };

    // determine whether this is a base currency or a quote currency
    let currency_status = market_info.get_currency_status(&currency)?;

    // then determine if it's a buy or a sell depending on currency_status
    // and if taker or maker
    match currency_status {
        CurrencyStatus::BaseCurrency => {
            // if we receive BaseCurrency, then it's a sell order
            // so check if it's taker or maker
            match market_info.top_level_bid {
                // no bids registered, so it's a limit maker order
                None => {
                    // no bids registered, so it's a limit maker order
                    USER_ORDERS.update(
                        deps.storage,
                        sender.clone(),
                        |orders| -> Result<_, ContractError> {
                            let mut orders = orders.unwrap_or_default();
                            orders.push(UserOrderRecord {
                                order_side: OrderSide::Sell,
                                price: order_price,
                                market_id: market_id,
                                quantity: order_quantity,
                            });

                            return Ok(orders);
                        },
                    )?;

                    liquidity_provider::process_limit_maker(
                        deps,
                        sender,
                        market_id,
                        order_price,
                        order_quantity,
                        OrderSide::Sell,
                    )?;

                    /*
                    match market_info.top_level_ask {
                        None => {
                            // no asks also, so maker
                            liquidity_provider::process_limit_maker(deps, info, market_id, order_price, order_quantity, OrderSide::Sell)?;
                        },
                        Some(val_id_top_level_ask) => {
                            // there are asks, need to compare prices
                            let top_ask_level_data = LEVELS_DATA.load(deps.storage, val_id_top_level_ask)?;

                        }
                    }
                    */
                }
                // bids registered, compare to price
                Some(val_id_top_level_bid) => {
                    let top_bid_level_data =
                        LEVELS_DATA.load(deps.storage, val_id_top_level_bid)?;
                    if top_bid_level_data.price < order_price {
                        // this is a limit maker
                        USER_ORDERS.update(
                            deps.storage,
                            sender.clone(),
                            |orders| -> Result<_, ContractError> {
                                let mut orders = orders.unwrap_or_default();
                                orders.push(UserOrderRecord {
                                    order_side: OrderSide::Sell,
                                    price: order_price,
                                    market_id: market_id,
                                    quantity: order_quantity,
                                });

                                return Ok(orders);
                            },
                        )?;

                        liquidity_provider::process_limit_maker(
                            deps,
                            sender,
                            market_id,
                            order_price,
                            order_quantity,
                            OrderSide::Sell,
                        )?;
                    } else {
                        // limit taker
                        panic!("Not implemented");
                    }
                }
            }
        }
        // if we receive BaseCurrency, then it's a sell order
        // so check if it's taker or maker
        CurrencyStatus::QuoteCurrency => {
            match market_info.top_level_ask {
                None => {
                    // no bids registered, so it's a limit maker order
                    USER_ORDERS.update(
                        deps.storage,
                        sender.clone(),
                        |orders| -> Result<_, ContractError> {
                            let mut orders = orders.unwrap_or_default();
                            orders.push(UserOrderRecord {
                                order_side: OrderSide::Buy,
                                price: order_price,
                                market_id: market_id,
                                quantity: order_quantity,
                            });

                            return Ok(orders);
                        },
                    )?;

                    liquidity_provider::process_limit_maker(
                        deps,
                        sender,
                        market_id,
                        order_price,
                        order_quantity,
                        OrderSide::Buy,
                    )?;
                }
                // bids registered, compare to price
                Some(val_id_top_level_ask) => {
                    let top_ask_level_data =
                        LEVELS_DATA.load(deps.storage, val_id_top_level_ask)?;
                    if top_ask_level_data.price > order_price {
                        // this is a limit maker
                        USER_ORDERS.update(
                            deps.storage,
                            sender.clone(),
                            |orders| -> Result<_, ContractError> {
                                let mut orders = orders.unwrap_or_default();
                                orders.push(UserOrderRecord {
                                    order_side: OrderSide::Buy,
                                    price: order_price,
                                    market_id: market_id,
                                    quantity: order_quantity,
                                });

                                return Ok(orders);
                            },
                        )?;

                        liquidity_provider::process_limit_maker(
                            deps,
                            sender,
                            market_id,
                            order_price,
                            order_quantity,
                            OrderSide::Buy,
                        )?;
                    } else {
                        // limit taker
                        panic!("Not implemented");
                    }
                }
            }
        }
    }

    return Ok(Response::new());
}

fn execute_limit_order2(
    deps: DepsMut,
    info: MessageInfo,
    market_id: u64,
    order_price: Decimal,
) -> Result<Response, ContractError> {
    // validate funds
    let order_value = check_only_one_fund(&info)?;
    let order_quantity = order_value.amount;

    // load market info
    let market_info = match MARKET_INFO.load(deps.storage, market_id) {
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
                // no bids registered, so it's a limit maker order
                None => {
                    // no bids registered, so it's a limit maker order
                    USER_ORDERS.update(
                        deps.storage,
                        info.sender.clone(),
                        |orders| -> Result<_, ContractError> {
                            let mut orders = orders.unwrap_or_default();
                            orders.push(UserOrderRecord {
                                order_side: OrderSide::Sell,
                                price: order_price,
                                market_id: market_id,
                                quantity: order_quantity,
                            });

                            return Ok(orders);
                        },
                    )?;

                    liquidity_provider::process_limit_maker(
                        deps,
                        info.sender,
                        market_id,
                        order_price,
                        order_quantity,
                        OrderSide::Sell,
                    )?;

                    /*
                    match market_info.top_level_ask {
                        None => {
                            // no asks also, so maker
                            liquidity_provider::process_limit_maker(deps, info, market_id, order_price, order_quantity, OrderSide::Sell)?;
                        },
                        Some(val_id_top_level_ask) => {
                            // there are asks, need to compare prices
                            let top_ask_level_data = LEVELS_DATA.load(deps.storage, val_id_top_level_ask)?;

                        }
                    }
                    */
                }
                // bids registered, compare to price
                Some(val_id_top_level_bid) => {
                    let top_bid_level_data =
                        LEVELS_DATA.load(deps.storage, val_id_top_level_bid)?;
                    if top_bid_level_data.price < order_price {
                        // this is a limit maker
                        USER_ORDERS.update(
                            deps.storage,
                            info.sender.clone(),
                            |orders| -> Result<_, ContractError> {
                                let mut orders = orders.unwrap_or_default();
                                orders.push(UserOrderRecord {
                                    order_side: OrderSide::Sell,
                                    price: order_price,
                                    market_id: market_id,
                                    quantity: order_quantity,
                                });

                                return Ok(orders);
                            },
                        )?;

                        liquidity_provider::process_limit_maker(
                            deps,
                            info.sender,
                            market_id,
                            order_price,
                            order_quantity,
                            OrderSide::Sell,
                        )?;
                    } else {
                        // limit taker
                        panic!("Not implemented");
                    }
                }
            }
        }
        // if we receive BaseCurrency, then it's a sell order
        // so check if it's taker or maker
        CurrencyStatus::QuoteCurrency => {
            match market_info.top_level_ask {
                None => {
                    // no bids registered, so it's a limit maker order
                    USER_ORDERS.update(
                        deps.storage,
                        info.sender.clone(),
                        |orders| -> Result<_, ContractError> {
                            let mut orders = orders.unwrap_or_default();
                            orders.push(UserOrderRecord {
                                order_side: OrderSide::Buy,
                                price: order_price,
                                market_id: market_id,
                                quantity: order_quantity,
                            });

                            return Ok(orders);
                        },
                    )?;

                    liquidity_provider::process_limit_maker(
                        deps,
                        info.sender,
                        market_id,
                        order_price,
                        order_quantity,
                        OrderSide::Buy,
                    )?;
                }
                // bids registered, compare to price
                Some(val_id_top_level_ask) => {
                    let top_ask_level_data =
                        LEVELS_DATA.load(deps.storage, val_id_top_level_ask)?;
                    if top_ask_level_data.price > order_price {
                        // this is a limit maker
                        USER_ORDERS.update(
                            deps.storage,
                            info.sender.clone(),
                            |orders| -> Result<_, ContractError> {
                                let mut orders = orders.unwrap_or_default();
                                orders.push(UserOrderRecord {
                                    order_side: OrderSide::Buy,
                                    price: order_price,
                                    market_id: market_id,
                                    quantity: order_quantity,
                                });

                                return Ok(orders);
                            },
                        )?;

                        liquidity_provider::process_limit_maker(
                            deps,
                            info.sender,
                            market_id,
                            order_price,
                            order_quantity,
                            OrderSide::Buy,
                        )?;
                    } else {
                        // limit taker
                        panic!("Not implemented");
                    }
                }
            }
        }
    }

    return Ok(Response::new());
}

/*
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
*/

fn process_limit_taker(deps: DepsMut, info: MessageInfo, market_id: u64, order_price: Decimal) {}

fn add_maker_order(deps: DepsMut, info: MessageInfo, market_id: u64, order_price: Decimal) {}

fn execute_market_order(
    _deps: DepsMut,
    _info: MessageInfo,
    market_id: u64,
) -> Result<Response, ContractError> {
    return Ok(Response::new());
}
