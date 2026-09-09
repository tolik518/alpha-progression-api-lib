/// Reads a value extracted from the app bundle out of the build environment.
/// Supplied by `build.rs` from `.env`, or by a real environment variable.
macro_rules! app_secret {
    ($name:literal) => {
        env!(
            $name,
            concat!(
                $name,
                " is not set — copy .env.example to .env and fill it in ",
                "(see docs/extracting-api-keys.md)"
            )
        )
    };
}

mod auth;
mod api;
mod cloudant;
mod crypto;

pub mod error;
pub mod models;
pub mod client;

pub use client::AlphaProgressionClient;
pub use error::Error;
pub use models::*;
