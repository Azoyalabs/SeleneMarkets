use cosmwasm_std::{
    from_binary, Addr, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128,
};

use crate::{
    market_logic::{liquidity_consumer, liquidity_provider, liquidity_remover},
    msg::{ExecuteMsg, SeleneCw20Msg},
    state::{LEVELS_DATA, LEVEL_ORDERS, MARKET_INFO, USER_ORDERS},
    structs::{CurrencyStatus, MarketInfo, OrderSide, UserOrderRecord},
    utils::{check_only_one_fund, create_funds_message, create_id_level_no_status},
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
                SeleneCw20Msg::MarketOrder { market_id } => execute_market_order_cw20(
                    deps,
                    sender,
                    info.sender.to_string(),
                    receive_msg.amount,
                    market_id,
                ),
            }
        }
        ExecuteMsg::LimitOrder {
            market_id,
            price,
            //order_side,
        } => execute_limit_order(deps, info, market_id, price),
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
    let market_info = MARKET_INFO.load(deps.storage, market_id)?;
    let currency_info =
        market_info.get_currency_status_from_price(deps.as_ref().storage, order_price)?;

    // check if order exists
    let mut user_orders = USER_ORDERS.load(deps.storage, info.sender.clone())?;
    let target_order_id = user_orders
        .iter()
        .position(|order| order.market_id == market_id && order.price == order_price);

    let transfer_msg = match target_order_id {
        None => return Err(ContractError::OrderDoesNotExist {}),
        Some(order_id) => {
            let order_data = user_orders[order_id].clone();

            // remove order from user list of orders
            user_orders.swap_remove(order_id);
            USER_ORDERS.save(deps.storage, info.sender.clone(), &user_orders)?;

            // and remove from book
            liquidity_remover::remove_order(
                deps.storage,
                info.clone(),
                market_id,
                order_price,
                order_data.quantity,
                order_data.order_side.to_owned(),
            )?;

            create_funds_message(order_data.quantity, currency_info, info.sender)
        }
    };

    return Ok(Response::new().add_message(transfer_msg));
}

fn execute_limit_order_cw20(
    deps: DepsMut,
    sender: Addr,
    currency: String,
    order_quantity: Uint128,
    market_id: u64,
    order_price: Decimal,
) -> Result<Response, ContractError> {
    // load market info
    let market_info = match MARKET_INFO.load(deps.storage, market_id) {
        Err(_) => return Err(ContractError::UnknownMarketId { id: market_id }),
        Ok(market_info) => market_info,
    };

    // determine whether this is a base currency or a quote currency
    let currency_status = market_info.get_currency_status(&currency)?;

    let order_side = MarketInfo::get_order_side_from_currency_status(currency_status.clone());

    // check if the user already has an order at this price
    if let Ok(user_orders) = USER_ORDERS.load(deps.storage, sender.clone()) {
        match user_orders.into_iter().find(|order| {
            order.market_id == market_id
                && order.price == order_price
                && order.order_side == order_side
        }) {
            None => (),
            Some(_) => {
                // update user orders
                USER_ORDERS.update(
                    deps.storage,
                    sender.clone(),
                    |orders| -> Result<_, ContractError> {
                        let orders = orders
                            .unwrap()
                            .into_iter()
                            .map(|mut order| {
                                if order.market_id == market_id && order.price == order_price {
                                    order.quantity += order_quantity;
                                }

                                order
                            })
                            .collect();

                        return Ok(orders);
                    },
                )?;

                // update level orders
                let level_id = create_id_level_no_status(&market_info, order_price);
                LEVEL_ORDERS.update(
                    deps.storage,
                    level_id,
                    |level_orders| -> Result<_, ContractError> {
                        let level_orders = level_orders
                            .unwrap()
                            .into_iter()
                            .map(|mut order| {
                                if order.user == sender {
                                    order.amount += order_quantity;
                                }

                                order
                            })
                            .collect();

                        return Ok(level_orders);
                    },
                )?;

                return Ok(Response::new());
            }
        }
    }

    let mut out_msgs: Vec<CosmosMsg> = vec![];

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
                        deps.storage,
                        sender,
                        market_id,
                        order_price,
                        order_quantity,
                        OrderSide::Sell,
                    )?;
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
                            deps.storage,
                            sender,
                            market_id,
                            order_price,
                            order_quantity,
                            OrderSide::Sell,
                        )?;
                    } else {
                        // limit taker
                        out_msgs = liquidity_consumer::process_liquidity_taker(
                            deps,
                            sender,
                            market_id,
                            Some(order_price),
                            order_quantity,
                            OrderSide::Sell,
                        )?;
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
                        deps.storage,
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
                            deps.storage,
                            sender,
                            market_id,
                            order_price,
                            order_quantity,
                            OrderSide::Buy,
                        )?;
                    } else {
                        // limit taker
                        out_msgs = liquidity_consumer::process_liquidity_taker(
                            deps,
                            sender,
                            market_id,
                            Some(order_price),
                            order_quantity,
                            OrderSide::Buy,
                        )?;
                    }
                }
            }
        }
    }

    return Ok(Response::new().add_messages(out_msgs));
}

