#![allow(unused)]

mod common;

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
    use cw_multi_test::{App, Executor};
    use selene_markets::{
        msg::{
            ExecuteMsg, GetMarketBookResponse, GetMarketsResponse, GetUserAsksResponse,
            GetUserBidsResponse, GetUserOrdersResponse, QueryMsg,
        },
        state::LEVEL_ORDERS,
        structs::OrderSide,
    };

    use crate::common::test_utils::{
        create_market_native_only_pair, instantiate_selene, CashMachine, NATIVE_DENOM_1,
        NATIVE_DENOM_2, TEST_USER_1, TEST_USER_2,
    };

    pub const NATIVE_DENOM_1_EUR: &'static str = "heur";
    pub const NATIVE_DENOM_2_USD: &'static str = "husd";

    #[test]
    fn only_native_limit_taker() {
        let (mut router, market_addr) = instantiate_selene();
        create_market_native_only_pair(&mut router, market_addr.clone());

        let user_1 = Addr::unchecked(TEST_USER_1);
        let user_2 = Addr::unchecked(TEST_USER_2);

        // mint some native denom of base currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_2_USD.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_1, amount_order.clone());

        // send the order
        let order_price = Decimal::from_atomics(Uint128::new(500), 1).unwrap();

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

        // order should be in the book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };
        let res: GetMarketBookResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        println!("book: {:?}", res);

        // now send taker order
        // mint some native denom of quote currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_1_EUR.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_2, amount_order.clone());

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

        // check what remains on the book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };
        let res: GetMarketBookResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        println!("book: {:?}", res);

        // user_1 should have received NATIVE_DENOM_1
        let balance_post_order = router
            .wrap()
            .query_balance(user_1.clone(), NATIVE_DENOM_1_EUR)
            .unwrap();
        assert!(!balance_post_order.amount.is_zero());
        println!("balance: {}", balance_post_order);

        // user_2 should have received NATIVE_DENOM_2
        let balance_post_order = router
            .wrap()
            .query_balance(user_2.clone(), NATIVE_DENOM_2_USD)
            .unwrap();
        //assert!(!balance_post_order.amount.is_zero());
        println!("balance: {}", balance_post_order);
    }

    /// send a limit taker that is much larget than available on the book
    /// should create appropriate new level
    #[test]
    fn only_native_limit_big_order() {
        let (mut router, market_addr) = instantiate_selene();
        create_market_native_only_pair(&mut router, market_addr.clone());

        let user_1 = Addr::unchecked(TEST_USER_1);
        let user_2 = Addr::unchecked(TEST_USER_2);

        // mint some native denom of base currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_2_USD.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_1, amount_order.clone());

        // send the buy order
        let order_price = Decimal::from_atomics(Uint128::new(500), 1).unwrap();

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

        // order should be in the book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };
        let res: GetMarketBookResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();

        // now send taker order
        // mint some native denom of quote currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_1_EUR.into(),
            amount: Uint128::new(100000000),
        };

        router.mint_native(&user_2, amount_order.clone());

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

        // check what remains on the book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };
        let res: GetMarketBookResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();

        // the bid should have been consumed, there should be a ask instead
        assert_eq!(res.bids.len(), 0);
        assert_eq!(res.asks.len(), 1);

        // and the ask should be from user_2
        let msg = QueryMsg::GetUserOrders {
            user_address: user_2.clone(),
            target_market: Some(0),
        };
        let res: GetUserOrdersResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.orders.len(), 1);
        assert_eq!(res.orders[0].order_side, OrderSide::Sell);

        let msg = QueryMsg::GetUserOrders {
            user_address: user_1.clone(),
            target_market: Some(0),
        };
        let res: GetUserOrdersResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();

        assert_eq!(res.orders.len(), 0);
    }

    /// send a limit taker and check returned balances
    #[test]
    fn only_native_limit_check_cashback() {
        let (mut router, market_addr) = instantiate_selene();
        create_market_native_only_pair(&mut router, market_addr.clone());

        let user_1 = Addr::unchecked(TEST_USER_1);
        let user_2 = Addr::unchecked(TEST_USER_2);

        // mint some native denom of base currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_2_USD.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_1, amount_order.clone());

        // send the buy order
        let order_price = Decimal::from_atomics(Uint128::new(500), 1).unwrap();

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

        // order should be in the book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };
        let res: GetMarketBookResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();

        // now send taker order
        // mint some native denom of quote currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_1_EUR.into(),
            amount: Uint128::new(100000000),
        };

        router.mint_native(&user_2, amount_order.clone());

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

        // check what remains on the book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };
        let res: GetMarketBookResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();

        println!("book: {:?}", res);

        // the bid should have been consumed, there should be a ask instead
        assert_eq!(res.bids.len(), 0);
        assert_eq!(res.asks.len(), 1);

        // and the ask should be from user_2
        let msg = QueryMsg::GetUserOrders {
            user_address: user_2.clone(),
            target_market: Some(0),
        };
        let res: GetUserOrdersResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.orders.len(), 1);
        assert_eq!(res.orders[0].order_side, OrderSide::Sell);

        let msg = QueryMsg::GetUserOrders {
            user_address: user_1.clone(),
            target_market: Some(0),
        };
        let res: GetUserOrdersResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();

        assert_eq!(res.orders.len(), 0);

        // checking out balances
        let balances_user_1 = router.wrap().query_all_balances(user_1.clone()).unwrap();
        let balances_user_2 = router.wrap().query_all_balances(user_2.clone()).unwrap();
        println!("bal user 1: {:?}", balances_user_1);
        println!("bal user 2: {:?}", balances_user_2);

        let balances_contract = router
            .wrap()
            .query_all_balances(market_addr.clone())
            .unwrap();
        println!("bal user contract: {:?}", balances_contract);
    }

    /// Try to replicate an issue with taker that creates an order above ask
    #[test]
    fn only_native_replication() {
        let (mut router, market_addr) = instantiate_selene();
        create_market_native_only_pair(&mut router, market_addr.clone());

        let user_1 = Addr::unchecked(TEST_USER_1);
        let user_2 = Addr::unchecked(TEST_USER_2);

        // mint some native denom of base currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_2_USD.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_1, amount_order.clone());

        // send the bid order
        let bid_order_price = Decimal::from_atomics(Uint128::new(1500), 1).unwrap();

        let msg = ExecuteMsg::LimitOrder {
            market_id: 0,
            price: bid_order_price.clone(),
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

        // now send limit order on the other side, which should consume the bid order
        // mint some native denom of base currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_1_EUR.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_1, amount_order.clone());
        // check what remains on the book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };
        let res: GetMarketBookResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        println!("book: {:?}", res);

        // send ask order
        let ask_order_price = Decimal::from_atomics(Uint128::new(500), 1).unwrap();

        let msg = ExecuteMsg::LimitOrder {
            market_id: 0,
            price: ask_order_price.clone(),
        };

        let _res = router
            .execute_contract(
                user_1.clone(),
                market_addr.clone(),
                &msg,
                &[amount_order.clone()],
            )
            .unwrap();

        // check what remains on the book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };
        let res: GetMarketBookResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        println!("book: {:?}", res);

        // now try to consume the ask
        let amount_order = Coin {
            denom: NATIVE_DENOM_1_EUR.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_2, amount_order.clone());

        let ask_order_price = Decimal::from_atomics(Uint128::new(2500), 1).unwrap();

        let msg = ExecuteMsg::LimitOrder {
            market_id: 0,
            price: bid_order_price.clone(),
        };

        let _res = router
            .execute_contract(
                user_2.clone(),
                market_addr.clone(),
                &msg,
                &[amount_order.clone()],
            )
            .unwrap();

        // check what remains on the book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };
        let res: GetMarketBookResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
    }

    #[test]
    fn native_market() {
        let (mut router, market_addr) = instantiate_selene();
        create_market_native_only_pair(&mut router, market_addr.clone());

        let user_1 = Addr::unchecked(TEST_USER_1);
        let user_2 = Addr::unchecked(TEST_USER_2);

        // mint some native denom of base currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_2_USD.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_1, amount_order.clone());

        // send the bid order
        let bid_order_price = Decimal::from_atomics(Uint128::new(1500), 1).unwrap();

        let msg = ExecuteMsg::LimitOrder {
            market_id: 0,
            price: bid_order_price.clone(),
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

        // now send market order on the other side, which should consume the bid order
        // mint some native denom of base currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_1_EUR.into(),
            amount: Uint128::new(10),
        };

        router.mint_native(&user_1, amount_order.clone());
        // check what remains on the book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };

        // send ask order
        let ask_order_price = Decimal::from_atomics(Uint128::new(500), 1).unwrap();

        let msg = ExecuteMsg::MarketOrder { market_id: 0 };

        let _res = router
            .execute_contract(
                user_1.clone(),
                market_addr.clone(),
                &msg,
                &[amount_order.clone()],
            )
            .unwrap();

        let msg = QueryMsg::GetMarketBook { market_id: 0, nb_levels: 10 };
        let res: GetMarketBookResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        println!("book: {:?}", res);
    }


}
