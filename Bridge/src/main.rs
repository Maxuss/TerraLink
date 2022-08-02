use log::info;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::opt::BridgeOptions;
use crate::server::bind;

pub mod proto;
pub mod server;
pub mod opt;
pub mod client;

#[tokio::main]
async fn main() {
    let opts = File::open("terralink.toml").await;
    if let Err(_) = opts {
        println!("Looks like it's your first time using TerraLink bridge! Options file was created in terralink.toml");
        println!("Restart the server to apply changes!");

        let mut file = File::create("terralink.toml").await.unwrap();
        file.write_all(&toml::ser::to_string(&BridgeOptions::default()).unwrap().into_bytes()).await.unwrap();
        return;
    }
    let mut strbuf = String::new();
    opts.unwrap().read_to_string(&mut strbuf).await.unwrap();
    let opts: BridgeOptions = toml::de::from_str(&strbuf).unwrap();

    info!("Starting bridge...");
    bind(opts).await.unwrap()
}
