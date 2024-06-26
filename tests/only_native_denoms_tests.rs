#![allow(unused)]

mod common;

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
    use cw_multi_test::{App, Executor};
    use selene_markets::{
        msg::{
            ExecuteMsg, GetMarketBookResponse, GetMarketsResponse, GetUserAsksResponse,
            GetUserBidsResponse, QueryMsg,
        },
        structs::OrderSide,
    };

    use crate::common::test_utils::{
        create_market_native_only_pair, instantiate_selene, CashMachine, NATIVE_DENOM_1,
        NATIVE_DENOM_2, TEST_USER_1,
    };

    #[test]
    fn only_natives_set_limit_order_bid() {
        let (mut router, market_addr) = instantiate_selene();
        create_market_native_only_pair(&mut router, market_addr.clone());

        let user_1 = Addr::unchecked(TEST_USER_1);

        // mint some native denom of base currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_2.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_1, amount_order.clone());

        // send the order
        let msg = ExecuteMsg::LimitOrder {
            market_id: 0,
            price: Decimal::one(),
            //order_side: OrderSide::Buy,
        };

        let _res = router
            .execute_contract(
                user_1.clone(),
                market_addr.clone(),
                &msg,
                &[amount_order.clone()],
            )
            .unwrap();

        // query orders
        let msg = QueryMsg::GetUserBids {
            user_address: user_1.clone(),
            target_market: Some(0),
        };
        let res: GetUserBidsResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.orders.len(), 1);

        let msg = QueryMsg::GetUserAsks {
            user_address: user_1,
            target_market: Some(0),
        };
        let res: GetUserAsksResponse = router.wrap().query_wasm_smart(market_addr, &msg).unwrap();
        assert_eq!(res.orders.len(), 0);
    }

    #[test]
    fn only_natives_update_limit_order_bid() {
        let (mut router, market_addr) = instantiate_selene();
        create_market_native_only_pair(&mut router, market_addr.clone());

        let user_1 = Addr::unchecked(TEST_USER_1);

        // mint some native denom of base currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_2.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_1, amount_order.clone());

        // send the order
        let msg = ExecuteMsg::LimitOrder {
            market_id: 0,
            price: Decimal::one(),
            //order_side: OrderSide::Buy,
        };

        let _res = router
            .execute_contract(
                user_1.clone(),
                market_addr.clone(),
                &msg,
                &[amount_order.clone()],
            )
            .unwrap();

        // mint some native denom of base currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_2.into(),
            amount: Uint128::new(10000),
        };

        // query orders
        let msg = QueryMsg::GetUserBids {
            user_address: user_1.clone(),
            target_market: Some(0),
        };
        let res: GetUserBidsResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.orders.len(), 1);

        // mint more
        router.mint_native(&user_1, amount_order.clone());

        // send the order
        let msg = ExecuteMsg::LimitOrder {
            market_id: 0,
            price: Decimal::one(),
            //order_side: OrderSide::Buy,
        };

        let _res = router
            .execute_contract(
                user_1.clone(),
                market_addr.clone(),
                &msg,
                &[amount_order.clone()],
            )
            .unwrap();

        // query orders again
        let msg = QueryMsg::GetUserBids {
            user_address: user_1.clone(),
            target_market: Some(0),
        };
        let res_after_update: GetUserBidsResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res_after_update.orders.len(), 1);

        assert_eq!(res_after_update.orders[0].quantity, Uint128::new(20000));
    }

    #[test]
    fn only_natives_set_limit_order_ask() {
        let (mut router, market_addr) = instantiate_selene();
        create_market_native_only_pair(&mut router, market_addr.clone());

        let user_1 = Addr::unchecked(TEST_USER_1);

        // mint some native denom of base currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_1.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_1, amount_order.clone());

        // send the order
        let msg = ExecuteMsg::LimitOrder {
            market_id: 0,
            price: Decimal::one(),
            //order_side: OrderSide::Buy,
        };

        let _res = router
            .execute_contract(
                user_1.clone(),
                market_addr.clone(),
                &msg,
                &[amount_order.clone()],
            )
            .unwrap();

        // query orders
        let msg = QueryMsg::GetUserBids {
            user_address: user_1.clone(),
            target_market: Some(0),
        };
        let res: GetUserBidsResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.orders.len(), 0);

        let msg = QueryMsg::GetUserAsks {
            user_address: user_1,
            target_market: Some(0),
        };
        let res: GetUserAsksResponse = router.wrap().query_wasm_smart(market_addr, &msg).unwrap();
        assert_eq!(res.orders.len(), 1);
    }

    #[test]
    fn only_natives_set_limit_order_and_cancel_bid() {
        let (mut router, market_addr) = instantiate_selene();
        create_market_native_only_pair(&mut router, market_addr.clone());

        let user_1 = Addr::unchecked(TEST_USER_1);

        // mint some native denom of base currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_2.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_1, amount_order.clone());

        // send the order
        let order_price = Decimal::one();

        let msg = ExecuteMsg::LimitOrder {
            market_id: 0,
            price: order_price.clone(),
            //order_side: OrderSide::Buy,
        };

        let _res = router
            .execute_contract(
                user_1.clone(),
                market_addr.clone(),
                &msg,
                &[amount_order.clone()],
            )
            .unwrap();

        // query orders
        let msg = QueryMsg::GetUserBids {
            user_address: user_1.clone(),
            target_market: Some(0),
        };
        let res: GetUserBidsResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.orders.len(), 1);

        let msg = QueryMsg::GetUserAsks {
            user_address: user_1.clone(),
            target_market: Some(0),
        };
        let res: GetUserAsksResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.orders.len(), 0);

        // check from get market book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };
        let res: GetMarketBookResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.bids.len(), 1);
        assert_eq!(res.asks.len(), 0);

        // check user balance
        let balance_pre_cancel = router
            .wrap()
            .query_balance(user_1.clone(), NATIVE_DENOM_2)
            .unwrap();
        assert!(balance_pre_cancel.amount.is_zero());

        // cancel the order
        let msg = ExecuteMsg::RemoveLimitOrder {
            market_id: 0,
            price: order_price,
        };
        let res = router
            .execute_contract(user_1.clone(), market_addr.clone(), &msg, &[])
            .unwrap();

        // check no orders remaining
        let msg = QueryMsg::GetUserBids {
            user_address: user_1.clone(),
            target_market: Some(0),
        };
        let res: GetUserBidsResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.orders.len(), 0);

        let msg = QueryMsg::GetUserAsks {
            user_address: user_1.clone(),
            target_market: Some(0),
        };
        let res: GetUserAsksResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.orders.len(), 0);

        // check from get market book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };
        let res: GetMarketBookResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.bids.len(), 0);
        assert_eq!(res.asks.len(), 0);

        // check user balance
        let balance_post_cancel = router
            .wrap()
            .query_balance(user_1.clone(), NATIVE_DENOM_2)
            .unwrap();
        assert!(!balance_post_cancel.amount.is_zero());
        assert_eq!(balance_post_cancel, amount_order);
    }

    mod native_taker_orders {
        use crate::common::test_utils::TEST_USER_2;

        use super::*;

        #[test]
        fn native_limit_taker() {
            let (mut router, market_addr) = instantiate_selene();
            create_market_native_only_pair(&mut router, market_addr.clone());

            let user_1 = Addr::unchecked(TEST_USER_1);
            let user_2 = Addr::unchecked(TEST_USER_2);

            // mint some native denom of base currency
            let amount_order = Coin {
                denom: NATIVE_DENOM_2.into(),
                amount: Uint128::new(10000),
            };

            router.mint_native(&user_1, amount_order.clone());

            // send the order
            let order_price = Decimal::one();

            let msg = ExecuteMsg::LimitOrder {
                market_id: 0,
                price: order_price.clone(),
                //order_side: OrderSide::Buy,
            };

            let _res = router
                .execute_contract(
                    user_1.clone(),
                    market_addr.clone(),
                    &msg,
                    &[amount_order.clone()],
                )
                .unwrap();

            // now send taker order
            // mint some native denom of quote currency
            let amount_order = Coin {
                denom: NATIVE_DENOM_1.into(),
                amount: Uint128::new(10000),
            };

            router.mint_native(&user_2, amount_order.clone());

            let order_price = Decimal::one();

            let msg = ExecuteMsg::LimitOrder {
                market_id: 0,
                price: order_price.clone(),
                //order_side: OrderSide::Buy,
            };

            let _res = router
                .execute_contract(
                    user_2.clone(),
                    market_addr.clone(),
                    &msg,
                    &[amount_order.clone()],
                )
                .unwrap();

            // check user balance
            let balance_post_order = router
                .wrap()
                .query_balance(user_1.clone(), NATIVE_DENOM_1)
                .unwrap();
            assert!(!balance_post_order.amount.is_zero());
            assert_eq!(balance_post_order, amount_order);
        }

        #[test]
        fn native_limit_taker_not_price_one() {
            let (mut router, market_addr) = instantiate_selene();
            create_market_native_only_pair(&mut router, market_addr.clone());

            let user_1 = Addr::unchecked(TEST_USER_1);
            let user_2 = Addr::unchecked(TEST_USER_2);

            // mint some native denom of base currency
            let amount_order = Coin {
                denom: NATIVE_DENOM_2.into(),
                amount: Uint128::new(10000),
            };

            router.mint_native(&user_1, amount_order.clone());

            // send the order
            let order_price = Decimal::from_atomics(Uint128::new(5), 1).unwrap();

            let msg = ExecuteMsg::LimitOrder {
                market_id: 0,
                price: order_price.clone(),
                //order_side: OrderSide::Buy,
            };

            let _res = router
                .execute_contract(
                    user_1.clone(),
                    market_addr.clone(),
                    &msg,
                    &[amount_order.clone()],
                )
                .unwrap();

            // now send taker order
            // mint some native denom of quote currency
            let amount_order = Coin {
                denom: NATIVE_DENOM_1.into(),
                amount: Uint128::new(10000),
            };

            router.mint_native(&user_2, amount_order.clone());

            //let order_price = Decimal::one();

            let msg = ExecuteMsg::LimitOrder {
                market_id: 0,
                price: order_price.clone(),
                //order_side: OrderSide::Buy,
            };

            let _res = router
                .execute_contract(
                    user_2.clone(),
                    market_addr.clone(),
                    &msg,
                    &[amount_order.clone()],
                )
                .unwrap();

            // check user balance
            let balance_post_order = router
                .wrap()
                .query_balance(user_1.clone(), NATIVE_DENOM_1)
                .unwrap();
            assert!(!balance_post_order.amount.is_zero());
            //assert_eq!(balance_post_order, amount_order);
        }
    }
}
