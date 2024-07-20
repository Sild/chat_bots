use binance::model::DayTickerEvent;
use binance::websockets::WebsocketEvent;
use crate::binance::BinanceStream;
use crate::db::ArcDB;

pub struct StreamProcessor {
    db: ArcDB,
    stream: BinanceStream
}

impl StreamProcessor {
    pub fn new(db: ArcDB, stream: BinanceStream) -> Self {
        Self {
            db,
            stream
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        while let Ok(event) = self.stream.next() {
            match event {
                // 24hr rolling window ticker statistics for all symbols that changed in an array.
                WebsocketEvent::DayTickerAll(ticker_events) => {
                    self.process_day_ticket_event(ticker_events).await?;
                },
                _ => (),
            };
        }
        log::info!("StreamProcessor finished.");
        Ok(())
    }

    async fn process_day_ticket_event(&self, ticker_events: Vec<DayTickerEvent>) -> anyhow::Result<()> {
        for tick_event in ticker_events {
            if tick_event.symbol != "BTCUSDT" {
                continue;
            }
            // println!("{} close: {}", tick_event.symbol, tick_event.current_close);
        }
        Ok(())
    }
}