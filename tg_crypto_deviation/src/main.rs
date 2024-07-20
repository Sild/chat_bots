extern crate core;

mod binance;
mod telegram;
mod db;
mod stream_processor;

use std::sync::Arc;
use ::binance::websockets::{WebsocketEvent};
use futures::future::{try_join_all};
use futures::FutureExt;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    if let Err(e) = run().await {
        log::error!("App finish with error: {:?}", e);
    }
}

async fn run() -> anyhow::Result<()> {
    log::info!("App started");
    let tg_cli = telegram::TGClient::new();
    // let bin_rest_cli = binance::BinanceRestClient::new();
    let binance_stream = binance::BinanceStream::new()?;
    let db = Arc::new(db::DB::new());
    let mut stream_processor = stream_processor::StreamProcessor::new(db, binance_stream);
    let futures = vec![
        tg_cli.run().boxed(),
        stream_processor.run().boxed(),
    ];
    try_join_all(futures).await?;
    Ok(())
}