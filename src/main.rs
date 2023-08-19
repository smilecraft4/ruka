#![allow(unused)]

use ruka::{
    cli::{parse_command_args, Parameter},
    error::Result,
    metadata::Metadata,
};

#[tokio::main]
async fn main() -> Result<()> {
    let data = Metadata::from_json(String::from("dqsdqdqsd"))?;
    dbg!(data);

    Ok(())
}
