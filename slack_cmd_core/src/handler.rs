use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub trait Handler {
    fn name(&self) -> String;
    fn short_description(&self) -> String;
    fn supported_channels(&self) -> Vec<String>;
    fn handle(&self, args: Vec<String>) -> Result<()>;
}

struct DefaultHelpHandler {
    name: String,
    short_description: String,
    supported_channels: Vec<String>,
    available_handlers: Vec<Arc<RwLock<dyn Handler>>>,
}

impl DefaultHelpHandler {
    pub fn new(handlers: Vec<Arc<RwLock<dyn Handler>>>) -> Self {
        DefaultHelpHandler {
            name: "help".to_string(),
            short_description: "Prints this help message".to_string(),
            supported_channels: vec![String::from("*")],
            available_handlers: handlers.clone(),
        }
    }
}

impl Handler for DefaultHelpHandler {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn short_description(&self) -> String {
        self.short_description.clone()
    }
    fn supported_channels(&self) -> Vec<String> {
        self.supported_channels.clone()
    }
    fn handle(&self, _args: Vec<String>) -> Result<()> {
        println!("{}: {}", self.name, self.short_description);
        // for handler in &self.available_handlers {
        //     let handler = handler.read().await;
        //     println!("{}: {}", handler.name(), handler.short_description());
        // }
        Ok(())
    }
}
