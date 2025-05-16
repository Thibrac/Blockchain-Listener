use rocket::serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Clone, Serialize)]
pub struct CurrentBlockData {
    pub block_number: u64,
    pub tx_count: usize,
}
pub type SharedBlockData = Arc<Mutex<CurrentBlockData>>;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: &'static str,
}
