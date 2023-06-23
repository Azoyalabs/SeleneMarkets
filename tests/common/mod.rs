#![allow(unused)]

#[cfg(test)]
pub mod test_utils {
    use cosmwasm_std::{Addr, Coin, Empty};
    use cw_multi_test::{App, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

    // You'll need to change the lib name here
    use selene_markets::{
        contract::{execute, instantiate, query},
        msg::{AdminExecuteMsg, ExecuteMsg, InstantiateMsg},
        structs::CurrencyInfo,
    };

    pub const TEST_ADMIN: &'static str = "admin";
    pub const CONTRACT_LABEL: &'static str = "CONTRACT_LABEL";

    pub const TEST_USER_1: &'static str = "user1";
    pub const TEST_USER_2: &'static str = "user2";

    pub const NATIVE_DENOM_1: &'static str = "heur";
    pub const NATIVE_DENOM_2: &'static str = "husd";

    pub fn get_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query); //.with_reply(reply);
        Box::new(contract)
    }

    pub fn instantiate_selene() -> (App, Addr) {
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

        return (router, instantiate_res.unwrap());
    }

    pub trait CashMachine {
        fn mint_native(&mut self, beneficiary: &Addr, to_mint: Coin);
    }

    impl CashMachine for App {
        fn mint_native(&mut self, beneficiary: &Addr, to_mint: Coin) {
            self.sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: beneficiary.to_owned().into_string(),
                amount: vec![to_mint],
            }))
            .unwrap();
        }
    }

    pub fn create_market_native_only_pair(router: &mut App, contract_addr: Addr) {
        let admin = Addr::unchecked(TEST_ADMIN);

        let admin_msg = AdminExecuteMsg::AddMarket {
            base_currency: CurrencyInfo::Native {
                denom: NATIVE_DENOM_1.into(),
            },
            quote_currency: CurrencyInfo::Native {
                denom: NATIVE_DENOM_2.into(),
            },
        };

        let msg = ExecuteMsg::Admin(admin_msg);

        let _res = router
            .execute_contract(admin, contract_addr.clone(), &msg, &[])
            .unwrap();
    }
}
