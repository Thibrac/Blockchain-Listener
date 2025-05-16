#[macro_use]
extern crate rocket;

mod blockdata;
mod routes;

use blockdata::{CurrentBlockData, SharedBlockData};
use ethers::prelude::*;
use routes::{data, health};
use std::env;
use std::sync::{Arc, Mutex};

async fn eth_listener(share_data: SharedBlockData) -> eyre::Result<()> {
    let url = env::var("ALCHEMY_URL")?;
    let provider = Provider::<Ws>::connect(url).await?;
    let mut stream = provider.subscribe_blocks().await?;

    while let Some(block) = stream.next().await {
        let block_number = block.number.unwrap_or_default();
        if let Some(full_block) = provider.get_block_with_txs(block.hash.unwrap()).await? {
            let tx_count = full_block.transactions.len();
            let mut data = share_data.lock().unwrap();

            println!("Nouveau bloc : {block_number}, nb de tx = {}", tx_count);
            data.block_number = block_number.as_u64();
            data.tx_count = tx_count;
        }
    }

    Ok(())
}

#[rocket::main]
async fn main() -> eyre::Result<()> {
    dotenv::dotenv().ok();
    let current_block_data = Arc::new(Mutex::new(CurrentBlockData {
        block_number: 0,
        tx_count: 0,
    }));

    let share_data = Arc::clone(&current_block_data);
    tokio::spawn(async {
        if let Err(err) = eth_listener(share_data).await {
            eprintln!("❌ eth_listener error: {:?}", err);
        }
    });
    rocket::build()
        .manage(current_block_data)
        .mount("/health", routes![health])
        .mount("/data", routes![data])
        .launch()
        .await?;
    Ok(())
}
