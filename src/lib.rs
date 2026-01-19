use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Heartbeat(pub i32);

#[derive(Serialize, Deserialize)]
pub struct Stats {
    pub connected: u32,
    pub seed: i32,
}
