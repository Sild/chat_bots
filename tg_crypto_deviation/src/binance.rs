use std::ops::Deref;
use std::sync::{Arc, mpsc};
use std::sync::atomic::AtomicBool;
use binance::websockets::{all_ticker_stream, WebSockets};
use binance::ws_model::WebsocketEvent;
use tokio::task::JoinHandle;
use tokio::time::sleep;

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
            let mut web_socket = WebSockets::new(|events: Vec<WebsocketEvent>| {
                for ev in events {
                    if let Err(e) = tx.send(ev) {
                        log::error!("Error while pushing event to queue: {:?}", e.to_string());
                    }
                }
                Ok(())
            });
            if let Err(e) = web_socket.connect(all_ticker_stream()).await {
                log::error!("Error connecting to websocket: {:?}", e);
                return;
            }
            log::info!("Running websocket event loop...");
            if let Err(e) = web_socket.event_loop(&keep_running).await {
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

    pub async fn next(&mut self) -> anyhow::Result<WebsocketEvent> {
        loop {
            match self.rx.recv_timeout(std::time::Duration::from_millis(10)) {
                Err(mpsc::RecvTimeoutError::Timeout) => sleep(std::time::Duration::from_millis(20)).await,
                res => return Ok(res?),
            }
        }

    }
}