#![feature(arbitrary_self_types, decl_macro, proc_macro_hygiene)]

#![warn(rust_2018_idioms)]

mod configuration;
mod connection;
mod database;
pub mod error;
mod locked_connection;
mod result;
mod settings;

pub(crate) use configuration::DieselConfiguration as Configuration;
pub(crate) use settings::Settings;
pub(crate) use connection::Connection;
pub(crate) use locked_connection::LockedConnection;
pub use database::Database as Database;
pub use result::Result;