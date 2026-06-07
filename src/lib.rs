pub mod application;
pub mod benchmark;
pub mod domain;
pub mod infrastructure;
mod shared;

pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}
