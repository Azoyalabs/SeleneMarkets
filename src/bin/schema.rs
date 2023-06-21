use std::{env, path::Path};

use cosmwasm_schema::{
    export_schema, export_schema_with_title, generate_api, schema_for, write_api,
};

use selene_markets::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SeleneCw20Msg};

fn main() {
    let schema = schema_for!(SeleneCw20Msg);

    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    };

    let mut out_dir = env::current_dir().unwrap();
    out_dir.push("schema");
    export_schema_with_title(&schema, Path::new(out_dir.as_path()), "selene_cw20_msg");
}
