pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod shared;

pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

pub fn run() {
    println!("{} is ready.", crate_name());
}
