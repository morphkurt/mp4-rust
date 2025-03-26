use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::{hev1::HvcCBox, mp4box::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hvc1Box {
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,

    #[serde(with = "value_u32")]
    pub horizresolution: FixedPointU16,

    #[serde(with = "value_u32")]
    pub vertresolution: FixedPointU16,
    pub frame_count: u16,
    pub depth: u16,
    pub hvcc: HvcCBox,
}

impl Default for Hvc1Box {
    fn default() -> Self {
        Hvc1Box {
            data_reference_index: 0,
            width: 0,
            height: 0,
            horizresolution: FixedPointU16::new(0x48),
            vertresolution: FixedPointU16::new(0x48),
            frame_count: 1,
            depth: 0x0018,
            hvcc: HvcCBox::default(),
        }
    }
}

impl Hvc1Box {
    pub fn new(config: &HevcConfig) -> Self {
        Self {
            data_reference_index: 1,
            width: config.width.unwrap_or(0),
            height: config.height.unwrap_or(0),
            horizresolution: FixedPointU16::new(0x48),
            vertresolution: FixedPointU16::new(0x48),
            frame_count: 1,
            depth: 0x0018,
            hvcc: HvcCBox {
                configuration_version: config.configuration_version.unwrap_or(1),
                general_profile_space: config.general_profile_space.unwrap_or(0),
                general_tier_flag: config.general_tier_flag.unwrap_or(false),
                general_profile_idc: config.general_profile_idc.unwrap_or(1),
                general_profile_compatibility_flags: config.general_profile_compatibility_flags.unwrap_or(0),
                general_constraint_indicator_flag: config.general_constraint_indicator_flag.unwrap_or(0),
                general_level_idc: config.general_level_idc.unwrap_or(93),
                min_spatial_segmentation_idc: config.min_spatial_segmentation_idc.unwrap_or(0),
                parallelism_type: config.parallelism_type.unwrap_or(0),
                chroma_format_idc: config.chroma_format_idc.unwrap_or(1),
                bit_depth_luma_minus8: config.bit_depth_luma_minus8.unwrap_or(0),
                bit_depth_chroma_minus8: config.bit_depth_chroma_minus8.unwrap_or(0),
                avg_frame_rate: config.avg_frame_rate.unwrap_or(0),
                constant_frame_rate: config.constant_frame_rate.unwrap_or(0),
                num_temporal_layers: config.num_temporal_layers.unwrap_or(1),
                temporal_id_nested: config.temporal_id_nested.unwrap_or(false),
                length_size_minus_one: config.length_size_minus_one.unwrap_or(3),
                arrays: config.arrays.clone().unwrap_or_default(),
            },
        }
    }

    pub fn get_type(&self) -> BoxType {
        BoxType::Hvc1Box
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + 8 + 70 + self.hvcc.box_size()
    }
}

impl Mp4Box for Hvc1Box {
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
            "data_reference_index={} width={} height={} frame_count={}",
            self.data_reference_index, self.width, self.height, self.frame_count
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for Hvc1Box {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        reader.read_u32::<BigEndian>()?; // reserved
        reader.read_u16::<BigEndian>()?; // reserved
        let data_reference_index = reader.read_u16::<BigEndian>()?;

        reader.read_u32::<BigEndian>()?; // pre-defined, reserved
        reader.read_u64::<BigEndian>()?; // pre-defined
        reader.read_u32::<BigEndian>()?; // pre-defined
        let width = reader.read_u16::<BigEndian>()?;
        let height = reader.read_u16::<BigEndian>()?;
        let horizresolution = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);
        let vertresolution = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);
        reader.read_u32::<BigEndian>()?; // reserved
        let frame_count = reader.read_u16::<BigEndian>()?;
        skip_bytes(reader, 32)?; // compressorname
        let depth = reader.read_u16::<BigEndian>()?;
        reader.read_i16::<BigEndian>()?; // pre-defined

        let header = BoxHeader::read(reader)?;
        let BoxHeader { name, size: s } = header;
        if s > size {
            return Err(Error::InvalidData(
                "hev1 box contains a box with a larger size than it",
            ));
        }
        if name == BoxType::HvcCBox {
            let hvcc = HvcCBox::read_box(reader, s)?;

            skip_bytes_to(reader, start + size)?;

            Ok(Hvc1Box {
                data_reference_index,
                width,
                height,
                horizresolution,
                vertresolution,
                frame_count,
                depth,
                hvcc,
            })
        } else {
            Err(Error::InvalidData("hvcc not found"))
        }
    }
}

