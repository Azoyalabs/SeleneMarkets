pub mod liquidity_consumer;
pub mod liquidity_provider;
pub mod liquidity_remover;

/*
fn compute_price(price: Uint128, consumption_results: &Vec<ConsumptionResult>) {
    let mut avg = Decimal256::zero();
    let mut total_amount: Uint256 = Uint256::zero();
    let price = Decimal256::new(price.into());
    for rslt in consumption_results {
        // do a sum at price ?
        for elem in &rslt.bin_records_consumed {
            //avg += elem.amount
            let amount = Decimal256::new(elem.amount.into());
            total_amount += elem.amount.into();
            avg += price.checked_mul(amount).unwrap();
        }
    }
}
*/
