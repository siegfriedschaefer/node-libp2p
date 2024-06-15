struct chatapp {
    
}

mod cli;
pub mod util;
pub mod protocol;

use serde::{Deserialize, Serialize};
use std::str::FromStr;


pub use ethers::types::{Address, U256};
pub use libp2p::PeerId;

pub use cli::{BootNode, TransportArgs, Network};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicConfig {
    pub mtu_discovery_max: u16,
    pub keep_alive_interval_ms: u32,
    pub max_idle_timeout_ms: u32,
}

fn parse_var<T: FromStr>(var: &str, default: T) -> T {
    std::env::var(var).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

impl QuicConfig {
    pub fn from_env() -> Self {
        let mtu_discovery_max = parse_var("MTU_DISCOVERY_MAX", 1452);
        let keep_alive_interval_ms = parse_var("KEEP_ALIVE_INTERVAL_MS", 5000);
        let max_idle_timeout_ms = parse_var("MAX_IDLE_TIMEOUT_MS", 60000);
        Self {
            mtu_discovery_max,
            keep_alive_interval_ms,
            max_idle_timeout_ms,
        }
    }
}