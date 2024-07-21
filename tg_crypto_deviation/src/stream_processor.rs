use crate::binance::BinanceStream;
use crate::db::ArcDB;
use binance::ws_model::{DayTickerEvent, WebsocketEvent};

pub struct StreamProcessor {
    db: ArcDB,
    stream: BinanceStream,
}

impl StreamProcessor {
    pub fn new(db: ArcDB, stream: BinanceStream) -> Self {
        Self { db, stream }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        while let Ok(event) = self.stream.next().await {
            match event {
                // 24hr rolling window ticker statistics for all symbols that changed in an array.
                WebsocketEvent::DayTicker(ticker_event) => {
                    self.process_day_ticket_event(ticker_event).await?;
                }
                _ => (),
            };
        }
        log::info!("StreamProcessor finished.");
        Ok(())
    }

    async fn process_day_ticket_event(&self, ticker_event: Box<DayTickerEvent>) -> anyhow::Result<()> {
        if ticker_event.symbol != "BTCUSDT" {
            return Ok(());
        }
        log::debug!("{} close: {}", ticker_event.symbol, ticker_event.current_close);
        Ok(())
    }
}
