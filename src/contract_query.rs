use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdResult, Uint256};
use erased_serde::Serialize;

use crate::{
    msg::{
        GetAdminResponse, GetMarketBookResponse, GetMarketsResponse, GetUserAsksResponse,
        GetUserBidsResponse, GetUserOrdersResponse, QueryMsg,
    },
    state::{ADMIN, LEVELS_DATA, LEVEL_ORDERS, MARKET_INFO, USER_ORDERS},
    structs::{BookLevel, OrderSide, SingleMarketInfo},
};

pub fn route_query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let res: Box<dyn Serialize> = match msg {
        QueryMsg::GetAdmin {} => get_admin(deps),
        QueryMsg::GetMarkets {} => get_markets(deps),
        QueryMsg::GetUserBids {
            user_address,
            target_market,
        } => get_user_bids(deps, user_address, target_market),
        QueryMsg::GetUserAsks {
            user_address,
            target_market,
        } => get_user_asks(deps, user_address, target_market),
        QueryMsg::GetMarketBook {
            market_id,
            nb_levels,
        } => get_market_book(deps, market_id, nb_levels),
        QueryMsg::GetUserOrders {
            user_address,
            target_market,
        } => get_user_orders(deps, user_address, target_market),
        //_ => panic!("Not implemented"),
    };

    return Ok(to_binary(&res)?);
}

fn get_market_book(deps: Deps, market_id: u64, nb_levels: u32) -> Box<dyn Serialize> {
    let market_info = MARKET_INFO.load(deps.storage, market_id).unwrap();

    let mut bids: Vec<BookLevel> = vec![];
    let mut asks: Vec<BookLevel> = vec![];

    let mut curr_depth: u32 = 0;
    let mut curr_id_level = market_info.top_level_bid;
    loop {
        if curr_depth >= nb_levels {
            break;
        }
        match curr_id_level {
            None => break,
            Some(val_curr_id) => {
                let level_data = LEVELS_DATA.load(deps.storage, val_curr_id).unwrap();
                let level_orders = LEVEL_ORDERS.load(deps.storage, val_curr_id).unwrap();

                bids.push(BookLevel {
                    price: level_data.price,
                    quantity: level_orders
                        .iter()
                        .map(|order| Uint256::from(order.amount))
                        .sum(),
                });

                curr_depth += 1;
                curr_id_level = level_data.id_next;
            }
        }
    }

    curr_depth = 0;
    curr_id_level = market_info.top_level_ask;
    loop {
        if curr_depth >= nb_levels {
            break;
        }
        match curr_id_level {
            None => break,
            Some(val_curr_id) => {
                let level_data = LEVELS_DATA.load(deps.storage, val_curr_id).unwrap();
                let level_orders = LEVEL_ORDERS.load(deps.storage, val_curr_id).unwrap();

                asks.push(BookLevel {
                    price: level_data.price,
                    quantity: level_orders
                        .iter()
                        .map(|order| Uint256::from(order.amount))
                        .sum(),
                });

                curr_depth += 1;
                curr_id_level = level_data.id_next;
            }
        }
    }

    return Box::new(GetMarketBookResponse {
        bids: bids,
        asks: asks,
    });
}

fn get_user_orders(
    deps: Deps,
    user_address: Addr,
    target_market: Option<u64>,
) -> Box<dyn Serialize> {
    let user_orders = USER_ORDERS
        .load(deps.storage, user_address)
        .unwrap_or_default();

    let user_orders = match target_market {
        None => user_orders,
        Some(target_market_id) => user_orders
            .into_iter()
            .filter(|order| order.market_id == target_market_id)
            .collect(),
    };

    return Box::new(GetUserOrdersResponse {
        orders: user_orders,
    });
}

fn get_user_bids(deps: Deps, user_address: Addr, target_market: Option<u64>) -> Box<dyn Serialize> {
    let user_orders = USER_ORDERS
        .load(deps.storage, user_address)
        .unwrap_or_default();

    let user_orders = match target_market {
        None => user_orders,
        Some(target_market_id) => user_orders
            .into_iter()
            .filter(|order| {
                order.market_id == target_market_id && order.order_side == OrderSide::Buy
            })
            .collect(),
    };

    return Box::new(GetUserBidsResponse {
        orders: user_orders,
    });
}

fn get_user_asks(deps: Deps, user_address: Addr, target_market: Option<u64>) -> Box<dyn Serialize> {
    let user_orders = USER_ORDERS
        .load(deps.storage, user_address)
        .unwrap_or_default();

    let user_orders = match target_market {
        None => user_orders,
        Some(target_market_id) => user_orders
            .into_iter()
            .filter(|order| {
                order.market_id == target_market_id && order.order_side == OrderSide::Sell
            })
            .collect(),
    };

    return Box::new(GetUserAsksResponse {
        orders: user_orders,
    });
}

/*
fn get_user_bids(deps: Deps, user_address: Addr, target_market: Option<u64>) -> Box<dyn Serialize> {
    let user_orders: Vec<InternalUserOrder> = USER_ORDERS
        .prefix(user_address)
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|elem| match elem {
            Err(_) => None,
            Ok((market_id, user_orders)) => match target_market {
                None => Some(user_orders),
                Some(id_target_market) => {
                    if market_id == id_target_market {
                        Some(user_orders.iter().map(|order| {
                            UserOrder {
                                quantity: order
                            }
                        }))
                    } else {
                        None
                    }
                }
            },
        })
        .flatten()
        .collect();

    // flatten

    return Box::new(GetUserBidsResponse { orders: vec![] });
}
*/

fn get_markets(deps: Deps) -> Box<dyn Serialize> {
    let markets = MARKET_INFO
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|elem| match elem {
            Err(_) => None,
            Ok((_, market_info)) => Some(SingleMarketInfo {
                market_id: market_info.market_id,
                quote_currency: market_info.quote_currency,
                base_currency: market_info.base_currency,
            }),
        })
        .collect();

    return Box::new(GetMarketsResponse { markets: markets });
}

fn get_admin(deps: Deps) -> Box<dyn Serialize> {
    return Box::new(GetAdminResponse {
        admin: ADMIN.load(deps.storage).ok(),
    });
}
