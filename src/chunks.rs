use crate::error::PngError;
use crc::{Crc, CRC_32_ISO_HDLC};
use std::io::{Seek, SeekFrom, Write};

const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

pub struct ChunkWriter;

impl ChunkWriter {
    pub fn write_chunk<W: Write + Seek>(
        writer: &mut W,
        chunk_type: &[u8; 4],
        data: &[u8],
    ) -> Result<(), PngError> {
        // Write length (big-endian)
        let length = data.len() as u32;
        writer.write_all(&length.to_be_bytes())?;

        // Write chunk type
        writer.write_all(chunk_type)?;

        // Write data
        writer.write_all(data)?;

        // Calculate CRC
        let crc_data: Vec<u8> = chunk_type.iter().chain(data.iter()).cloned().collect();
        let crc = CRC32.checksum(&crc_data);

        // Write CRC
        writer.write_all(&crc.to_be_bytes())?;

        Ok(())
    }
}