impl<W: Write> WriteBox<&mut W> for Hvc1Box {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.data_reference_index)?;

        writer.write_u32::<BigEndian>(0)?; // pre-defined, reserved
        writer.write_u64::<BigEndian>(0)?; // pre-defined
        writer.write_u32::<BigEndian>(0)?; // pre-defined
        writer.write_u16::<BigEndian>(self.width)?;
        writer.write_u16::<BigEndian>(self.height)?;
        writer.write_u32::<BigEndian>(self.horizresolution.raw_value())?;
        writer.write_u32::<BigEndian>(self.vertresolution.raw_value())?;
        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.frame_count)?;
        // skip compressorname
        write_zeros(writer, 32)?;
        writer.write_u16::<BigEndian>(self.depth)?;
        writer.write_i16::<BigEndian>(-1)?; // pre-defined

        // Write the hvcc configuration data without its header
        writer.write_u8(self.hvcc.configuration_version)?;
        let general_profile_space = (self.hvcc.general_profile_space & 0b11) << 6;
        let general_tier_flag = u8::from(self.hvcc.general_tier_flag) << 5;
        let general_profile_idc = self.hvcc.general_profile_idc & 0b11111;
        writer.write_u8(general_profile_space | general_tier_flag | general_profile_idc)?;
        writer.write_u32::<BigEndian>(self.hvcc.general_profile_compatibility_flags)?;
        writer.write_u48::<BigEndian>(self.hvcc.general_constraint_indicator_flag)?;
        writer.write_u8(self.hvcc.general_level_idc)?;
        writer.write_u16::<BigEndian>(self.hvcc.min_spatial_segmentation_idc & 0x0FFF)?;
        writer.write_u8(self.hvcc.parallelism_type & 0b11)?;
        writer.write_u8(self.hvcc.chroma_format_idc & 0b11)?;
        writer.write_u8(self.hvcc.bit_depth_luma_minus8 & 0b111)?;
        writer.write_u8(self.hvcc.bit_depth_chroma_minus8 & 0b111)?;
        writer.write_u16::<BigEndian>(self.hvcc.avg_frame_rate)?;
        let constant_frame_rate = (self.hvcc.constant_frame_rate & 0b11) << 6;
        let num_temporal_layers = (self.hvcc.num_temporal_layers & 0b111) << 3;
        let temporal_id_nested = u8::from(self.hvcc.temporal_id_nested) << 2;
        let length_size_minus_one = self.hvcc.length_size_minus_one & 0b11;
        writer.write_u8(constant_frame_rate | num_temporal_layers | temporal_id_nested | length_size_minus_one)?;
        writer.write_u8(self.hvcc.arrays.len() as u8)?;
        for arr in &self.hvcc.arrays {
            writer.write_u8((arr.nal_unit_type & 0b111111) | u8::from(arr.completeness) << 7)?;
            writer.write_u16::<BigEndian>(arr.nalus.len() as _)?;
            for nalu in &arr.nalus {
                writer.write_u16::<BigEndian>(nalu.size)?;
                writer.write_all(&nalu.data)?;
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
    fn test_hvc1() {
        let src_box = Hvc1Box {
            data_reference_index: 1,
            width: 320,
            height: 240,
            horizresolution: FixedPointU16::new(0x48),
            vertresolution: FixedPointU16::new(0x48),
            frame_count: 1,
            depth: 24,
            hvcc: HvcCBox {
                configuration_version: 1,
                ..Default::default()
            },
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::Hvc1Box);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = Hvc1Box::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
