use env_logger::Builder;
use std::process;

#[macro_use]
extern crate log;

pub mod data;
pub mod error;

mod config;

pub use config::Config;

use error::Error;

pub const APP_NAME: &str = "webhook";

pub type Result<T> = std::result::Result<T, Error>;

pub fn init_logger() {
    Builder::new()
        .format_timestamp_secs()
        .filter_module(APP_NAME, log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .init();
}

pub fn abort(message: &str, err: Error) -> ! {
    error!("{} Reason: {}", message, err);
    process::exit(1)
}
