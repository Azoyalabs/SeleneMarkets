use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128, Uint256};
use serde::{Deserialize, Serialize};

use crate::ContractError;

#[cw_serde]
pub enum OrderSide {
    Buy,
    Sell,
}

#[cw_serde]
pub enum OrderType {
    Maker,
    Taker,
}

#[cw_serde]
#[derive(Hash)]
pub enum CurrencyInfo {
    Native { denom: String },
    Cw20 { address: String },
}

/// In the case of BTC-USD, BTC is the base currency and USD the quote currency
/// which means: if we receive BaseCurrency, then it's a sell order, else it's a buy
/// e.g. on BTC-USD if we receive USD it's to buy BTC
#[cw_serde]
#[derive(Hash)]
pub enum CurrencyStatus {
    BaseCurrency,
    QuoteCurrency,
}

#[cw_serde]
pub struct MarketInfo {
    pub market_id: u64,
    pub base_currency: CurrencyInfo,
    pub quote_currency: CurrencyInfo,
    pub top_level_bid: Option<u64>, // Option<u64>
    pub top_level_ask: Option<u64>, // Option<u64>
}

#[cw_serde]
pub struct SingleMarketInfo {
    pub market_id: u64,
    pub base_currency: CurrencyInfo,
    pub quote_currency: CurrencyInfo,
}

impl MarketInfo {
    pub fn get_currency_status(&self, target_denom: &str) -> Result<CurrencyStatus, ContractError> {
        if self.is_base_currency(target_denom) {
            return Ok(CurrencyStatus::BaseCurrency);
        }

        if self.is_quote_currency(target_denom) {
            return Ok(CurrencyStatus::QuoteCurrency);
        }

        return Err(ContractError::MismatchDenomAndMarket {});
    }

    pub fn is_valid_currency(&self, target_denom: &str) -> bool {
        return self.is_base_currency(target_denom) || self.is_quote_currency(target_denom);
    }

    pub fn is_quote_currency(&self, target_denom: &str) -> bool {
        return match &self.quote_currency {
            CurrencyInfo::Cw20 { address } => target_denom.eq(address),
            CurrencyInfo::Native { denom } => target_denom.eq(denom),
        };
    }

    pub fn is_base_currency(&self, target_denom: &str) -> bool {
        return match &self.base_currency {
            CurrencyInfo::Cw20 { address } => target_denom.eq(address),
            CurrencyInfo::Native { denom } => target_denom.eq(denom),
        };
    }
}

/// acts as a doubly linked list
#[cw_serde]
pub struct LevelData {
    /// id_previous level in linked list compared to midprice (so for bids higher price, for asks lower price)
    pub id_previous: Option<u64>, //Option<Vec<u8>>,
    /// id_next level in linked list compared to midprice (so for bids lower price, for asks higher price)
    pub id_next: Option<u64>, //Option<Vec<u8>>,
    pub price: Decimal,
}

impl LevelData {
    pub fn is_head(&self) -> bool {
        return self.id_previous.is_none();
    }

    pub fn is_tail(&self) -> bool {
        return self.id_next.is_none();
    }
}

/*
#[cw_serde]
pub struct UserOrder {
    pub market_id: u64,
    pub price: Decimal,
}
*/

/*
#[derive(Serialize, Deserialize, Debug)]
pub struct InternalUserOrder {
    pub market_id: u64,
    pub price: Decimal,
}
*/

/*
#[derive(Serialize, Deserialize, Debug)]
pub struct InternalUserOrder {
    pub order_side: OrderSide,
    pub price: Decimal,
    pub quantity: Uint128,
}
*/

#[cw_serde]
pub struct UserOrderRecord {
    pub market_id: u64,
    pub order_side: OrderSide,
    pub price: Decimal,
    pub quantity: Uint128,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TempOrderRep {
    pub order_side: OrderSide,
    pub price: Decimal,
    pub quantity: Uint128,
}

#[cw_serde]
pub struct UserOrder {
    pub market_id: u64,
    pub price: Decimal,
    pub quantity: Uint128,
}

#[cw_serde]
pub struct LevelOrder {
    pub user: Addr,
    pub amount: Uint128,
}

pub type LevelOrders = Vec<LevelOrder>;

#[cw_serde]
pub struct BookLevel {
    pub price: Decimal,
    pub quantity: Uint256,
}

impl BookLevel {
    pub fn new(price: Decimal) -> Self {
        return BookLevel {
            price: price,
            quantity: Uint256::zero(),
        };
    }
}
