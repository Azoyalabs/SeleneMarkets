use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal};
use cw20::Cw20ReceiveMsg;

use crate::structs::{CurrencyInfo, OrderSide, UserOrder};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum AdminExecuteMsg {
    UpdateAdmin {
        new_admin: String,
    },
    //AddMarket { },
    AddMarket {
        base_currency: CurrencyInfo,
        quote_currency: CurrencyInfo,
    },
}

/// messages to be used in a cw20::send message
#[cw_serde]
pub enum SeleneCw20Msg {
    LimitOrder {
        market_id: u64,
        price: Decimal,
        order_side: OrderSide,
    },
    MarketOrder {
        market_id: u64,
        order_side: OrderSide,
    },
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    /// limit order for a native coin
    LimitOrder {
        market_id: u64,
        price: Decimal,
        order_side: OrderSide,
    },
    /// market order for a native coin
    MarketOrder {
        market_id: u64,
    },

    Admin(AdminExecuteMsg),
}

#[cw_serde]
pub enum MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetAdminResponse)]
    GetAdmin {},

    #[returns(GetUserBids)]
    GetUserBids { target_market: Option<u64> },

    #[returns(GetUserAsks)]
    GetUserAsks { target_market: Option<u64> },
}

#[cw_serde]
pub struct GetUserBids {
    pub orders: Vec<UserOrder>,
}

#[cw_serde]
pub struct GetUserAsks {
    pub orders: Vec<UserOrder>,
}

#[cw_serde]
pub struct GetAdminResponse {
    pub admin: Option<Addr>,
}
