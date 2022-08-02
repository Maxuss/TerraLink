use std::io::Cursor;
use anyhow::bail;
use anyhow::Ok;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

macro_rules! packets {
    ($(
        $name:ident($id:literal) {
            $(
            $field:ident: $ty:ident
            ),* $(,)*
        }
    );* $(;)?) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub enum ProtocolPacket {
            $(
            $name($name)
            ),*
        }

        #[async_trait::async_trait]
        impl Packet for ProtocolPacket {
            async fn read(bytes: &mut Cursor<Vec<u8>>) -> anyhow::Result<Self> {
                match bytes.read_u8().await? {
                    $(
                    $id => Ok(ProtocolPacket::$name($name::read(bytes).await?)),
                    )*
                    other => {
                        log::warn!("Invalid packet ID received: {}!", other);
                        bail!("Invalid packet ID!")
                    }
                }
            }

            async fn write(&self, buf: &mut Vec<u8>) -> anyhow::Result<()> {
                match &self {
                    $(
                    ProtocolPacket::$name(packet) => {
                        ($id as u8).write(buf).await?;
                        packet.write(buf).await?;
                    }
                    ),*
                }
                Ok(())
            }
        }

        $(
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            $(
            pub $field:$ty
            ),*
        }

        #[async_trait::async_trait]
        impl Packet for $name {
            async fn read(bytes: &mut Cursor<Vec<u8>>) -> anyhow::Result<Self> {
                $(
                let $field = $ty::read(bytes).await?;
                )*
                Ok(Self {
                    $(
                    $field
                    )*
                })
            }

            async fn write(&self, buf: &mut Vec<u8>) -> anyhow::Result<()> {
                $(
                self.$field.write(buf).await?;
                )*
                Ok(())
            }
        }
        )*
    };
}

macro_rules! primitive_impl {
    ($($prim:ident => ($i:ident, $o:ident)),* $(,)?) => {
        $(
            #[async_trait::async_trait]
            impl Packet for $prim {
                async fn read(bytes: &mut Cursor<Vec<u8>>) -> anyhow::Result<Self> {
                    bytes.$i().await.map_err(anyhow::Error::from)
                }

                async fn write(&self, buf: &mut Vec<u8>) -> anyhow::Result<()> {
                    buf.$o(*self).await.map_err(anyhow::Error::from)
                }
            }
        )*
    };
}

#[async_trait::async_trait]
pub trait Packet {
    async fn read(bytes: &mut Cursor<Vec<u8>>) -> anyhow::Result<Self> where Self: Sized;
    async fn write(&self, buf: &mut Vec<u8>) -> anyhow::Result<()>;
}

primitive_impl! {
    u8 => (read_u8, write_u8),
    i8 => (read_i8, write_i8),
    i16 => (read_i16, write_i16),
    i32 => (read_i32, write_i32),
    i64 => (read_i64, write_i64),

    u16 => (read_u16, write_u16),
    u32 => (read_u32, write_u32),
    u64 => (read_u64, write_u64),

    f64 => (read_f64, write_f64),
    f32 => (read_f32, write_f32),
}

#[async_trait::async_trait]
impl Packet for bool {
    async fn read(bytes: &mut Cursor<Vec<u8>>) -> anyhow::Result<Self> where Self: Sized {
        Ok(bytes.read_u8().await? == 1)
    }

    async fn write(&self, buf: &mut Vec<u8>) -> anyhow::Result<()> {
        buf.write_u8(if *self { 0x01 } else { 0x00 }).await?;
        Ok(())
    }
}

const MAX_STR_LEN: usize = i16::MAX as usize;

#[async_trait::async_trait]
impl Packet for String {
    async fn read(bytes: &mut Cursor<Vec<u8>>) -> anyhow::Result<Self> where Self: Sized {
        let size = i32::read(bytes).await? as usize;
        if size > MAX_STR_LEN {
            log::warn!("String buffer overflow!");
            bail!("String buffer overflow!");
        }
        let mut buf = vec![0u8; size];
        bytes.read_exact(&mut buf).await?;
        return String::from_utf8(buf).map_err(anyhow::Error::from)
    }

    async fn write(&self, buf: &mut Vec<u8>) -> anyhow::Result<()> {
        let size = self.len();
        if size > MAX_STR_LEN {
            log::warn!("Tried to write string of size over {}!", MAX_STR_LEN);
            bail!("String too long!")
        }
        (size as i32).write(buf).await?;
        buf.extend_from_slice(self.as_bytes());
        Ok(())
    }
}

packets! {
    Connect(0x00) {
        brand: String
    };
    Disconnect(0x01) {
        reason: String
    };
    AdvanceState(0x02) {
        bridge_info: String
    };
}