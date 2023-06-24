use std::ops::Mul;

use cosmwasm_std::{Decimal, Uint128};

use crate::structs::{LevelOrder, OrderSide};

use crate::structs::LevelOrders;

#[derive(Debug)]
pub struct ConsumptionResult {
    /// has the level been fully consumed
    pub is_fully_consumed: bool,
    /// from the initial order quantity, how much is left to consume
    pub remaining_to_consume: Uint128, //Decimal,
    /// records that have been consumed, partially or fully
    pub bin_records_consumed: Vec<LevelOrder>,
}

impl ConsumptionResult {
    pub fn new(to_consume: Uint128) -> Self {
        return ConsumptionResult {
            bin_records_consumed: vec![],
            is_fully_consumed: false,
            remaining_to_consume: to_consume, //Decimal::new(to_consume),
        };
    }
}

pub struct ConsumedOrdersLevel {
    pub price: Decimal,
    pub orders: Vec<LevelOrder>,
}

impl ConsumedOrdersLevel {
    pub fn from_consumption_result(price: Decimal, rslt: ConsumptionResult) -> Self {
        return ConsumedOrdersLevel {
            price: price,
            orders: rslt.bin_records_consumed,
        };
    }
}

pub trait LiquidityConsumer {
    fn consume(
        &mut self,
        price: Decimal,
        quantity: Uint128,
        //order_side: OrderSide,
    ) -> ConsumptionResult;
}

impl LiquidityConsumer for LevelOrders {
    fn consume(
        &mut self,
        price: Decimal,
        quantity: Uint128,
        //order_side: OrderSide,
    ) -> ConsumptionResult {
        /*
        let qtt_needed = match order_side {
            OrderSide::Buy => quantity.checked_div_floor(price).unwrap(),
            OrderSide::Sell => quantity.checked_div_floor(price).unwrap(),
        };
        */

        let mut rslt = ConsumptionResult::new(quantity.checked_div_floor(price).unwrap());
        println!("initial rslt: {:?}", rslt);
        println!("initial qtt: {:?}", quantity);
        println!("price: {}", price);
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
                    curr.amount -= curr.amount;
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

        /*
        // need to adjust quantity remaining depending on price ?
        match order_side {
            OrderSide::Sell => {
                let qtt_needed = rslt.remaining_to_consume.checked_mul_floor(price).unwrap();
                rslt.remaining_to_consume = qtt_needed;
            }
            OrderSide::Buy => {
                let qtt_needed = rslt.remaining_to_consume.checked_mul_floor(price).unwrap();
                rslt.remaining_to_consume = qtt_needed;
            }
        }
        */

        rslt.remaining_to_consume = rslt.remaining_to_consume.checked_mul_floor(price).unwrap();

        return rslt;
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Decimal, Uint128};

    use crate::{
        market_logic::liquidity_consumer::structs::LiquidityConsumer,
        structs::{LevelOrder, OrderSide},
    };

    mod consume_sells {
        use super::*;

        #[test]
        fn consumer_sell_order_price_0_5_no_remainder() {
            let user_1 = Addr::unchecked("user1");
            let user_2 = Addr::unchecked("user2");

            let mut level_orders = vec![
                LevelOrder {
                    user: user_1,
                    amount: Uint128::new(1),
                },
                LevelOrder {
                    user: user_2,
                    amount: Uint128::new(1),
                },
            ];

            let price = Decimal::from_atomics(5u128, 1).unwrap();
            println!("{}", price);

            let order_side = OrderSide::Sell;

            // so levels are bids, we have a seller here
            // so to clear it all, need to sell 1
            // would get 2 in return?

            let rslt = level_orders.consume(price, Uint128::one()); //, order_side);
            println!("rslt consume: {:?}", rslt);

            assert!(rslt.is_fully_consumed);
            assert_eq!(level_orders.len(), 0);
            assert!(rslt.remaining_to_consume.is_zero());
        }

