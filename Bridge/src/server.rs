use std::sync::Arc;
use log::info;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use crate::BridgeOptions;
use crate::client::{ClientType, EarlyStageClient};
use crate::proto::{AdvanceState, Disconnect, ProtocolPacket};

pub async fn bind(opts: BridgeOptions) -> anyhow::Result<()> {
    let server = TcpListener::bind("127.0.0.1:25535").await?;
    info!("Bridge Server started on 127.0.0.1:25535");

    let mut minecraft_write: Option<Arc<Mutex<OwnedWriteHalf>>> = None;
    let mut terraria_write: Option<Arc<Mutex<OwnedWriteHalf>>> = None;

    let bridge_info = "TerraLink Bridge/0.1.0".to_string();

    loop {
        if let Ok((stream, addr)) = server.accept().await {
            info!("Receiving connection from client at {}!", addr);
            let mut client = EarlyStageClient::new(stream, addr, opts.clone());
            let ty = client.do_handshake().await;
            if let Err(err) = ty {
                client.send_packet(ProtocolPacket::Disconnect(Disconnect {
                    reason: format!("Invalid connection sequence: {}", err)
                })).await?;
                continue
            }
            match ty? {
                ClientType::Terraria => {
                    if let Some(_) = &terraria_write {
                        client.send_packet(ProtocolPacket::Disconnect(Disconnect {
                            reason: "Terraria client already connected!".to_string()
                        })).await?;
                        continue;
                    } else {
                        client.send_packet(ProtocolPacket::AdvanceState(AdvanceState {
                            bridge_info: bridge_info.clone()
                        })).await?;
                        terraria_write = Some(client.writer.clone());
                    }
                    if let Some(mw) = minecraft_write.clone() {
                        client.into_terraria(mw).begin_transmission()?;
                    }
                }
                ClientType::Minecraft => {
                    if let Some(_) = &minecraft_write {
                        client.send_packet(ProtocolPacket::Disconnect(Disconnect {
                            reason: "Minecraft client already connected!".to_string()
                        })).await?;
                        continue;
                    } else {
                        client.send_packet(ProtocolPacket::AdvanceState(AdvanceState {
                            bridge_info: bridge_info.clone()
                        })).await?;
                        minecraft_write = Some(client.writer.clone());
                    }
                    if let Some(tw) = terraria_write.clone() {
                        client.into_minecraft(tw).begin_transmission()?;
                    }
                }
            }
        }
    }
}