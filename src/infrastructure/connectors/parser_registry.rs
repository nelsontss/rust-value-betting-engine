use std::collections::HashMap;

use serde_json::Value;

use crate::domain::entities::Platform;
use crate::domain::Game;
use crate::infrastructure::connectors::betano_connector::BetanoParser;

pub trait DataParser: Send {
    fn parse(&self, data: Value) -> Vec<Game>;
}

impl DataParser for BetanoParser {
    fn parse(&self, data: Value) -> Vec<Game> {
        BetanoParser::parse_data(data)
    }
}

pub struct ParserRegistry {
    parsers: HashMap<Platform, Box<dyn DataParser>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        let mut registry = ParserRegistry {
            parsers: HashMap::new(),
        };
        registry.register(Platform::Betano, Box::new(BetanoParser::new()));
        registry
    }

    pub fn register(&mut self, platform: Platform, parser: Box<dyn DataParser>) {
        self.parsers.insert(platform, parser);
    }

    pub fn parse(&self, platform: &Platform, data: Value) -> Option<Vec<Game>> {
        self.parsers.get(platform).map(|parser| parser.parse(data))
    }
}
