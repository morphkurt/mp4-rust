use serde::Serialize;

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SidxBox {
    pub version: u8,
    pub flags: u32,

    pub reference_id: u32,
    pub timescale: u32,
    pub earliest_presentation_time: u64,
    pub first_offset: u64,
    pub reserved: u16,
    pub reference_count: u16,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Segment {
    pub reference_type: bool,
    pub reference_size: u32,
    pub subsegment_duration: u32,
    pub starts_with_sap: bool,
    pub sap_type: u8,
    pub sap_delta_time: u32,
}

impl SidxBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::SidxBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        if self.version == 1 {
            size += 24;
        } else if self.version == 0 {
            size += 16;
        }
        size += 4;
        size += self.reference_count as u64 * 12;
        size
    }
}

impl Default for SidxBox {
    fn default() -> Self {
        SidxBox {
            version: 0,
            flags: 0,
            timescale: 1000,
            first_offset: 0,
            earliest_presentation_time: 0,
            reference_id: 0,
            reserved: 0,
            reference_count: 0,
            segments: Vec::new(),
        }
    }
}

impl Mp4Box for SidxBox {
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
            "timescale={} first_offset={} earliest_presentation_time={} reference_id={}, reserved={}, reference_count={}",
            self.timescale,
            self.first_offset,
            self.earliest_presentation_time,
            self.reference_id,
            self.reserved,
            self.reference_count,
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for SidxBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let (reference_id, timescale, earliest_presentation_time, first_offset) = if version == 0 {
            (
                reader.read_u32::<BigEndian>()?,
                reader.read_u32::<BigEndian>()?,
                reader.read_u32::<BigEndian>()? as u64,
                reader.read_u32::<BigEndian>()? as u64,
            )
        } else if version == 1 {
            (
                reader.read_u32::<BigEndian>()?,
                reader.read_u32::<BigEndian>()?,
                reader.read_u64::<BigEndian>()?,
                reader.read_u64::<BigEndian>()?,
            )
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
        };
        let reserved = reader.read_u16::<BigEndian>()?;
        let reference_count = reader.read_u16::<BigEndian>()?;
        let mut segments = Vec::with_capacity(reference_count as usize);

        for _ in 0..reference_count {
            let segment = Segment::read(reader)?;
            segments.push(segment);
        }

        skip_bytes_to(reader, start + size)?;

        Ok(SidxBox {
            version,
            flags,
            reference_id,
            timescale,
            earliest_presentation_time,
            first_offset,
            reserved,
            reference_count,
            segments,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for SidxBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u32::<BigEndian>(self.reference_id)?;
            writer.write_u32::<BigEndian>(self.timescale)?;
            writer.write_u64::<BigEndian>(self.earliest_presentation_time)?;
            writer.write_u64::<BigEndian>(self.first_offset)?;
        } else if self.version == 0 {
            writer.write_u32::<BigEndian>(self.reference_id)?;
            writer.write_u32::<BigEndian>(self.timescale)?;
            writer.write_u32::<BigEndian>(self.earliest_presentation_time as u32)?;
            writer.write_u32::<BigEndian>(self.first_offset as u32)?;
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
        }
        writer.write_u16::<BigEndian>(self.reserved)?; // reserved = 0
        writer.write_u16::<BigEndian>(self.reference_count)?; // reserved = 0

        for segment in self.segments.iter() {
            segment.write(writer)?;
        }

        Ok(size)
    }
}

impl Segment {
    fn size(&self) -> usize {
        12 as usize
    }

    fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let reference = reader.read_u32::<BigEndian>()?;
        let reference_type = reference >> 31 == 1;
        let reference_size = reference & 0x7FFFFFFF;
        let subsegment_duration = reader.read_u32::<BigEndian>()?;
        let sap = reader.read_u32::<BigEndian>()?;
        let starts_with_sap = sap >> 31 == 1;
        let sap_type = (sap & 0x70000000 >> 28) as u8;
        let sap_delta_time = sap & 0x0FFFFFFF;
        Ok(Segment {
            reference_type,
            reference_size,
            subsegment_duration,
            starts_with_sap,
            sap_type,
            sap_delta_time,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<u64> {
        let reference_type_flag = u32::from(self.reference_type) << 31;
        writer.write_u32::<BigEndian>(reference_type_flag | self.reference_size)?;
        writer.write_u32::<BigEndian>(self.subsegment_duration)?;
        let starts_with_sap_flag = u32::from(self.starts_with_sap) << 31;
        let sap_type = (self.sap_type as u32) << 28;
        writer.write_u32::<BigEndian>(starts_with_sap_flag | sap_type | self.sap_delta_time)?;
        Ok(self.size() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_sidx32() {
        let segment = Segment {
            reference_type: false,
            reference_size: 0,
            subsegment_duration: 123000,
            starts_with_sap: false,
            sap_type: 0,
            sap_delta_time: 0,
        };

        let src_box = SidxBox {
            version: 0,
            flags: 0,
            reference_id: 0,
            timescale: 0,
            earliest_presentation_time: 1344,
            first_offset: 212,
            reserved: 0,
            reference_count: 1,
            segments: vec![segment],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::SidxBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = SidxBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_sidx64() {
        let segment = Segment {
            reference_type: false,
            reference_size: 0,
            subsegment_duration: 123000,
            starts_with_sap: false,
            sap_type: 0,
            sap_delta_time: 0,
        };

        let src_box = SidxBox {
            version: 1,
            flags: 0,
            reference_id: 0,
            timescale: 0,
            earliest_presentation_time: 1344,
            first_offset: 212,
            reserved: 0,
            reference_count: 1,
            segments: vec![segment],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::SidxBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = SidxBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
