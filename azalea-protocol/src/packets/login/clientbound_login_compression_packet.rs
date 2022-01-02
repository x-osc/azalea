use std::hash::Hash;

use crate::mc_buf::{Readable, Writable};

use super::LoginPacket;

#[derive(Hash, Clone, Debug)]
pub struct ClientboundLoginCompressionPacket {
    pub compression_threshold: i32,
}

impl ClientboundLoginCompressionPacket {
    pub fn get(self) -> LoginPacket {
        LoginPacket::ClientboundLoginCompressionPacket(self)
    }

    pub fn write(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buf.write_varint(self.compression_threshold).unwrap();
        Ok(())
    }

    pub async fn read<T: tokio::io::AsyncRead + std::marker::Unpin + std::marker::Send>(
        buf: &mut T,
    ) -> Result<LoginPacket, String> {
        let compression_threshold = buf.read_varint().await?;

        Ok(ClientboundLoginCompressionPacket {
            compression_threshold,
        }
        .get())
    }
}