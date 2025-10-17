use anyhow::{Result, bail};
use background::App;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("Missing command argument ('update' or 'status')");
    }

    App::new()?.run(&args[1])?;

    Ok(())
}
