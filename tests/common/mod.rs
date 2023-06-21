#[cfg(test)]
pub mod test_utils {
    use cosmwasm_std::Empty;
    use cw_multi_test::{Contract, ContractWrapper};

    // You'll need to change the lib name here
    use selene_markets::contract::{execute, instantiate, query};

    pub fn get_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query); //.with_reply(reply);
        Box::new(contract)
    }
}
