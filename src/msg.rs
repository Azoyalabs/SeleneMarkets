use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal};
use cw20::Cw20ReceiveMsg;

use crate::structs::{BookLevel, CurrencyInfo, SingleMarketInfo, UserOrderRecord};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum AdminExecuteMsg {
    UpdateAdmin {
        new_admin: String,
    },
    AddMarket {
        base_currency: CurrencyInfo,
        quote_currency: CurrencyInfo,
    },
}

/// messages to be used in a cw20::send message
#[cw_serde]
pub enum SeleneCw20Msg {
    LimitOrder { market_id: u64, price: Decimal },
    MarketOrder { market_id: u64 },
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    /// limit order for a native coin
    LimitOrder {
        market_id: u64,
        price: Decimal,
    },
    /// market order for a native coin
    MarketOrder {
        market_id: u64,
    },
    RemoveLimitOrder {
        market_id: u64,
        price: Decimal,
    },

    Admin(AdminExecuteMsg),
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetAdminResponse)]
    GetAdmin {},

    #[returns(GetMarketsResponse)]
    GetMarkets {},

    #[returns(GetUserBidsResponse)]
    GetUserBids {
        user_address: Addr,
        target_market: Option<u64>,
    },

    #[returns(GetUserAsksResponse)]
    GetUserAsks {
        user_address: Addr,
        target_market: Option<u64>,
    },

    #[returns(GetUserOrdersResponse)]
    GetUserOrders {
        user_address: Addr,
        target_market: Option<u64>,
    },

    #[returns(GetMarketBookResponse)]
    GetMarketBook { market_id: u64, nb_levels: u32 },
}

#[cw_serde]
pub struct GetMarketBookResponse {
    pub bids: Vec<BookLevel>,
    pub asks: Vec<BookLevel>,
}

#[cw_serde]
pub struct GetMarketsResponse {
    pub markets: Vec<SingleMarketInfo>,
}

#[cw_serde]
pub struct GetUserOrdersResponse {
    pub orders: Vec<UserOrderRecord>,
}

#[cw_serde]
pub struct GetUserBidsResponse {
    pub orders: Vec<UserOrderRecord>,
}

#[cw_serde]
pub struct GetUserAsksResponse {
    pub orders: Vec<UserOrderRecord>,
}

#[cw_serde]
pub struct GetAdminResponse {
    pub admin: Option<Addr>,
}
