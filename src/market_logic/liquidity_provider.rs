use std::cmp::Ordering;

use cosmwasm_std::{Addr, Decimal, Response, Storage, Uint128};

use crate::{
    state::{LEVELS_DATA, LEVEL_ORDERS, MARKET_INFO},
    structs::{LevelData, LevelOrder, OrderSide},
    utils::{create_id_level_no_status, wrapped_comparison},
    ContractError,
};

pub fn process_limit_maker(
    storage: &mut dyn Storage,
    sender: Addr,
    market_id: u64,
    order_price: Decimal,
    order_quantity: Uint128,
    order_side: OrderSide,
) -> Result<Response, ContractError> {
    // we start by setting some comparators
    // this allows us to reuse code for bids and asks
    let (closer_to_midprice_comparator, _further_to_midprice_comparator) = match order_side {
        OrderSide::Buy => (Ordering::Greater, Ordering::Less),
        OrderSide::Sell => (Ordering::Less, Ordering::Greater),
    };

    // access market info
    let mut market_info = MARKET_INFO.load(storage, market_id)?;

    let mut id_prev_level: Option<u64> = None;
    let mut id_current_level = match order_side {
        OrderSide::Buy => market_info.top_level_bid,
        OrderSide::Sell => market_info.top_level_ask,
    };

    // now create infinite loop, we walk the linked list until we find the spot where to add
    loop {
        match id_current_level {
            // no current market which means we have reached end of the linked list
            None => {
                match id_prev_level {
                    // append to previous if there is one
                    // technically this should have been loaded previously, so may avoid re-reading from storage to save on gas
                    Some(val_id_prev_level) => {
                        let id = create_id_level_no_status(&market_info, order_price);

                        // for dev, we'll load again
                        //let prev_market_data = LEVELS_DATA.load(deps.storage, val_id_prev_level)?;

                        LEVELS_DATA.update(
                            storage,
                            val_id_prev_level,
                            |prev_market_data| -> Result<_, ContractError> {
                                let mut prev_market_data = prev_market_data.unwrap();
                                prev_market_data.id_next = Some(id);

                                return Ok(prev_market_data);
                            },
                        )?;

                        let level_data = LevelData {
                            id_next: None,
                            id_previous: id_prev_level,
                            price: order_price,
                        };

                        let level_orders = vec![LevelOrder {
                            user: sender.clone(),
                            amount: order_quantity,
                        }];

                        LEVELS_DATA.save(storage, id, &level_data)?;
                        LEVEL_ORDERS.save(storage, id, &level_orders)?;

                        break;
                    }
                    // else this is a special case, this is a market with an empty side since no current and no previous
                    // so need to set it in market info directly, it's top of book
                    None => {
                        let id = create_id_level_no_status(&market_info, order_price);

                        let level_data = LevelData {
                            id_next: None,
                            id_previous: None,
                            price: order_price,
                        };

                        let level_orders = vec![LevelOrder {
                            user: sender.clone(),
                            amount: order_quantity,
                        }];

                        match order_side {
                            OrderSide::Buy => market_info.top_level_bid = Some(id),
                            OrderSide::Sell => market_info.top_level_ask = Some(id),
                        }
                        //market_info.top_level_bid = Some(id);

                        LEVELS_DATA.save(storage, id, &level_data)?;
                        LEVEL_ORDERS.save(storage, id, &level_orders)?;

                        MARKET_INFO.save(storage, market_id, &market_info)?;

                        break;
                    }
                }
            }
            // there is a following market in the list, compare its price to the order price
            // and insert or continue going down the list
            Some(val_id_current_level) => {
                // load data current level
                let curr_level_data = LEVELS_DATA.load(storage, val_id_current_level)?;

                // now we check where our order price stands compared to the current level
                // if it's closer to the midprice than the level price, this means we need to insert a level
                // else if it's equal, add order to list of orders at this level
                // else, continue going down the list
                if wrapped_comparison(
                    order_price,
                    curr_level_data.price,
                    closer_to_midprice_comparator,
                ) {
                    let id = create_id_level_no_status(&market_info, order_price);

                    let level_data = LevelData {
                        id_next: id_current_level,
                        id_previous: id_prev_level,
                        price: order_price,
                    };

                    let level_orders = vec![LevelOrder {
                        user: sender.clone(),
                        amount: order_quantity,
                    }];

                    LEVELS_DATA.save(storage, id, &level_data)?;
                    LEVEL_ORDERS.save(storage, id, &level_orders)?;

                    // closer to midprice than current level price, so insert between current level and previous
                    LEVELS_DATA.update(
                        storage,
                        val_id_current_level,
                        |elem| -> Result<_, ContractError> {
                            let mut elem = elem.unwrap();
                            elem.id_previous = Some(id);
                            return Ok(elem);
                        },
                    )?;

                    // set in previous level if there was a previous
                    // if no previous level, this means this is top of book
                    match id_prev_level {
                        None => {
                            MARKET_INFO.update(
                                storage,
                                market_id,
                                |elem| -> Result<_, ContractError> {
                                    let mut elem = elem.unwrap();
                                    match order_side {
                                        OrderSide::Buy => elem.top_level_bid = Some(id),
                                        OrderSide::Sell => elem.top_level_ask = Some(id),
                                    }

                                    return Ok(elem);
                                },
                            )?;
                        }
                        Some(val_id_prev_level) => {
                            LEVELS_DATA.update(
                                storage,
                                val_id_prev_level,
                                |elem| -> Result<_, ContractError> {
                                    let mut elem = elem.unwrap();
                                    elem.id_next = Some(id);

                                    return Ok(elem);
                                },
                            )?;
                        }
                    }

                    break;
                } else if wrapped_comparison(curr_level_data.price, order_price, Ordering::Equal) {
                    // is a match on price, add to orders
                    LEVEL_ORDERS.update(
                        storage,
                        val_id_current_level,
                        |elem| -> Result<_, ContractError> {
                            let mut elem = elem.unwrap();
                            elem.push(LevelOrder {
                                user: sender.clone(),
                                amount: order_quantity,
                            });

                            return Ok(elem);
                        },
                    )?;

                    break;
                } else {
                    // price is further from midprice than the current level price
                    // continue going down the list
                    id_prev_level = id_current_level;
                    id_current_level = LEVELS_DATA.load(storage, val_id_current_level)?.id_next;
                }
            }
        }
    }

    return Ok(Response::new());
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::mock_dependencies, Addr, Decimal, Uint128};

    use crate::{
        contract_admin_execute::add_market,
        state::MARKET_INFO,
        structs::{CurrencyInfo, OrderSide},
    };

    use super::process_limit_maker;

    mod only_bids {
        use crate::{state::LEVELS_DATA, utils::create_id_level_no_status};

        use super::*;

        /// test to add a single order in an empty book
        #[test]
        fn liq_provider_new_book_single_order() {
            let mut deps = mock_dependencies();

            add_market(
                deps.as_mut(),
                CurrencyInfo::Native {
                    denom: "husd".into(),
                },
                CurrencyInfo::Native {
                    denom: "heur".into(),
                },
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user"),
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Buy,
            )
            .unwrap();
        }

        /// test to add a single order in an empty book, then add another one at the same level
        #[test]
        fn liq_provider_new_book_two_orders() {
            let mut deps = mock_dependencies();

            add_market(
                deps.as_mut(),
                CurrencyInfo::Native {
                    denom: "husd".into(),
                },
                CurrencyInfo::Native {
                    denom: "heur".into(),
                },
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Buy,
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user"),
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Buy,
            )
            .unwrap();
        }

        /// test to add a single bid order in an empty book, then add another one at a different level
        /// with higher price first
        #[test]
        fn liq_provider_new_book_two_orders_two_levels_higher_price_first() {
            let mut deps = mock_dependencies();

            add_market(
                deps.as_mut(),
                CurrencyInfo::Native {
                    denom: "husd".into(),
                },
                CurrencyInfo::Native {
                    denom: "heur".into(),
                },
            )
            .unwrap();

            let top_price = Decimal::from_atomics(Uint128::new(1280), 3).unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                top_price.clone(),
                Uint128::new(100),
                OrderSide::Buy,
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Buy,
            )
            .unwrap();

            // we need to check what the top level is for bids
            let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
            let id_top_level_bid = market_info.top_level_bid.unwrap();

            let level_info = LEVELS_DATA
                .load(deps.as_ref().storage, id_top_level_bid)
                .unwrap();

            assert_eq!(top_price, level_info.price);
        }

        /// test to add a single bid order in an empty book, then add another one at a different level
        /// with lower price first
        #[test]
        fn liq_provider_new_book_two_orders_two_levels_lower_price_first() {
            let mut deps = mock_dependencies();

            add_market(
                deps.as_mut(),
                CurrencyInfo::Native {
                    denom: "husd".into(),
                },
                CurrencyInfo::Native {
                    denom: "heur".into(),
                },
            )
            .unwrap();

            let top_price = Decimal::from_atomics(Uint128::new(1280), 3).unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Buy,
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                top_price.clone(),
                Uint128::new(100),
                OrderSide::Buy,
            )
            .unwrap();

            // we need to check what the top level is for bids
            let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
            let id_top_level_bid = market_info.top_level_bid.unwrap();

            let level_info = LEVELS_DATA
                .load(deps.as_ref().storage, id_top_level_bid)
                .unwrap();

            assert_eq!(top_price, level_info.price);
        }

        /// create 3 levels in the market, with the following price order: mid, high, low
        #[test]
        fn liq_provider_new_book_three_levels_mid_high_low() {
            let mut deps = mock_dependencies();

            add_market(
                deps.as_mut(),
                CurrencyInfo::Native {
                    denom: "husd".into(),
                },
                CurrencyInfo::Native {
                    denom: "heur".into(),
                },
            )
            .unwrap();

            let top_price = Decimal::from_atomics(Uint128::new(1280), 3).unwrap();
            let mid_price = Decimal::one();
            let bottom_price = Decimal::from_atomics(Uint128::new(988), 3).unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                mid_price,
                Uint128::new(100),
                OrderSide::Buy,
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                top_price.clone(),
                Uint128::new(100),
                OrderSide::Buy,
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                bottom_price.clone(),
                Uint128::new(100),
                OrderSide::Buy,
            )
            .unwrap();

            // we need to check what the top level is for bids
            let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();

            let id_top_level_bid = create_id_level_no_status(&market_info, top_price);
            let id_mid_level_bid = create_id_level_no_status(&market_info, mid_price);
            let id_bottom_level_bid = create_id_level_no_status(&market_info, bottom_price);

            assert_eq!(market_info.top_level_bid.unwrap(), id_top_level_bid);

            let top_level_info = LEVELS_DATA
                .load(deps.as_ref().storage, id_top_level_bid)
                .unwrap();
            assert_eq!(top_level_info.id_next.unwrap(), id_mid_level_bid);
            assert!(top_level_info.id_previous.is_none());

            let mid_level_info = LEVELS_DATA
                .load(deps.as_ref().storage, id_mid_level_bid)
                .unwrap();
            assert_eq!(mid_level_info.id_next.unwrap(), id_bottom_level_bid);
            assert_eq!(mid_level_info.id_previous.unwrap(), id_top_level_bid);

            let bottom_level_info = LEVELS_DATA
                .load(deps.as_ref().storage, id_bottom_level_bid)
                .unwrap();
            assert_eq!(bottom_level_info.id_previous.unwrap(), id_mid_level_bid);
            assert!(bottom_level_info.id_next.is_none());
        }
    }

    /// same as only_bids, except this time is sell orders
    mod only_asks {
        use crate::{state::LEVELS_DATA, utils::create_id_level_no_status};

        use super::*;

        /// test to add a single order in an empty book
        #[test]
        fn liq_provider_new_book_single_order() {
            let mut deps = mock_dependencies();

            add_market(
                deps.as_mut(),
                CurrencyInfo::Native {
                    denom: "husd".into(),
                },
                CurrencyInfo::Native {
                    denom: "heur".into(),
                },
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user"),
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Sell,
            )
            .unwrap();
        }

        /// test to add a single order in an empty book, then add another one at the same level
        #[test]
        fn liq_provider_new_book_two_orders() {
            let mut deps = mock_dependencies();

            add_market(
                deps.as_mut(),
                CurrencyInfo::Native {
                    denom: "husd".into(),
                },
                CurrencyInfo::Native {
                    denom: "heur".into(),
                },
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Sell,
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user"),
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Sell,
            )
            .unwrap();
        }

        /// test to add a single bid order in an empty book, then add another one at a different level
        /// with higher price first
        #[test]
        fn liq_provider_new_book_two_orders_two_levels_higher_price_first() {
            let mut deps = mock_dependencies();

            add_market(
                deps.as_mut(),
                CurrencyInfo::Native {
                    denom: "husd".into(),
                },
                CurrencyInfo::Native {
                    denom: "heur".into(),
                },
            )
            .unwrap();

            let top_price = Decimal::from_atomics(Uint128::new(888), 3).unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                top_price.clone(),
                Uint128::new(100),
                OrderSide::Sell,
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Sell,
            )
            .unwrap();

            // we need to check what the top level is for asks
            let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
            let id_top_level_ask = market_info.top_level_ask.unwrap();

            let level_info = LEVELS_DATA
                .load(deps.as_ref().storage, id_top_level_ask)
                .unwrap();

            assert_eq!(top_price, level_info.price);
        }

        /// test to add a single bid order in an empty book, then add another one at a different level
        /// with lower price first
        #[test]
        fn liq_provider_new_book_two_orders_two_levels_lower_price_first() {
            let mut deps = mock_dependencies();

            add_market(
                deps.as_mut(),
                CurrencyInfo::Native {
                    denom: "husd".into(),
                },
                CurrencyInfo::Native {
                    denom: "heur".into(),
                },
            )
            .unwrap();

            let top_price = Decimal::from_atomics(Uint128::new(888), 3).unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Sell,
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                top_price.clone(),
                Uint128::new(100),
                OrderSide::Sell,
            )
            .unwrap();

            // we need to check what the top level is for asks
            let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
            let id_top_level_ask = market_info.top_level_ask.unwrap();

            let level_info = LEVELS_DATA
                .load(deps.as_ref().storage, id_top_level_ask)
                .unwrap();

            assert_eq!(top_price, level_info.price);
        }

        /// create 3 levels in the market, with the following price order: mid, high, low
        #[test]
        fn liq_provider_new_book_three_levels_mid_high_low() {
            let mut deps = mock_dependencies();

            add_market(
                deps.as_mut(),
                CurrencyInfo::Native {
                    denom: "husd".into(),
                },
                CurrencyInfo::Native {
                    denom: "heur".into(),
                },
            )
            .unwrap();

            let bottom_price = Decimal::from_atomics(Uint128::new(1280), 3).unwrap();
            let mid_price = Decimal::one();
            let top_price = Decimal::from_atomics(Uint128::new(988), 3).unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                mid_price,
                Uint128::new(100),
                OrderSide::Sell,
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                top_price.clone(),
                Uint128::new(100),
                OrderSide::Sell,
            )
            .unwrap();

            process_limit_maker(
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                bottom_price.clone(),
                Uint128::new(100),
                OrderSide::Sell,
            )
            .unwrap();

            // we need to check what the top level is for bids
            let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();

            let id_top_level_ask = create_id_level_no_status(&market_info, top_price);
            let id_mid_level_ask = create_id_level_no_status(&market_info, mid_price);
            let id_bottom_level_ask = create_id_level_no_status(&market_info, bottom_price);

            assert_eq!(market_info.top_level_ask.unwrap(), id_top_level_ask);

            let top_level_info = LEVELS_DATA
                .load(deps.as_ref().storage, id_top_level_ask)
                .unwrap();
            assert_eq!(top_level_info.id_next.unwrap(), id_mid_level_ask);
            assert!(top_level_info.id_previous.is_none());

            let mid_level_info = LEVELS_DATA
                .load(deps.as_ref().storage, id_mid_level_ask)
                .unwrap();
            assert_eq!(mid_level_info.id_next.unwrap(), id_bottom_level_ask);
            assert_eq!(mid_level_info.id_previous.unwrap(), id_top_level_ask);

            let bottom_level_info = LEVELS_DATA
                .load(deps.as_ref().storage, id_bottom_level_ask)
                .unwrap();
            assert_eq!(bottom_level_info.id_previous.unwrap(), id_mid_level_ask);
            assert!(bottom_level_info.id_next.is_none());
        }
    }
}