fn execute_limit_order(
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

    let order_side = MarketInfo::get_order_side_from_currency_status(currency_status.clone());

    // check if the user already has an order at this price
    if let Ok(user_orders) = USER_ORDERS.load(deps.storage, info.sender.clone()) {
        match user_orders.into_iter().find(|order| {
            order.market_id == market_id
                && order.price == order_price
                && order.order_side == order_side
        }) {
            None => (),
            Some(_) => {
                // update user orders
                USER_ORDERS.update(
                    deps.storage,
                    info.sender.clone(),
                    |orders| -> Result<_, ContractError> {
                        let orders = orders
                            .unwrap()
                            .into_iter()
                            .map(|mut order| {
                                if order.market_id == market_id && order.price == order_price {
                                    order.quantity += order_quantity;
                                }

                                order
                            })
                            .collect();

                        return Ok(orders);
                    },
                )?;

                // update level orders
                let level_id = create_id_level_no_status(&market_info, order_price);
                LEVEL_ORDERS.update(
                    deps.storage,
                    level_id,
                    |level_orders| -> Result<_, ContractError> {
                        let level_orders = level_orders
                            .unwrap()
                            .into_iter()
                            .map(|mut order| {
                                if order.user == info.sender {
                                    order.amount += order_quantity;
                                }

                                order
                            })
                            .collect();

                        return Ok(level_orders);
                    },
                )?;

                return Ok(Response::new());
            }
        }
    }

    let mut out_msgs: Vec<CosmosMsg> = vec![];

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
                        deps.storage,
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
                            deps.storage,
                            info.sender,
                            market_id,
                            order_price,
                            order_quantity,
                            OrderSide::Sell,
                        )?;
                    } else {
                        // limit taker
                        out_msgs = liquidity_consumer::process_liquidity_taker(
                            deps,
                            info.sender,
                            market_id,
                            Some(order_price),
                            order_quantity,
                            OrderSide::Sell,
                        )?;
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
                        deps.storage,
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
                            deps.storage,
                            info.sender,
                            market_id,
                            order_price,
                            order_quantity,
                            OrderSide::Buy,
                        )?;
                    } else {
                        // limit taker
                        out_msgs = liquidity_consumer::process_liquidity_taker(
                            deps,
                            info.sender,
                            market_id,
                            Some(order_price),
                            order_quantity,
                            OrderSide::Buy,
                        )?;
                    }
                }
            }
        }
    }

    return Ok(Response::new().add_messages(out_msgs));
}

fn execute_market_order(
    deps: DepsMut,
    info: MessageInfo,
    market_id: u64,
) -> Result<Response, ContractError> {
    // validate funds
    let order_value = check_only_one_fund(&info)?;
    let order_quantity = order_value.amount;

    let market_info = MARKET_INFO.load(deps.storage, market_id)?;
    let order_side = market_info.get_order_side_from_currency(&order_value.denom)?;

    let msgs = liquidity_consumer::process_liquidity_taker(
        deps,
        info.sender,
        market_id,
        None,
        order_quantity,
        order_side,
    )?;

    return Ok(Response::new().add_messages(msgs));
}

fn execute_market_order_cw20(
    deps: DepsMut,
    sender: Addr,
    currency: String,
    order_quantity: Uint128,
    market_id: u64,
) -> Result<Response, ContractError> {
    let market_info = MARKET_INFO.load(deps.storage, market_id)?;
    let order_side = market_info.get_order_side_from_currency(&currency)?;

    let msgs = liquidity_consumer::process_liquidity_taker(
        deps,
        sender,
        market_id,
        None,
        order_quantity,
        order_side,
    )?;

    return Ok(Response::new().add_messages(msgs));
}
