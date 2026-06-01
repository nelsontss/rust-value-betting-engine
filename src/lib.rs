use crate::application::services::bookmaker_scrapper_service::BookmakerScrapperService;

pub mod application;
pub mod benchmark;
pub mod domain;
pub mod infrastructure;
pub mod shared;

pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

pub fn run() {
    let mut bookmaker_scrapper_service = BookmakerScrapperService::new();

    bookmaker_scrapper_service.run();

    println!("{} is ready.", crate_name());
}
