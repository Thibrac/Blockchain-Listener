#[macro_use]
extern crate rocket;

use reqwest;
use rocket::serde::Deserialize;
use std::result::Result;

#[derive(Deserialize, Debug)]
struct BlockData {
    block_number: u64,
    tx_count: u64,
}

#[get("/")]
async fn index() -> Result<String, rocket::response::Debug<reqwest::Error>> {
    let url = "http://localhost:8000/data";
    let res = reqwest::get(url).await?;
    let block: BlockData = res.json().await?;
    Ok(format!(
        "it works :\nBlockNumber = {}, Nombre de txs = {}",
        block.block_number, block.tx_count
    ))
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    rocket::build().mount("/", routes![index]).launch().await?;
    Ok(())
}
