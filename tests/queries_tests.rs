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
    fn queries_set_bid_and_ask_is_in_book() {
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
        };

        let _res = router
            .execute_contract(
                user_1.clone(),
                market_addr.clone(),
                &msg,
                &[amount_order.clone()],
            )
            .unwrap();

        // mint some native denom of quote currency
        let amount_order = Coin {
            denom: NATIVE_DENOM_1.into(),
            amount: Uint128::new(10000),
        };

        router.mint_native(&user_1, amount_order.clone());

        // send the order
        let msg = ExecuteMsg::LimitOrder {
            market_id: 0,
            price: Decimal::from_atomics(Uint128::new(2), 0).unwrap(),
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
        let res: GetUserAsksResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.orders.len(), 1);

        // query book
        let msg = QueryMsg::GetMarketBook {
            market_id: 0,
            nb_levels: 10,
        };
        let res: GetMarketBookResponse = router.wrap().query_wasm_smart(market_addr, &msg).unwrap();
        assert_eq!(res.bids.len(), 1);
        assert_eq!(res.asks.len(), 1);
    }
}
