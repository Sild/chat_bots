use std::ops::Deref;
use std::sync::{Arc, mpsc};
use std::sync::atomic::AtomicBool;
use std::thread::sleep;
use binance::websockets::{WebsocketEvent, WebSockets};
use tokio::task::JoinHandle;

pub struct BinanceRestClient {

}

impl BinanceRestClient {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct BinanceStream {
    rx: mpsc::Receiver<WebsocketEvent>,
}

unsafe impl Sync for BinanceStream {}

impl BinanceStream {
    pub fn new() -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel();

        tokio::spawn(async move {
            let keep_running = AtomicBool::new(true);
            let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
                if let Err(e) = tx.send(event) {
                    log::error!("Error sending event: {:?}", e.to_string());
                }
                Ok(())
            });
            if let Err(e) = web_socket.connect(&"!ticker@arr".to_string()) {
                log::error!("Error connecting to websocket: {:?}", e);
                return;
            }
            log::info!("Running websocket event loop...");
            if let Err(e) = web_socket.event_loop(&keep_running) {
                log::error!("Error in event loop: {:?}", e);
                return
            }
            log::info!("Event loop finished.");
        });

        let stream = Self {
            rx,
        };
        Ok(stream)
    }

    pub fn next(&mut self) -> anyhow::Result<WebsocketEvent> {
        Ok(self.rx.recv()?)
    }
}