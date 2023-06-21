mod common;

#[cfg(test)]
mod tests {
    use cosmwasm_contract_template::msg::InstantiateMsg;
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, Executor};

    use crate::common::test_utils::get_contract;

    const TEST_ADMIN: &'static str = "ADMIN";
    const CONTRACT_LABEL: &'static str = "CONTRACT_LABEL";

    #[test]
    fn successful_deployment() {
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
}
