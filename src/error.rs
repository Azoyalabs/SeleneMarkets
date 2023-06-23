use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Never")]
    Never {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Only one fund allowed in native market messages")]
    InvalidNumberOfFunds {},

    #[error("Unknown market id: {id}")]
    UnknownMarketId { id: u64 },

    #[error("This is not a valid denomination for this market")]
    MismatchDenomAndMarket {},

    #[error("Order to cancel does not exist")]
    OrderDoesNotExist {},

    #[error("Not enough liquidity to execute market order")]
    NotEnoughLiquidityMarketOrder {},
}
