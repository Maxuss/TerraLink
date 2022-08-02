use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeOptions {
    #[serde(default = "default_threshold")]
    pub packet_bounds: usize
}

impl Default for BridgeOptions {
    fn default() -> Self {
        Self {
            packet_bounds: default_threshold()
        }
    }
}

fn default_threshold() -> usize {
    return 16
}