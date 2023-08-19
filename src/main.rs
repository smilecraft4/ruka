#![allow(unused)]

use ruka::{
    cli::{parse_command_args, Parameter},
    error::Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_command_args();
    let param = Parameter::from_args(&args)?;

    dbg!(param);

    Ok(())
}
