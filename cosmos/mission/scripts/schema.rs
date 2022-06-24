use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use mission_contract::{
    msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
    state::State,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("cosmos/mission/schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    macro_rules! generate_schema {
        ($msg: tt) => {
            export_schema(&schema_for!($msg), &out_dir)
        };
    }

    generate_schema!(InstantiateMsg);
    generate_schema!(ExecuteMsg);
    generate_schema!(QueryMsg);
    generate_schema!(State);
    generate_schema!(CountResponse);
}
