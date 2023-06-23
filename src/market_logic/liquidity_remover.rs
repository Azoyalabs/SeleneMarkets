use cosmwasm_std::{Decimal, MessageInfo, Response, Storage, Uint128};

use crate::{
    state::{LEVEL_ORDERS, MARKET_INFO},
    state_utils,
    structs::{LevelOrder, OrderSide},
    utils::create_id_level_no_status,
    ContractError,
};

pub fn remove_order(
    storage: &mut dyn Storage,
    info: MessageInfo,
    market_id: u64,
    order_price: Decimal,
    order_quantity: Uint128,
    _order_side: OrderSide,
) -> Result<Response, ContractError> {
    let market_info = MARKET_INFO.load(storage, market_id)?;

    // get market id
    let id = create_id_level_no_status(&market_info, order_price);

    // remove order that matches data. We do not allow multiple orders by the same user at the same level
    let orders: Vec<LevelOrder> = LEVEL_ORDERS
        .load(storage, id)?
        .into_iter()
        .filter(|order| order.amount != order_quantity && order.user != info.sender)
        .collect();

    // remove level if no more orders
    if orders.len() == 0 {
        state_utils::remove_level(storage, market_id, id)?;
    } else {
        // else store it back
        LEVEL_ORDERS.save(storage, id, &orders)?;
    }

    return Ok(Response::new());
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::mock_dependencies, Addr, Decimal, MessageInfo, Uint128};

    use crate::{
        contract_admin_execute::add_market,
        market_logic::liquidity_provider::process_limit_maker,
        state::MARKET_INFO,
        structs::{CurrencyInfo, OrderSide},
    };

    use super::remove_order;

    mod only_bids {
        use super::*;

        #[test]
        fn liq_remover() {
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
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Buy,
            )
            .unwrap();

            // check market status
            let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
            assert!(market_info.top_level_bid.is_some());

            remove_order(
                deps.as_mut().storage,
                info,
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Buy,
            )
            .unwrap();

            // check market status, there shouldn't be anything left
            let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
            assert!(market_info.top_level_bid.is_none());
        }
    }

    mod only_asks {
        use super::*;

        #[test]
        fn liq_remover() {
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
                deps.as_mut().storage,
                Addr::unchecked("user").clone(),
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Sell,
            )
            .unwrap();

            // check market status
            let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
            assert!(market_info.top_level_ask.is_some());

            remove_order(
                deps.as_mut().storage,
                info,
                0,
                Decimal::one(),
                Uint128::new(100),
                OrderSide::Sell,
            )
            .unwrap();

            // check market status, there shouldn't be anything left
            let market_info = MARKET_INFO.load(deps.as_ref().storage, 0).unwrap();
            assert!(market_info.top_level_ask.is_none());
        }
    }
}
