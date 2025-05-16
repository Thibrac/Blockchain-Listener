use crate::{
    SharedBlockData,
    blockdata::{CurrentBlockData, HealthStatus},
};
use rocket::serde::{Serialize, json::Json};

#[get("/")]
pub fn data(share_data: &rocket::State<SharedBlockData>) -> Json<CurrentBlockData> {
    let data = share_data.lock().unwrap();

    Json(CurrentBlockData {
        block_number: data.block_number,
        tx_count: data.tx_count,
    })
}

#[get("/")]
pub fn health() -> Json<HealthStatus> {
    Json(HealthStatus { status: "Up" })
}