        #[test]
        fn consumer_sell_order_price_0_5_with_remainder() {
            let user_1 = Addr::unchecked("user1");
            let user_2 = Addr::unchecked("user2");

            let mut level_orders = vec![
                LevelOrder {
                    user: user_1,
                    amount: Uint128::new(1),
                },
                LevelOrder {
                    user: user_2,
                    amount: Uint128::new(1),
                },
            ];

            let price = Decimal::from_atomics(5u128, 1).unwrap();

            let order_side = OrderSide::Sell;

            // so levels are bids, we have a seller here
            // so to clear it all, need to sell 1
            // would get 2 in return?
            // with input quantity of 2, means remained should be 1
            let rslt = level_orders.consume(price, Uint128::new(2)); //, order_side);
            println!("rslt consume: {:?}", rslt);

            assert!(rslt.is_fully_consumed);
            assert_eq!(level_orders.len(), 0);

            assert_eq!(rslt.remaining_to_consume, Uint128::one());
        }

        #[test]
        fn consumer_sell_order_price_1_no_remainder() {
            let user_1 = Addr::unchecked("user1");
            let user_2 = Addr::unchecked("user2");
            let user_3 = Addr::unchecked("user3");

            let mut level_orders = vec![
                LevelOrder {
                    user: user_1,
                    amount: Uint128::new(1),
                },
                LevelOrder {
                    user: user_2,
                    amount: Uint128::new(1),
                },
            ];

            let price = Decimal::from_atomics(1u128, 0).unwrap();
            let order_side = OrderSide::Sell;

            // so levels are bids, we have a seller here
            // so to clear it all, need to sell 2
            // would get 2 in return?
            let rslt = level_orders.consume(price, Uint128::new(2)); //, order_side);
            assert!(rslt.is_fully_consumed);
            assert_eq!(level_orders.len(), 0);
            assert!(rslt.remaining_to_consume.is_zero());
        }
    }

    mod consume_buys {
        use super::*;

        #[test]
        fn consumer_buy_order_price_0_5_no_remainder() {
            let user_1 = Addr::unchecked("user1");
            let user_2 = Addr::unchecked("user2");

            let mut level_orders = vec![
                LevelOrder {
                    user: user_1,
                    amount: Uint128::new(1),
                },
                LevelOrder {
                    user: user_2,
                    amount: Uint128::new(1),
                },
            ];

            let price = Decimal::from_atomics(5u128, 1).unwrap();
            println!("{}", price);

            let order_side = OrderSide::Buy;

            // so levels are asks, we have a buyer here
            // so to clear it all, need to sell 1
            // would get 2 in return?

            let rslt = level_orders.consume(price, Uint128::one()); //, order_side);
            println!("rslt consume: {:?}", rslt);

            assert!(rslt.is_fully_consumed);
            assert_eq!(level_orders.len(), 0);
            assert!(rslt.remaining_to_consume.is_zero());
        }

        #[test]
        fn consumer_buy_order_price_0_5_with_remainder() {
            let user_1 = Addr::unchecked("user1");
            let user_2 = Addr::unchecked("user2");

            let mut level_orders = vec![
                LevelOrder {
                    user: user_1,
                    amount: Uint128::new(1),
                },
                LevelOrder {
                    user: user_2,
                    amount: Uint128::new(1),
                },
            ];

            let price = Decimal::from_atomics(5u128, 1).unwrap();

            let order_side = OrderSide::Buy;

            // so levels are asks, we have a buyer here
            // so to clear it all, need to sell 1
            // would get 2 in return?
            // with input quantity of 2, means remained should be 1
            let rslt = level_orders.consume(price, Uint128::new(2)); //, order_side);
            println!("rslt consume: {:?}", rslt);

            assert!(rslt.is_fully_consumed);
            assert_eq!(level_orders.len(), 0);

            assert_eq!(rslt.remaining_to_consume, Uint128::one());
        }

        #[test]
        fn consumer_buy_order_price_1_no_remainder() {
            let user_1 = Addr::unchecked("user1");
            let user_2 = Addr::unchecked("user2");

            let mut level_orders = vec![
                LevelOrder {
                    user: user_1,
                    amount: Uint128::new(1),
                },
                LevelOrder {
                    user: user_2,
                    amount: Uint128::new(1),
                },
            ];

            let price = Decimal::from_atomics(1u128, 0).unwrap();
            let order_side = OrderSide::Buy;

            // so levels are asks, we have a buyer here
            // so to clear it all, need to sell 2
            // would get 2 in return?
            let rslt = level_orders.consume(price, Uint128::new(2)); //, order_side);
            assert!(rslt.is_fully_consumed);
            assert_eq!(level_orders.len(), 0);
            assert!(rslt.remaining_to_consume.is_zero());
        }
    }
}
