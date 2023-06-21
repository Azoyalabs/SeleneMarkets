use cosmwasm_std::{Decimal, DepsMut, MessageInfo, Response, Uint128};

use crate::{
    state::{LEVELS_DATA, LEVEL_ORDERS, MARKET_INFO},
    structs::{LevelOrder, OrderSide},
    utils::create_id_level_no_status,
    ContractError,
};

pub fn remove_order(
    deps: DepsMut,
    info: MessageInfo,
    market_id: u64,
    order_price: Decimal,
    order_quantity: Uint128,
    order_side: OrderSide,
) -> Result<Response, ContractError> {
    let market_info = MARKET_INFO.load(deps.storage, market_id)?;

    // remove order
    let id = create_id_level_no_status(&market_info, order_price);
    /*
    LEVELS_DATA.update(deps.storage, id, |level_data| -> Result<_, ContractError> {
        let mut level_data = level_data.unwrap();

    })?;
    */

    /*
    LEVEL_ORDERS.update(deps.storage, id, |orders| -> Result<_, ContractError> {
        let mut orders = orders.unwrap();


        return Ok(orders);
    })?;
    */

    let mut orders = LEVEL_ORDERS.load(deps.storage, id)?;
    // remove order that matches data. We do not allow multiple orders by the same user at the same level
    let orders: Vec<LevelOrder> = orders
        .into_iter()
        .filter(|order| order.amount != order_quantity && order.user != info.sender)
        .collect();

    // remove level if no more orders
    if orders.len() == 0 {
        LEVEL_ORDERS.remove(deps.storage, id);

        let level_data = LEVELS_DATA.load(deps.storage, id)?;

        match level_data.id_previous {
            None => {
                // no previous in chain, so top of book, will need to update market info as well
                MARKET_INFO.update(
                    deps.storage,
                    market_id,
                    |elem| -> Result<_, ContractError> {
                        let mut elem = elem.unwrap();
                        match order_side {
                            OrderSide::Buy => elem.top_level_bid = level_data.id_next,
                            OrderSide::Sell => elem.top_level_ask = level_data.id_next,
                        }

                        return Ok(elem);
                    },
                )?;

                // and need to update next in chain if it exists
                match level_data.id_next {
                    None => (),
                    Some(val_id_next) => {
                        LEVELS_DATA.update(
                            deps.storage,
                            val_id_next,
                            |elem| -> Result<_, ContractError> {
                                let mut elem = elem.unwrap();
                                elem.id_previous = level_data.id_previous;

                                return Ok(elem);
                            },
                        )?;
                    }
                }
            }
            Some(val_id_previous) => {
                panic!();
            }
        }
    } else {
        // else store it back
        LEVEL_ORDERS.save(deps.storage, id, &orders)?;
    }

    return Ok(Response::new());
}
