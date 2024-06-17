
use libp2p::{
    identity::{ed25519, Keypair},
    multiaddr::Protocol,
    Multiaddr,
};
use std::path::PathBuf;


pub async fn get_keypair(path: Option<PathBuf>) -> anyhow::Result<Keypair> {
    let Some(path) = path else {
        return Ok(Keypair::generate_ed25519());
    };
    match tokio::fs::metadata(&path).await {
        Ok(meta) if meta.is_file() => {
            log::info!("Reading key from {}", path.display());
            let mut content = tokio::fs::read(&path).await?;
            let keypair = ed25519::Keypair::try_from_bytes(content.as_mut_slice())?;
            Ok(keypair.into())
        }
        Ok(_) => {
            anyhow::bail!("Path exists and is not a file")
        }
        Err(_) => {
            log::info!("Generating new key and saving into {}", path.display());
            let keypair = ed25519::Keypair::generate();
            tokio::fs::write(&path, keypair.to_bytes()).await?;
            Ok(keypair.into())
        }
    }
}

pub fn addr_is_reachable(addr: &Multiaddr) -> bool {
    match addr.iter().next() {
        Some(Protocol::Ip4(addr)) => {
            !(addr.is_loopback() || addr.is_link_local())
            // We need to allow private addresses for testing in local environment
            &&(!addr.is_private() || std::env::var("PRIVATE_NETWORK").is_ok())
        }
        Some(Protocol::Ip6(addr)) => !addr.is_loopback(),
        Some(Protocol::Dns(_) | Protocol::Dns4(_) | Protocol::Dns6(_) | Protocol::Dnsaddr(_)) => {
            true
        }
        _ => false,
    }
}
