use std::{
    cmp::Ordering,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use cosmwasm_std::{Coin, Decimal, MessageInfo};

use crate::{
    structs::{CurrencyStatus, MarketInfo},
    ContractError,
};

/// check if there is only one fund in the message and return the coin struct
pub fn check_only_one_fund(info: &MessageInfo) -> Result<Coin, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidNumberOfFunds {});
    } else {
        return Ok(info.funds[0].to_owned());
    }
}

/// Create an id for a level
pub fn create_id_level(
    market_info: &MarketInfo,
    price_level: Decimal,
    currency_status: CurrencyStatus,
) -> u64 {
    let mut s = DefaultHasher::new();
    market_info.base_currency.hash(&mut s);
    market_info.quote_currency.hash(&mut s);
    price_level.to_string().hash(&mut s);
    currency_status.hash(&mut s);

    return s.finish();
}

/// Create an id for a level
pub fn create_id_level_no_status(market_info: &MarketInfo, price_level: Decimal) -> u64 {
    let mut s = DefaultHasher::new();
    market_info.base_currency.hash(&mut s);
    market_info.quote_currency.hash(&mut s);
    price_level.to_string().hash(&mut s);

    return s.finish();
}

/// Wrapping ordering comparisons to avoid code repetition
/// Compares first to second (elem_1.cmp(&elem_2))
/// Examples with target_ordering == Ordering::Higher
/// wrapped_comparison(5, 7, target_ordering) == False
/// wrapped_comparison(7, 5, target_ordering) == False
pub fn wrapped_comparison<T>(elem_1: T, elem_2: T, target_ordering: Ordering) -> bool
where
    T: Ord,
{
    return elem_1.cmp(&elem_2) == target_ordering;
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::wrapped_comparison;

    #[test]
    fn utils_test_wrapped_comparison() {
        let target_ordering = Ordering::Greater;

        assert!(!wrapped_comparison(0u64, 12u64, target_ordering));
        assert!(wrapped_comparison(14u64, 12u64, target_ordering));
        assert!(!wrapped_comparison(12u64, 12u64, target_ordering));

        let target_ordering = Ordering::Equal;
        assert!(!wrapped_comparison(0u64, 12u64, target_ordering));
        assert!(!wrapped_comparison(14u64, 12u64, target_ordering));
        assert!(wrapped_comparison(12u64, 12u64, target_ordering));

        let target_ordering = Ordering::Less;
        assert!(wrapped_comparison(0u64, 12u64, target_ordering));
        assert!(!wrapped_comparison(14u64, 12u64, target_ordering));
        assert!(!wrapped_comparison(12u64, 12u64, target_ordering));
    }
}