#![no_std]

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "std")]
extern crate std;

pub mod errors;
pub mod instructions;
pub mod states;

pinocchio_pubkey::declare_id!("GbqefpNQgSDkGj3Yv3zdtUiVD9qgZo6LGw3ZTeBJgbWP");