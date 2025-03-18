use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OpusBox {
    pub data_reference_index: u16,
    pub channel_count: u16,
    pub sample_size: u16,

    #[serde(with = "value_u32")]
    pub sample_rate: FixedPointU16,
    pub dops_box: Option<DopsBox>,
}

impl Default for OpusBox {
    fn default() -> Self {
        Self {
            data_reference_index: 0,
            channel_count: 2,
            sample_size: 16,
            sample_rate: FixedPointU16::new(48000),
            dops_box: Some(DopsBox::default()),
        }
    }
}

impl OpusBox {

    pub fn new(config: &OpusConfig) -> Self {
        Self {
            channel_count: config.chan_conf as u16,
            sample_size: 16,
            sample_rate: FixedPointU16::new(config.freq_index.freq() as u16),
            dops_box: Some(DopsBox::new(config)),
            data_reference_index: 1,
        }
    }

    pub fn get_type(&self) -> BoxType {
        BoxType::OpusBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + 28;
        if let Some(ref dops_box) = self.dops_box {
            size += dops_box.box_size();
        }
        size
    }
}

impl Mp4Box for OpusBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "channel_count={} sample_size={} sample_rate={}",
            self.channel_count,
            self.sample_size,
            self.sample_rate.value()
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for OpusBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        reader.read_u32::<BigEndian>()?; // reserved
        reader.read_u16::<BigEndian>()?; // reserved
        let data_reference_index = reader.read_u16::<BigEndian>()?;
        reader.read_u16::<BigEndian>()?; // reserved
        reader.read_u16::<BigEndian>()?; // reserved
        reader.read_u32::<BigEndian>()?; // reserved
        let channel_count = reader.read_u16::<BigEndian>()?;
        let sample_size = reader.read_u16::<BigEndian>()?;
        reader.read_u32::<BigEndian>()?; // pre-defined, reserved
        let sample_rate = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);

        let header = BoxHeader::read(reader)?;
        let BoxHeader { name, size: s } = header;

        if s > size {
            return Err(Error::InvalidData(
                "opus box contains a box with a larger size than it",
            ));
        }
        let mut dops_box = None;
        if name == BoxType::DopsBox {
            dops_box = Some(DopsBox::read_box(reader, s)?);
        }
        skip_bytes_to(reader, start + size)?;
        Ok(OpusBox {
            data_reference_index,
            channel_count,
            sample_size,
            sample_rate,
            dops_box,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for OpusBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.data_reference_index)?;

        writer.write_u64::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.channel_count)?;
        writer.write_u16::<BigEndian>(self.sample_size)?;
        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u32::<BigEndian>(self.sample_rate.raw_value())?;

        if let Some(ref dops_box) = self.dops_box {
            dops_box.write_box(writer)?;
        }
        Ok(size)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DopsBox {
    pub version: u8,
    pub output_channel_count: u8,
    pub pre_skip: u16,
    pub input_sample_rate: u32,
    pub output_gain: i16,
    pub channel_mapping_family: u8,
    pub channel_mapping_table: Option<ChannelMappingTable>,
}

impl Default for DopsBox {
    fn default() -> Self {
        Self {
            version: 0,
            output_channel_count: 2,
            pre_skip: 16,
            input_sample_rate: 0,
            output_gain: -1,
            channel_mapping_family: 0,
            channel_mapping_table: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChannelMappingTable {
    pub stream_count: u8,
    pub coupled_count: u8,
    pub channel_mapping: Vec<u8>,
}

impl Default for ChannelMappingTable {
    fn default() -> Self {
        Self {
            stream_count: 0,
            coupled_count: 2,
            channel_mapping: Vec::new(),
        }
    }
}

impl Mp4Box for DopsBox {
    fn box_type(&self) -> BoxType {
        BoxType::DopsBox
    }

    fn box_size(&self) -> u64 {
        let mut channel_table_size = 0;
        if self.channel_mapping_family != 0 {
            channel_table_size = self.output_channel_count as u64 + 2;
        }
        HEADER_SIZE + 11 + channel_table_size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        Ok(String::new())
    }
}

impl DopsBox {
    pub fn new(config: &OpusConfig) -> Self {
        Self {
            version: 0,
            output_channel_count: config.chan_conf as u8,
            pre_skip: config.pre_skip,
            input_sample_rate: config.freq_index.freq(),
            output_gain: 0,
            channel_mapping_family: 0,
            channel_mapping_table: None,
        }
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for DopsBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;
        let version = reader.read_u8()?;
        let output_channel_count = reader.read_u8()?;
        let pre_skip = reader.read_u16::<BigEndian>()?;
        let input_sample_rate = reader.read_u32::<BigEndian>()?;
        let output_gain = reader.read_i16::<BigEndian>()?;
        let channel_mapping_family = reader.read_u8()?;
        let mut channel_mapping_table = None;
        if channel_mapping_family != 0 {
            let stream_count = reader.read_u8()?;
            let coupled_count = reader.read_u8()?;
            let mut channel_mapping = Vec::new();
            for _ in 0..output_channel_count {
                channel_mapping.push(reader.read_u8()?);
            }
            channel_mapping_table = Some(ChannelMappingTable {
                stream_count,
                coupled_count,
                channel_mapping,
            });
        }

        skip_bytes_to(reader, start + size)?;
        Ok(DopsBox {
            version,
            output_channel_count,
            pre_skip,
            input_sample_rate,
            output_gain,
            channel_mapping_family,
            channel_mapping_table,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for DopsBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u8(self.version)?;
        writer.write_u8(self.output_channel_count)?;
        writer.write_u16::<BigEndian>(self.pre_skip)?;
        writer.write_u32::<BigEndian>(self.input_sample_rate)?;
        writer.write_i16::<BigEndian>(self.output_gain)?;
        writer.write_u8(self.channel_mapping_family)?;

        if self.channel_mapping_family != 0 {
            let channel_mapping_table = self.channel_mapping_table.clone().unwrap();
            writer.write_u8(channel_mapping_table.stream_count)?;
            writer.write_u8(channel_mapping_table.coupled_count)?;
            for b in channel_mapping_table.channel_mapping.iter() {
                writer.write_u8(*b)?;
            }
        }
        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_opus() {
        let src_box = OpusBox {
            data_reference_index: 1,
            channel_count: 6,
            sample_size: 16,
            sample_rate: FixedPointU16::new(48000),
            dops_box: Some(DopsBox {
                version: 0,
                output_channel_count: 6,
                pre_skip: 312,
                input_sample_rate: 48000,
                output_gain: 0,
                channel_mapping_family: 1,
                channel_mapping_table: Some(ChannelMappingTable {
                    stream_count: 4,
                    coupled_count: 2,
                    channel_mapping: [0, 4, 1, 2, 3, 5].to_vec(),
                }),
            }),
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::OpusBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = OpusBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_opus_witout_channel_mapping_table() {
        let src_box = OpusBox {
            data_reference_index: 1,
            channel_count: 6,
            sample_size: 16,
            sample_rate: FixedPointU16::new(48000),
            dops_box: Some(DopsBox {
                version: 0,
                output_channel_count: 6,
                pre_skip: 312,
                input_sample_rate: 48000,
                output_gain: 0,
                channel_mapping_family: 0,
                channel_mapping_table: None,
            }),
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::OpusBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = OpusBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
