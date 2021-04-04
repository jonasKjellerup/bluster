// See https://github.com/SSheldon/rust-objc/pull/75 for updates on issues to do with compiler
// warnings caused by `ATOMIC_USIZE_INIT` being deprecated
#![allow(deprecated)]

#[macro_use]
extern crate bitflags;

mod error;
mod common;
pub mod peripheral;

#[cfg(target_os = "linux")]
mod bluez;

pub mod gatt;
//mod uuid;

pub use self::{error::*, peripheral::Peripheral/*, uuid::* */};
