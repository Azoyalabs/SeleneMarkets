use cosmwasm_std::Storage;

use crate::{
    state::{LEVELS_DATA, LEVEL_ORDERS, MARKET_INFO},
    ContractError,
};

/// Remove level data at id and joins previous and next in linked list  
/// If it is top of book, update
/// Also remove level orders at given id
pub fn remove_level(
    storage: &mut dyn Storage,
    market_id: u64,
    level_id: u64,
) -> Result<(), ContractError> {
    LEVEL_ORDERS.remove(storage, level_id);

    let level_data = LEVELS_DATA.load(storage, level_id)?;

    // if no previous, this means top of book so update market info
    if level_data.id_previous.is_none() {
        MARKET_INFO.update(
            storage,
            market_id,
            |market_info| -> Result<_, ContractError> {
                let mut market_info = market_info.unwrap();

                if let Some(val_top_bid_id) = market_info.top_level_bid {
                    if val_top_bid_id == level_id {
                        market_info.top_level_bid = level_data.id_next;
                    }
                } else if let Some(val_top_ask_id) = market_info.top_level_ask {
                    if val_top_ask_id == level_id {
                        market_info.top_level_ask = level_data.id_next;
                    }
                }

                return Ok(market_info);
            },
        )?;
    }

    // update connections in linked list
    if let Some(prev_id) = level_data.id_previous {
        LEVELS_DATA.update(storage, prev_id, |elem| -> Result<_, ContractError> {
            let mut elem = elem.unwrap();
            elem.id_next = level_data.id_next;

            return Ok(elem);
        })?;
    }

    if let Some(next_id) = level_data.id_next {
        LEVELS_DATA.update(storage, next_id, |elem| -> Result<_, ContractError> {
            let mut elem = elem.unwrap();
            elem.id_previous = level_data.id_previous;

            return Ok(elem);
        })?;
    }

    // delete state for level data
    LEVELS_DATA.remove(storage, level_id);

    return Ok(());
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::mock_dependencies, Addr, Decimal, MessageInfo, Uint128};

    use crate::{
        contract_admin_execute::add_market,
        market_logic::liquidity_provider::process_limit_maker,
        state::{LEVELS_DATA, LEVEL_ORDERS, MARKET_INFO},
        structs::{CurrencyInfo, OrderSide},
        utils::create_id_level_no_status,
    };

    use super::remove_level;

    /// Create a market with a single level then remove level
    /// should be no level anymore in market info
    #[test]
    fn test_utils_remove_single_level() {
        // set up
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

        let info = MessageInfo {
            sender: Addr::unchecked("user"),
            funds: vec![],
        };

        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            Decimal::one(),
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        // remove level
        let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
        let level_id = create_id_level_no_status(&market_info, Decimal::one());
        remove_level(deps.as_mut().storage, 0, level_id).unwrap();

        // check resulting, there should be no more orders and no more bids in book
        let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
        assert!(market_info.top_level_bid.is_none());
        assert!(LEVELS_DATA.load(deps.as_ref().storage, level_id).is_err());
        assert!(LEVEL_ORDERS.load(deps.as_ref().storage, level_id).is_err());
    }

    /// Create a market with a two level then remove top level
    /// top level should be deleted and top id should have been updated
    #[test]
    fn test_utils_remove_two_levels_remove_top() {
        // set up
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

        let info = MessageInfo {
            sender: Addr::unchecked("user"),
            funds: vec![],
        };

        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            Decimal::one(),
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        let top_price = Decimal::from_atomics(Uint128::new(1128), 3).unwrap();
        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            top_price,
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        // remove top level
        let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
        let level_id = create_id_level_no_status(&market_info, top_price);
        remove_level(deps.as_mut().storage, 0, level_id).unwrap();

        // check resulting, top id should have changed and records of top level should have been removed
        let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
        assert_ne!(market_info.top_level_bid.unwrap(), level_id);
        assert!(LEVELS_DATA.load(deps.as_ref().storage, level_id).is_err());
        assert!(LEVEL_ORDERS.load(deps.as_ref().storage, level_id).is_err());

        // check that there are no dangling connections for level data
        let level_data = LEVELS_DATA
            .load(deps.as_ref().storage, market_info.top_level_bid.unwrap())
            .unwrap();
        assert!(level_data.id_next.is_none());
        assert!(level_data.id_previous.is_none());
    }

    /// Create a market with a two level then remove bottom level
    /// bottom level should be deleted and top id should not have been updated
    #[test]
    fn test_utils_remove_two_levels_remove_bottom() {
        // set up
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

        let info = MessageInfo {
            sender: Addr::unchecked("user"),
            funds: vec![],
        };

        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            Decimal::one(),
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        let top_price = Decimal::from_atomics(Uint128::new(1128), 3).unwrap();
        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            top_price,
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        // remove bottom level
        let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
        let level_id = create_id_level_no_status(&market_info, Decimal::one());
        remove_level(deps.as_mut().storage, 0, level_id).unwrap();

        let top_level_id = create_id_level_no_status(&market_info, top_price);

        // check resulting, top id should not have changed and records of bottom level should have been removed
        let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
        assert_eq!(market_info.top_level_bid.unwrap(), top_level_id);
        assert!(LEVELS_DATA.load(deps.as_ref().storage, level_id).is_err());
        assert!(LEVEL_ORDERS.load(deps.as_ref().storage, level_id).is_err());

        // check that there are no dangling connections for level data
        let level_data = LEVELS_DATA
            .load(deps.as_ref().storage, top_level_id)
            .unwrap();
        assert!(level_data.id_next.is_none());
        assert!(level_data.id_previous.is_none());
    }

    /// Create a market with a three levels then remove top level
    /// top level should be deleted and top id should have been updated
    /// connections for the following level should have been modified
    #[test]
    fn test_utils_remove_three_levels_remove_top() {
        // set up
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

        let info = MessageInfo {
            sender: Addr::unchecked("user"),
            funds: vec![],
        };

        let middle_price = Decimal::one();
        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            middle_price,
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        let top_price = Decimal::from_atomics(Uint128::new(1128), 3).unwrap();
        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            top_price,
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        let bottom_price = Decimal::from_atomics(Uint128::new(976), 3).unwrap();
        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            bottom_price,
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();

        let top_level_id = create_id_level_no_status(&market_info, top_price);
        let middle_level_id = create_id_level_no_status(&market_info, middle_price);
        let bottom_level_id = create_id_level_no_status(&market_info, bottom_price);

        assert_eq!(market_info.top_level_bid.unwrap(), top_level_id);

        // remove top level
        remove_level(deps.as_mut().storage, 0, top_level_id).unwrap();

        // check resulting, top id should not have changed and records of top level should have been removed
        let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
        assert_eq!(market_info.top_level_bid.unwrap(), middle_level_id);
        assert!(LEVELS_DATA
            .load(deps.as_ref().storage, top_level_id)
            .is_err());
        assert!(LEVEL_ORDERS
            .load(deps.as_ref().storage, top_level_id)
            .is_err());
        assert!(LEVELS_DATA
            .load(deps.as_ref().storage, middle_level_id)
            .is_ok());
        assert!(LEVEL_ORDERS
            .load(deps.as_ref().storage, middle_level_id)
            .is_ok());
        assert!(LEVELS_DATA
            .load(deps.as_ref().storage, bottom_level_id)
            .is_ok());
        assert!(LEVEL_ORDERS
            .load(deps.as_ref().storage, bottom_level_id)
            .is_ok());

        // check that there are no dangling connections for level data
        let middle_level_data = LEVELS_DATA
            .load(deps.as_ref().storage, middle_level_id)
            .unwrap();
        assert_eq!(middle_level_data.id_next.unwrap(), bottom_level_id);
        assert!(middle_level_data.id_previous.is_none());

        let bottom_level_data = LEVELS_DATA
            .load(deps.as_ref().storage, bottom_level_id)
            .unwrap();

        assert!(bottom_level_data.id_next.is_none());
        assert_eq!(bottom_level_data.id_previous.unwrap(), middle_level_id);
    }

    /// Create a market with a three levels then remove middle level
    /// middle level should be deleted and top id should not have been updated
    /// connections for the top and bottom level should have been modified
    #[test]
    fn test_utils_remove_three_levels_remove_middle() {
        // set up
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

        let info = MessageInfo {
            sender: Addr::unchecked("user"),
            funds: vec![],
        };

        let middle_price = Decimal::one();
        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            middle_price,
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        let top_price = Decimal::from_atomics(Uint128::new(1128), 3).unwrap();
        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            top_price,
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        let bottom_price = Decimal::from_atomics(Uint128::new(976), 3).unwrap();
        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            bottom_price,
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();

        let top_level_id = create_id_level_no_status(&market_info, top_price);
        let middle_level_id = create_id_level_no_status(&market_info, middle_price);
        let bottom_level_id = create_id_level_no_status(&market_info, bottom_price);

        assert_eq!(market_info.top_level_bid.unwrap(), top_level_id);

        // remove middle level
        remove_level(deps.as_mut().storage, 0, middle_level_id).unwrap();

        // check resulting, top id should not have changed and records of middle level should have been removed
        let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
        assert_eq!(market_info.top_level_bid.unwrap(), top_level_id);
        assert!(LEVELS_DATA
            .load(deps.as_ref().storage, top_level_id)
            .is_ok());
        assert!(LEVEL_ORDERS
            .load(deps.as_ref().storage, top_level_id)
            .is_ok());
        assert!(LEVELS_DATA
            .load(deps.as_ref().storage, middle_level_id)
            .is_err());
        assert!(LEVEL_ORDERS
            .load(deps.as_ref().storage, middle_level_id)
            .is_err());
        assert!(LEVELS_DATA
            .load(deps.as_ref().storage, bottom_level_id)
            .is_ok());
        assert!(LEVEL_ORDERS
            .load(deps.as_ref().storage, bottom_level_id)
            .is_ok());

        // check that there are no dangling connections for level data
        let top_level_data = LEVELS_DATA
            .load(deps.as_ref().storage, top_level_id)
            .unwrap();
        assert_eq!(top_level_data.id_next.unwrap(), bottom_level_id);
        assert!(top_level_data.id_previous.is_none());

        let bottom_level_data = LEVELS_DATA
            .load(deps.as_ref().storage, bottom_level_id)
            .unwrap();

        assert!(bottom_level_data.id_next.is_none());
        assert_eq!(bottom_level_data.id_previous.unwrap(), top_level_id);
    }

    /// Create a market with a three levels then remove bottom level
    /// top level should be same and top id should not have been updated
    /// connections for the middle level should have been modified
    #[test]
    fn test_utils_remove_three_levels_remove_bottom() {
        // set up
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

        let info = MessageInfo {
            sender: Addr::unchecked("user"),
            funds: vec![],
        };

        let middle_price = Decimal::one();
        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            middle_price,
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        let top_price = Decimal::from_atomics(Uint128::new(1128), 3).unwrap();
        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            top_price,
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        let bottom_price = Decimal::from_atomics(Uint128::new(976), 3).unwrap();
        process_limit_maker(
            deps.as_mut(),
            Addr::unchecked("user").clone(),
            0,
            bottom_price,
            Uint128::new(100),
            OrderSide::Buy,
        )
        .unwrap();

        let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();

        let top_level_id = create_id_level_no_status(&market_info, top_price);
        let middle_level_id = create_id_level_no_status(&market_info, middle_price);
        let bottom_level_id = create_id_level_no_status(&market_info, bottom_price);

        assert_eq!(market_info.top_level_bid.unwrap(), top_level_id);

        // remove bottom level
        remove_level(deps.as_mut().storage, 0, bottom_level_id).unwrap();

        // check resulting, top id should not have changed and records of bottom level should have been removed
        let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
        assert_eq!(market_info.top_level_bid.unwrap(), top_level_id);
        assert!(LEVELS_DATA
            .load(deps.as_ref().storage, top_level_id)
            .is_ok());
        assert!(LEVEL_ORDERS
            .load(deps.as_ref().storage, top_level_id)
            .is_ok());
        assert!(LEVELS_DATA
            .load(deps.as_ref().storage, middle_level_id)
            .is_ok());
        assert!(LEVEL_ORDERS
            .load(deps.as_ref().storage, middle_level_id)
            .is_ok());
        assert!(LEVELS_DATA
            .load(deps.as_ref().storage, bottom_level_id)
            .is_err());
        assert!(LEVEL_ORDERS
            .load(deps.as_ref().storage, bottom_level_id)
            .is_err());

        // check that there are no dangling connections for level data
        let top_level_data = LEVELS_DATA
            .load(deps.as_ref().storage, top_level_id)
            .unwrap();
        assert_eq!(top_level_data.id_next.unwrap(), middle_level_id);
        assert!(top_level_data.id_previous.is_none());

        let middle_level_data = LEVELS_DATA
            .load(deps.as_ref().storage, middle_level_id)
            .unwrap();

        assert!(middle_level_data.id_next.is_none());
        assert_eq!(middle_level_data.id_previous.unwrap(), top_level_id);
    }
}
