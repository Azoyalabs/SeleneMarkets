use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use erased_serde::Serialize;

use crate::{
    msg::{GetAdminResponse, QueryMsg},
    state::ADMIN,
};

pub fn route_query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let res: Box<dyn Serialize> = match msg {
        QueryMsg::GetAdmin {} => get_admin(deps),
        _ => panic!("Not implemented"),
    };

    return Ok(to_binary(&res)?);
}

fn get_admin(deps: Deps) -> Box<dyn Serialize> {
    return Box::new(GetAdminResponse {
        admin: ADMIN.load(deps.storage).ok(),
    });
}
