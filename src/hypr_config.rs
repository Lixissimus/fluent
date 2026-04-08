use std::sync::Arc;

use hyprlang::{Config, ConfigError, Handler, HandlerContext, ParseResult};

#[derive(Debug, Default)]
struct MapHandler {
    mappings: Arc<Vec<String>>,
}

impl Handler for MapHandler {
    fn handle(&self, context: &HandlerContext) -> ParseResult<()> {
        println!("{}", context.value);
        if context.value.split(",").count() != 2 {
            return Err(ConfigError::HandlerError {
                handler: self.name().into(),
                message: format!("expected exact 2 parts, got {}", context.value),
            });
        }
        // TODO: does the handler trait bring any value compared to a function handler?
        // NEXT: implement a struct for the config values, parse in handle function to produce error, then create config struct later?
        // self.mappings.push(context.value.clone());
        Ok(())
    }

    fn name(&self) -> &str {
        "map handler"
    }
}

#[test]
fn load() {
    let mut config = Config::new();
    let map_handler = MapHandler::default();

    config.register_handler("map", map_handler);

    config.parse_file("fluent.conf").unwrap();
    let maps = config.get_handler_calls("map").unwrap();
    assert_eq!(maps.len(), 22);
}
