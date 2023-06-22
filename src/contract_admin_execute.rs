use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    msg::AdminExecuteMsg,
    state::{ADMIN, MARKET_ID_TRACKER, MARKET_INFO},
    structs::{CurrencyInfo, MarketInfo},
    ContractError,
};

pub fn route_admin_execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: AdminExecuteMsg,
) -> Result<Response, ContractError> {
    // validate caller is admin
    if !match ADMIN.load(deps.storage) {
        Ok(admin) => admin == info.sender,
        Err(_) => false,
    } {
        return Err(ContractError::Unauthorized {});
    }

    match msg {
        AdminExecuteMsg::UpdateAdmin { new_admin } => update_admin(deps, new_admin),
        AdminExecuteMsg::AddMarket {
            base_currency,
            quote_currency,
        } => add_market(deps, base_currency, quote_currency),
    }
}

fn update_admin(deps: DepsMut, new_admin: String) -> Result<Response, ContractError> {
    let new_admin = deps.api.addr_validate(&new_admin)?;

    ADMIN.update(deps.storage, |_| -> Result<_, ContractError> {
        return Ok(new_admin);
    })?;

    return Ok(Response::new());
}

pub fn add_market(
    deps: DepsMut,
    base_currency: CurrencyInfo,
    quote_currency: CurrencyInfo,
) -> Result<Response, ContractError> {
    let curr_id = MARKET_ID_TRACKER.load(deps.storage).unwrap_or_default();
    MARKET_INFO.save(
        deps.storage,
        curr_id,
        &MarketInfo {
            base_currency: base_currency,
            quote_currency: quote_currency,
            top_level_bid: None,
            top_level_ask: None,
        },
    )?;

    MARKET_ID_TRACKER.save(deps.storage, &(curr_id + 1))?;

    return Ok(Response::new());
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;

    use crate::structs::CurrencyInfo;

    use super::add_market;

    #[test]
    fn admin_add_market() {
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
    }
}
