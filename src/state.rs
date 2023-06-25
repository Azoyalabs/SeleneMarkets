use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::structs::{LevelData, LevelOrders, MarketInfo, UserOrderRecord};

pub const ADMIN: Item<Addr> = Item::new("admin");

/// Allocate ID to new markets  
pub const MARKET_ID_TRACKER: Item<u64> = Item::new("market_id_tracker");

/// Map market id to market info
pub const MARKET_INFO: Map<u64, MarketInfo> = Map::new("market_info");

//pub const MARKET_ORDERS: Map<(u64, OrderSide), >

/// Map level id to info about level (price and linked nodes as market is doubly linked list)
pub const LEVELS_DATA: Map<u64, LevelData> = Map::new("levels_data");

/// Map level id to orders at this price
pub const LEVEL_ORDERS: Map<u64, LevelOrders> = Map::new("level_orders");

// Tracking user orders
pub const USER_ORDERS: Map<Addr, Vec<UserOrderRecord>> = Map::new("user_orders");

/// Allocate id to new orders
pub const ORDER_ID_TRACKER: Item<u64> = Item::new("order_id_tracker");
