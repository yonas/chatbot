#![feature(try_blocks)]

use chat_rooms::app::App;
use loco_rs::cli;
use migration::Migrator;
use color_eyre::eyre::{Result, ErrReport};

#[tokio::main]
async fn main() -> Result<(), ErrReport> {
    color_eyre::install()?;

    match cli::main::<App, Migrator>().await {
        Ok(_) => { Ok(()) }
        Err(e) => { Err(ErrReport::new(e)) }
    }
}
