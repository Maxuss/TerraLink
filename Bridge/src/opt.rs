use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeOptions {
    #[serde(default = "default_threshold")]
    pub packet_bounds: usize
}

fn default_threshold() -> usize {
    return 16
}