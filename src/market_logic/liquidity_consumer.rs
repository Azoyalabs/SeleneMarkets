use cosmwasm_std::Uint128;

use crate::structs::{LevelOrder, LevelOrders};

#[derive(Debug)]
pub struct ConsumptionResult {
    pub is_fully_consumed: bool,
    pub remaining_to_consume: Uint128,
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
    fn consume(&mut self, quantity: Uint128) -> ConsumptionResult;
}

impl LiquidityConsumer for LevelOrders {
    fn consume(&mut self, quantity: Uint128) -> ConsumptionResult {
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

#[cfg(test)]
mod tests {
    use crate::{
        market_logic::liquidity_consumer::LiquidityConsumer,
        structs::{LevelOrder, LevelOrders},
    };
    use cosmwasm_std::{Addr, Uint128};

    /// Consume, single order in book that matches amount to be consumed
    /// so level is fully consumed, nothing left in the incoming order
    #[test]
    fn liquidity_consumer_full_single_no_remaining() {
        let mut level: LevelOrders = vec![LevelOrder {
            user: Addr::unchecked("user"),
            amount: Uint128::new(10000u128),
        }];

        let res = level.consume(Uint128::new(10000u128));
        //println!("{:?}", res);

        assert!(res.is_fully_consumed);
        assert!(res.remaining_to_consume.is_zero());
        assert_eq!(
            res.bin_records_consumed,
            vec![LevelOrder {
                user: Addr::unchecked("user"),
                amount: Uint128::new(10000u128)
            }]
        );
    }

    /// Consume, single order in book that is higher than amount to be consumed
    /// so level is not fully consumed, nothing left in the incoming order
    #[test]
    fn liquidity_consumer_full_single_remainder_in_level() {
        let mut level: LevelOrders = vec![LevelOrder {
            user: Addr::unchecked("user"),
            amount: Uint128::new(12000u128),
        }];

        let res = level.consume(Uint128::new(10000u128));
        //println!("{:?}", res);

        assert!(!res.is_fully_consumed);
        assert!(res.remaining_to_consume.is_zero());
        assert_eq!(
            res.bin_records_consumed,
            vec![LevelOrder {
                user: Addr::unchecked("user"),
                amount: Uint128::new(10000u128)
            }]
        );
    }

    /// Consume, single order in book that is lower than amount to consume
    /// so level should be fully consumed, and there should still be amount remaining in order
    #[test]
    fn liquidity_consumer_full_single_remaining_to_consume() {
        let mut level: LevelOrders = vec![LevelOrder {
            user: Addr::unchecked("user"),
            amount: Uint128::new(10000u128),
        }];

        let res = level.consume(Uint128::new(12000u128));
        //println!("{:?}", res);

        assert!(res.is_fully_consumed);
        assert!(!res.remaining_to_consume.is_zero());
        assert_eq!(
            res.bin_records_consumed,
            vec![LevelOrder {
                user: Addr::unchecked("user"),
                amount: Uint128::new(10000u128)
            }]
        );
    }
}
