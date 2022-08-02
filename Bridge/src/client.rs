use std::borrow::BorrowMut;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use anyhow::bail;
use flume::{Receiver, Sender};
use log::{debug, error, info, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::timeout;
use crate::BridgeOptions;
use crate::proto::{ProtocolPacket, Packet};

pub struct EarlyStageClient {
    pub writer: Arc<Mutex<OwnedWriteHalf>>,
    reader: OwnedReadHalf,
    addr: SocketAddr,
    opts: BridgeOptions
}

impl EarlyStageClient {
    pub fn new(stream: TcpStream, addr: SocketAddr, opts: BridgeOptions) -> Self {
        let (reader, writer) = stream.into_split();

        Self {
            writer: Arc::new(Mutex::new(writer)),
            reader,
            addr,
            opts
        }
    }

    pub async fn read_packet(&mut self) -> anyhow::Result<ProtocolPacket> {
        let mut buffer = Vec::new();
        let amount_read = timeout(Duration::from_secs(5), self.reader.read(&mut buffer)).await??;
        if amount_read == 0 {
            error!("Client sent 0 bytes!");
            bail!("Read 0 bytes!")
        }
        let mut cursor = Cursor::new(buffer);
        ProtocolPacket::read(&mut cursor).await
    }

    pub async fn send_packet(&mut self, packet: ProtocolPacket) -> anyhow::Result<()> {
        let mut buffer = Vec::new();
        packet.write(&mut buffer).await?;
        let reference = self.writer.borrow_mut();
        reference.lock().await.write_all(&mut buffer).await?;

        Ok(())
    }

    pub async fn do_handshake(&mut self) -> anyhow::Result<ClientType> {
        match self.read_packet().await? {
            ProtocolPacket::Connect(connect) => {
                let split: Vec<&str> = connect.brand.split("/").collect();
                if connect.brand.starts_with("TModLoader") {
                    // terraria client
                    let tmod_version = split[1];
                    info!("Client {} connected with tModLoader version {}", self.addr, tmod_version);
                    return Ok(ClientType::Terraria)
                } else if connect.brand.starts_with("Minecraft") {
                    // minecraft client
                    let (loader, version) = (split[1], split[2]);
                    info!("Client {} connected with {} minecraft version {}", self.addr, loader, version);
                    return Ok(ClientType::Minecraft)
                } else {
                    warn!("Client {} uses unknown/unsupported brand: {}", self.addr, connect.brand);
                    bail!("Invalid brand!")
                }
            },
            other => {
                warn!("Client {} sent packet {:?} when expected ConnectRequest packet!", self.addr, other);
                bail!("Invalid packet!")
            }
        }
    }

    pub fn into_minecraft(self, terraria_writer: Arc<Mutex<OwnedWriteHalf>>) -> MinecraftClient {
        MinecraftClient::new(terraria_writer, self.reader, self.addr, self.opts)
    }

    pub fn into_terraria(self, minecraft_writer: Arc<Mutex<OwnedWriteHalf>>) -> TerrariaClient {
        TerrariaClient::new(minecraft_writer, self.reader, self.addr, self.opts)
    }
}

pub struct TransmittingClient<T> {
    transmitter: T,
    writer: PacketWriter,
    reader: PacketReader,
    addr: SocketAddr,
}

impl<T> TransmittingClient<T> where T: Transmitter + Send + Sync {
    pub fn new(writer: Arc<Mutex<OwnedWriteHalf>>, reader: OwnedReadHalf, addr: SocketAddr, opts: BridgeOptions) -> Self {
        let (from_tx, from_rx) = flume::bounded(opts.packet_bounds);
        let (to_tx, to_rx) = flume::bounded(opts.packet_bounds);

        Self {
            transmitter: T::new(to_tx, from_rx),
            writer: PacketWriter::new(writer, to_rx),
            reader: PacketReader::new(reader, from_tx),
            addr
        }
    }

    pub async fn send_packet(&mut self, packet: ProtocolPacket) -> anyhow::Result<()> {
        self.writer.write_packet(packet).await
    }

    pub async fn read_packet(&mut self) -> anyhow::Result<ProtocolPacket> {
        self.reader.read_packet().await
    }

    pub fn begin_transmission(mut self) -> anyhow::Result<()> {
        info!("Entering main translation loop for client {}", self.addr);
        tokio::task::spawn(async move {
            self.reader.do_constant_read().await
        });
        tokio::task::spawn(async move {
            self.writer.do_constant_write().await
        });
        Ok(())
    }
}

pub type MinecraftClient = TransmittingClient<TransmitterMT>;
pub type TerrariaClient = TransmittingClient<TransmitterTM>;

#[async_trait::async_trait]
pub trait Transmitter {
    fn new(to_tx: Sender<ProtocolPacket>, from_rx: Receiver<ProtocolPacket>) -> Self;
    fn process(&self, packet: ProtocolPacket) -> anyhow::Result<ProtocolPacket>;
    async fn send(&mut self, packet: ProtocolPacket) -> anyhow::Result<()>;
    async fn recv(&mut self) -> anyhow::Result<ProtocolPacket>;
}

/// A minecraft -> terraria packet transmitter
pub struct TransmitterMT {
    terraria_tx: Sender<ProtocolPacket>,
    minecraft_rx: Receiver<ProtocolPacket>
}

/// A terraria -> minecraft packet transmitter
pub struct TransmitterTM {
    minecraft_tx: Sender<ProtocolPacket>,
    terraria_rx: Receiver<ProtocolPacket>
}

#[async_trait::async_trait]
impl Transmitter for TransmitterMT {
    fn new(to_tx: Sender<ProtocolPacket>, from_rx: Receiver<ProtocolPacket>) -> Self {
        Self {
            terraria_tx: to_tx,
            minecraft_rx: from_rx
        }
    }

    fn process(&self, packet: ProtocolPacket) -> anyhow::Result<ProtocolPacket> {
        debug!("M -> T: Processing packet {:?}", packet);
        Ok(packet)
    }

    async fn send(&mut self, packet: ProtocolPacket) -> anyhow::Result<()> {
        self.terraria_tx.send_async(self.process(packet)?).await?;
        Ok(())
    }

    async fn recv(&mut self) -> anyhow::Result<ProtocolPacket> {
        self.minecraft_rx.recv_async().await.map_err(anyhow::Error::from)
    }
}

#[async_trait::async_trait]
impl Transmitter for TransmitterTM {
    fn new(to_tx: Sender<ProtocolPacket>, from_rx: Receiver<ProtocolPacket>) -> Self {
        Self {
            minecraft_tx: to_tx,
            terraria_rx: from_rx
        }
    }

    fn process(&self, packet: ProtocolPacket) -> anyhow::Result<ProtocolPacket> {
        debug!("T -> M: Processing packet {:?}", packet);
        Ok(packet)
    }

    async fn send(&mut self, packet: ProtocolPacket) -> anyhow::Result<()> {
        self.minecraft_tx.send_async(self.process(packet)?).await?;
        Ok(())
    }

    async fn recv(&mut self) -> anyhow::Result<ProtocolPacket> {
        self.terraria_rx.recv_async().await.map_err(anyhow::Error::from)
    }
}


struct PacketReader {
    reader: OwnedReadHalf,
    sender: Sender<ProtocolPacket>
}

impl PacketReader {
    pub fn new(reader: OwnedReadHalf, sender: Sender<ProtocolPacket>) -> Self {
        Self {
            reader,
            sender
        }
    }

    async fn read_packet(&mut self) -> anyhow::Result<ProtocolPacket> {
        let mut buffer = Vec::new();
        let amount_read = timeout(Duration::from_secs(6), self.reader.read(&mut buffer)).await??;
        if amount_read == 0 {
            error!("Client sent 0 bytes!");
            bail!("Read 0 bytes!")
        }
        let mut cursor = Cursor::new(buffer);
        ProtocolPacket::read(&mut cursor).await
    }

    pub async fn do_constant_read(&mut self) -> anyhow::Result<()> {
        while let Ok(packet) = self.read_packet().await {
            self.sender.send_async(packet).await?;
        }
        bail!("Could not read packet! Queue empty!")
    }
}

struct PacketWriter {
    writer: Arc<Mutex<OwnedWriteHalf>>,
    receiver: Receiver<ProtocolPacket>
}

impl PacketWriter {
    pub fn new(writer: Arc<Mutex<OwnedWriteHalf>>, receiver: Receiver<ProtocolPacket>) -> Self {
        Self {
            writer,
            receiver
        }
    }

    async fn write_packet(&mut self, packet: ProtocolPacket) -> anyhow::Result<()> {
        let mut buffer = Vec::new();
        packet.write(&mut buffer).await?;
        self.writer.lock().await.write_all(&mut buffer).await?;

        Ok(())
    }

    pub async fn do_constant_write(&mut self) -> anyhow::Result<()> {
        while let Ok(packet) = self.receiver.recv_async().await {
            self.write_packet(packet).await?
        }
        bail!("Could not write packet! Queue empty!")
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ClientType {
    Terraria,
    Minecraft
}