#[macro_use]
extern crate rocket;

mod blockdata;
mod routes;

use blockdata::{CurrentBlockData, SharedBlockData};
use ethers::prelude::*;
use routes::{data, health};
use std::env;
use std::sync::{Arc, Mutex};

async fn blocks_listener(
    share_data: SharedBlockData,
    provider: Arc<Provider<Ws>>,
) -> eyre::Result<()> {
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

async fn events_listener(provider: Arc<Provider<Ws>>) -> eyre::Result<()> {
    let filter = Filter::new()
        .address(
            "0xdd6D76262Fd7BdDe428dcfCd94386EbAe0151603"
                .parse::<Address>()
                .unwrap(),
        )
        .event("OpPoked");
    let mut st2 = provider.subscribe_logs(&filter).await?;

    while let Some(log) = st2.next().await {
        println!("New log: {:?}", log);
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
    let url = env::var("ALCHEMY_URL")?;
    let provider = Arc::new(Provider::<Ws>::connect(url).await?);
    let p1 = Arc::clone(&provider);
    let p2 = Arc::clone(&provider);
    tokio::spawn(async {
        if let Err(err) = blocks_listener(share_data, p1).await {
            eprintln!("❌ blocks_listener error: {:?}", err);
        }
    });

    tokio::spawn(async {
        if let Err(err) = events_listener(p2).await {
            eprintln!("❌ events_listener error: {:?}", err);
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
