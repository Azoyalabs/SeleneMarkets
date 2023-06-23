use std::cmp::Ordering;

use cosmwasm_std::{Addr, Decimal, DepsMut, MessageInfo, Response, Uint128};

use crate::{
    state::{LEVELS_DATA, LEVEL_ORDERS, MARKET_INFO},
    state_utils,
    structs::{LevelData, LevelOrder, OrderSide},
    utils::{create_id_level_no_status, wrapped_comparison},
    ContractError,
};

use crate::structs::LevelOrders;

use super::liquidity_provider;

#[derive(Debug)]
pub struct ConsumptionResult {
    /// has the level been fully consumed
    pub is_fully_consumed: bool,
    /// from the initial order quantity, how much is left to consume
    pub remaining_to_consume: Uint128,
    /// records that have been consumed, partially or fully
    pub bin_records_consumed: Vec<LevelOrder>,
}

impl Default for ConsumptionResult {
    fn default() -> Self {
        return ConsumptionResult {
            bin_records_consumed: vec![],
            is_fully_consumed: false,
            remaining_to_consume: Uint128::zero(),
        };
    }
}

impl ConsumptionResult {
    pub fn new(to_consume: Uint128) -> Self {
        return ConsumptionResult {
            bin_records_consumed: vec![],
            is_fully_consumed: false,
            remaining_to_consume: to_consume,
        };
    }
}

pub trait LiquidityConsumer {
    fn consume(&mut self, price: Decimal, quantity: Uint128) -> ConsumptionResult;
}

impl LiquidityConsumer for LevelOrders {
    fn consume(&mut self, price: Decimal, quantity: Uint128) -> ConsumptionResult {
        let mut rslt = ConsumptionResult::new(quantity);
        loop {
            if let Some(mut curr) = self.pop() {
                if rslt.remaining_to_consume > curr.amount {
                    rslt.remaining_to_consume -= curr.amount;
                    rslt.bin_records_consumed.push(curr);
                } else {
                    rslt.bin_records_consumed.push(LevelOrder {
                        user: curr.user.clone(),
                        amount: rslt.remaining_to_consume,
                    });
                    curr.amount -= rslt.remaining_to_consume;
                    rslt.remaining_to_consume = Uint128::zero();

                    if !curr.amount.is_zero() {
                        self.push(curr);
                    }
                    break;
                }
            } else {
                rslt.is_fully_consumed = true;
                break;
            }
        }

        if self.len() == 0 {
            rslt.is_fully_consumed = true;
        }

        return rslt;
    }
}

pub fn process_liquidity_taker(
    deps: DepsMut,
    sender: Addr,
    market_id: u64,
    opt_order_price: Option<Decimal>, // optional, to check if market or limit order
    order_quantity: Uint128,
    order_side: OrderSide,
) -> Result<(), ContractError> {
    // we start by setting some comparators
    // this allows us to reuse code for bids and asks
    let (_closer_to_midprice_comparator, further_to_midprice_comparator) = match order_side {
        OrderSide::Buy => (Ordering::Greater, Ordering::Less),
        OrderSide::Sell => (Ordering::Less, Ordering::Greater),
    };

    // access market info
    let mut market_info = MARKET_INFO.load(deps.storage, market_id)?;

    //let mut id_prev_level: Option<u64> = None;
    let mut id_current_level = match order_side {
        OrderSide::Buy => market_info.top_level_ask,
        OrderSide::Sell => market_info.top_level_bid,
    };

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
                            deps,
                            sender,
                            market_id,
                            val_order_price,
                            remaining_quantity,
                            order_side,
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

                    if consumption_result.remaining_to_consume.is_zero() {
                        // check if there are orders remaining in the current level to update market info
                        if level_orders.len() == 0 {
                            // if there are none, remove this level
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

                        // nothing left in the consuming order, can break out of the consuming logic
                        break;
                    } else {
                        // order has not been fully filled yet, continue in consumption logic
                        // level has been fully consumed, remove it
                        state_utils::remove_level(deps.storage, market_id, val_id_current_level)?;
                        remaining_quantity = consumption_result.remaining_to_consume;

                        // set next level to consume
                        id_current_level = curr_level_data.id_next;
                    }
                } else {
                    // we do not consume the next level
                    // this means this is a limit taker, so need to add a level
                    liquidity_provider::process_limit_maker(
                        deps,
                        sender,
                        market_id,
                        opt_order_price.unwrap(),
                        remaining_quantity,
                        order_side,
                    )?;
                    break;
                }
            }
        }
    }

    return Ok(());
}
