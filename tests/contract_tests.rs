#![allow(unused)]

mod common;

#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, Executor};
    use selene_markets::{
        msg::{AdminExecuteMsg, ExecuteMsg, GetMarketsResponse, InstantiateMsg, QueryMsg},
        structs::CurrencyInfo,
    };

    use crate::common::test_utils::{
        create_market_native_only_pair, get_contract, instantiate_selene, CONTRACT_LABEL,
        NATIVE_DENOM_1, NATIVE_DENOM_2, TEST_ADMIN,
    };

    #[test]
    fn test_successful_deployment() {
        let mut router = App::default();
        let contract_code_id = router.store_code(get_contract());

        let instantiate_msg = InstantiateMsg {};

        let admin = Addr::unchecked(TEST_ADMIN);
        let instantiate_res = router.instantiate_contract(
            contract_code_id,
            admin,
            &instantiate_msg,
            &[],
            String::from(CONTRACT_LABEL),
            Some(TEST_ADMIN.to_owned()),
        );

        match instantiate_res {
            Ok(_contract_address) => (),
            Err(err) => panic!("Failed to instantiate contract: {}", err.to_string()),
        }
    }

    #[test]
    fn test_create_market() {
        let (mut router, market_addr) = instantiate_selene();
        create_market_native_only_pair(&mut router, market_addr.clone());

        // query markets, check if only one
        let msg = QueryMsg::GetMarkets {};
        let res: GetMarketsResponse = router
            .wrap()
            .query_wasm_smart(market_addr.clone(), &msg)
            .unwrap();

        assert_eq!(res.markets.len(), 1);
    }
}
