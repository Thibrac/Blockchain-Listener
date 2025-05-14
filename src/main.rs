#[macro_use]
extern crate rocket;
use ethers::prelude::*;
use std::env;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct BlockInfos {
    block_number: u64,
    tx_count: usize,
}

#[get("/data")]
fn data(share_data: &rocket::State<Arc<Mutex<BlockInfos>>>) -> String {
    let data = share_data.lock().unwrap();

    format!(
        "Dernier block : [{}] avec {}txs",
        data.block_number, data.tx_count
    )
}

async fn eth_listener(share_data: Arc<Mutex<BlockInfos>>) -> eyre::Result<()> {
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
    let data = Arc::new(Mutex::new(BlockInfos {
        block_number: 0,
        tx_count: 0,
    }));

    let share_data = Arc::clone(&data);
    tokio::spawn(async {
        if let Err(err) = eth_listener(share_data).await {
            eprintln!("❌ eth_listener error: {:?}", err);
        }
    });
    rocket::build()
        .manage(data)
        .mount("/", routes![data])
        .launch()
        .await?;
    Ok(())
}
