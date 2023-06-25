use std::cmp::Ordering;

use cosmwasm_std::{Addr, CosmosMsg, Decimal, DepsMut, Uint128};

use crate::{
    market_logic::liquidity_provider,
    state::{LEVELS_DATA, LEVEL_ORDERS, MARKET_INFO, USER_ORDERS},
    state_utils,
    structs::{CurrencyStatus, LevelOrder, MarketInfo, OrderSide, UserOrderRecord},
    utils::{create_funds_message, wrapped_comparison},
    ContractError,
};

use crate::structs::LevelOrders;

use super::structs::{ConsumedOrdersLevel, LiquidityConsumer};

pub fn process_liquidity_taker(
    deps: DepsMut,
    sender: Addr,
    market_id: u64,
    opt_order_price: Option<Decimal>, // optional, to check if market or limit order
    order_quantity: Uint128,
    order_side: OrderSide,
) -> Result<Vec<CosmosMsg>, ContractError> {
    // we start by setting some comparators
    // this allows us to reuse code for bids and asks
    let (_closer_to_midprice_comparator, further_to_midprice_comparator) = match order_side {
        OrderSide::Sell => (Ordering::Greater, Ordering::Less),
        OrderSide::Buy => (Ordering::Less, Ordering::Greater),
    };

    // access market info
    let mut market_info = MARKET_INFO.load(deps.storage, market_id)?;
    let currency_status = MarketInfo::get_currency_status_from_order_side(order_side.clone());

    //let mut id_prev_level: Option<u64> = None;
    let mut id_current_level = match order_side {
        OrderSide::Buy => market_info.top_level_ask,
        OrderSide::Sell => market_info.top_level_bid,
    };

    let mut to_send_back = Uint128::zero();
    let mut consumed_orders: Vec<ConsumedOrdersLevel> = vec![];
    let mut remaining_quantity: Uint128 = order_quantity;
    loop {
        match id_current_level {
            // no current market, if it is a taker then we are at the end of the list and must put a limit order
            // and update market info
            None => {
                match opt_order_price {
                    None => return Err(ContractError::NotEnoughLiquidityMarketOrder {}),
                    Some(val_order_price) => {
                        // end of the line, need to set a limit order
                        // this means the book has been fully consumed, so must remove top level of market
                        match order_side {
                            OrderSide::Buy => {
                                market_info.top_level_ask = None;
                            }
                            OrderSide::Sell => {
                                market_info.top_level_bid = None;
                            }
                        }

                        // and save
                        MARKET_INFO.save(deps.storage, market_id, &market_info)?;

                        // now insert the new level
                        liquidity_provider::process_limit_maker(
                            deps.storage,
                            sender.clone(),
                            market_id,
                            val_order_price,
                            remaining_quantity,
                            order_side.clone(),
                        )?;

                        // need to add order for user
                        USER_ORDERS.update(
                            deps.storage,
                            sender.clone(),
                            |orders| -> Result<_, ContractError> {
                                let mut orders = orders.unwrap_or_default();
                                orders.push(UserOrderRecord {
                                    market_id: market_id,
                                    order_side: order_side.clone(),
                                    price: opt_order_price.unwrap(),
                                    quantity: remaining_quantity,
                                });
                                return Ok(orders);
                            },
                        )?;
                        break;
                    }
                }
            }
            // there is a following market in the list, compare its price to the order price
            // and stop with insert or continue consuming liquidity
            Some(val_id_current_level) => {
                let curr_level_data = LEVELS_DATA.load(deps.storage, val_id_current_level)?;
                // seperate between market and limit orders
                let is_consume_level = match opt_order_price {
                    None => true,
                    Some(val_order_price) => !wrapped_comparison(
                        curr_level_data.price,
                        val_order_price,
                        further_to_midprice_comparator,
                    ),
                };

                if is_consume_level {
                    // consume the level
                    let mut level_orders = LEVEL_ORDERS.load(deps.storage, val_id_current_level)?;
                    let consumption_result =
                        level_orders.consume(curr_level_data.price, remaining_quantity);

                    match currency_status {
                        CurrencyStatus::QuoteCurrency => {
                            // received base currency in input, so output is quote currency
                            to_send_back += consumption_result.to_send_back;
                        }
                        CurrencyStatus::BaseCurrency => {
                            // need to convert amount
                            to_send_back += consumption_result
                                .to_send_back
                                .checked_mul_ceil(curr_level_data.price)
                                .unwrap();
                        }
                    }

                    if consumption_result.remaining_to_consume.is_zero() {
                        // check if there are orders remaining in the current level to update market info
                        if level_orders.len() == 0 {
                            // if there are none, remove this level
                            match order_side {
                                OrderSide::Buy => {
                                    market_info.top_level_ask = curr_level_data.id_next;
                                }
                                OrderSide::Sell => {
                                    market_info.top_level_bid = curr_level_data.id_next;
                                }
                            }
                            MARKET_INFO.save(deps.storage, market_id, &market_info)?;
                            
                            state_utils::remove_level(
                                deps.storage,
                                market_id,
                                val_id_current_level,
                            )?;
                        } else {
                            // there are remaning orders, update market info top of book with current level
                            match order_side {
                                OrderSide::Buy => {
                                    market_info.top_level_ask = id_current_level;
                                }
                                OrderSide::Sell => {
                                    market_info.top_level_bid = id_current_level;
                                }
                            }
                            MARKET_INFO.save(deps.storage, market_id, &market_info)?;

                            // and save the remaining orders
                            LEVEL_ORDERS.save(deps.storage, val_id_current_level, &level_orders)?;
                        }

                        consumed_orders.push(ConsumedOrdersLevel::from_consumption_result(
                            curr_level_data.price,
                            consumption_result,
                        ));

                        // nothing left in the consuming order, can break out of the consuming logic
                        break;
                    } else {
                        // order has not been fully filled yet, continue in consumption logic
                        // level has been fully consumed, remove it
                        state_utils::remove_level(deps.storage, market_id, val_id_current_level)?;
                        remaining_quantity = consumption_result.remaining_to_consume;

                        consumed_orders.push(ConsumedOrdersLevel::from_consumption_result(
                            curr_level_data.price,
                            consumption_result,
                        ));

                        // set next level to consume
                        id_current_level = curr_level_data.id_next;
                    }
                } else {
                    // we do not consume the next level
                    // this means this is a limit taker, so need to add a level
                    liquidity_provider::process_limit_maker(
                        deps.storage,
                        sender.clone(),
                        market_id,
                        opt_order_price.unwrap(),
                        remaining_quantity,
                        order_side.clone(),
                    )?;

                    // need to add order for user
                    USER_ORDERS.update(
                        deps.storage,
                        sender.clone(),
                        |orders| -> Result<_, ContractError> {
                            let mut orders = orders.unwrap_or_default();
                            orders.push(UserOrderRecord {
                                market_id: market_id,
                                order_side: order_side.clone(),
                                price: opt_order_price.unwrap(),
                                quantity: remaining_quantity,
                            });
                            return Ok(orders);
                        },
                    )?;
                    break;
                }
            }
        }
    }

    // process consumed orders in state of user orders
    let currency_info = market_info.get_currency_info_from_side(order_side.clone());

    let mut messages: Vec<CosmosMsg> = vec![];
    for cons in &consumed_orders {
        for order in &cons.orders {
            USER_ORDERS.update(
                deps.storage,
                order.user.clone(),
                |user_orders| -> Result<_, ContractError> {
                    //let mut user_orders = user_orders.unwrap();

                    let user_orders = user_orders
                        .unwrap()
                        .into_iter()
                        .filter_map(|mut user_order| {
                            if user_order.market_id == market_id && user_order.price == cons.price {
                                user_order.quantity -= order.amount;
                                if user_order.quantity.is_zero() {
                                    None
                                } else {
                                    Some(user_order)
                                }
                            } else {
                                Some(user_order)
                            }
                        })
                        .collect();

                    return Ok(user_orders);
                },
            )?;

            let return_amount = match currency_status {
                CurrencyStatus::QuoteCurrency => {
                    // received base currency in input, so output is quote currency
                    order.amount.checked_mul_ceil(cons.price).unwrap()
                }
                CurrencyStatus::BaseCurrency => {
                    // need to convert amount
                    order.amount
                }
            };

            messages.push(create_funds_message(
                return_amount,
                currency_info.clone(),
                order.user.clone(),
            ));
        }
    }

    // add funds to send back to trader
    let trader_currency_info = market_info.get_currency_info_from_side(match order_side {
        OrderSide::Buy => OrderSide::Sell,
        OrderSide::Sell => OrderSide::Buy,
    });

    messages.push(create_funds_message(
        to_send_back,
        trader_currency_info,
        sender,
    ));

    return Ok(messages);
}
