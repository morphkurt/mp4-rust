use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::vp09::Vp09Box;
use crate::mp4box::*;
use crate::mp4box::{
    avc1::Avc1Box, hev1::Hev1Box, hvc1::Hvc1Box, mp4a::Mp4aBox, opus::OpusBox, tx3g::Tx3gBox,
};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct StsdBox {
    pub version: u8,
    pub flags: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avc1: Option<Avc1Box>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hev1: Option<Hev1Box>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hvc1: Option<Hvc1Box>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vp09: Option<Vp09Box>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mp4a: Option<Mp4aBox>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub opus: Option<OpusBox>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx3g: Option<Tx3gBox>,
}

impl StsdBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::StsdBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE + 4;
        if let Some(ref avc1) = self.avc1 {
            size += avc1.box_size();
        } else if let Some(ref hev1) = self.hev1 {
            size += hev1.box_size();
        } else if let Some(ref hvc1) = self.hvc1 {
            size += hvc1.box_size();
        } else if let Some(ref vp09) = self.vp09 {
            size += vp09.box_size();
        } else if let Some(ref mp4a) = self.mp4a {
            size += mp4a.box_size();
        } else if let Some(ref opus) = self.opus {
            size += opus.box_size();
        } else if let Some(ref tx3g) = self.tx3g {
            size += tx3g.box_size();
        }
        size
    }
}

impl Mp4Box for StsdBox {
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
        let s = String::new();
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for StsdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        reader.read_u32::<BigEndian>()?; // XXX entry_count

        let mut avc1 = None;
        let mut hev1 = None;
        let mut hvc1 = None;
        let mut vp09 = None;
        let mut mp4a = None;
        let mut opus = None;
        let mut tx3g = None;

        // Get box header.
        let header = BoxHeader::read(reader)?;
        let BoxHeader { name, size: s } = header;
        if s > size {
            return Err(Error::InvalidData(
                "stsd box contains a box with a larger size than it",
            ));
        }

        match name {
            BoxType::Avc1Box => {
                avc1 = Some(Avc1Box::read_box(reader, s)?);
            }
            BoxType::Hev1Box => {
                hev1 = Some(Hev1Box::read_box(reader, s)?);
            }
            BoxType::Hvc1Box => {
                hvc1 = Some(Hvc1Box::read_box(reader, s)?);
            }
            BoxType::Vp09Box => {
                vp09 = Some(Vp09Box::read_box(reader, s)?);
            }
            BoxType::Mp4aBox => {
                mp4a = Some(Mp4aBox::read_box(reader, s)?);
            }
            BoxType::OpusBox => {
                opus = Some(OpusBox::read_box(reader, s)?);
            }
            BoxType::Tx3gBox => {
                tx3g = Some(Tx3gBox::read_box(reader, s)?);
            }
            _ => {}
        }

        skip_bytes_to(reader, start + size)?;

        Ok(StsdBox {
            version,
            flags,
            avc1,
            hev1,
            hvc1,
            vp09,
            opus,
            mp4a,
            tx3g,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for StsdBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(1)?; // entry_count

        if let Some(ref avc1) = self.avc1 {
            avc1.write_box(writer)?;
        } else if let Some(ref hev1) = self.hev1 {
            hev1.write_box(writer)?;
        } else if let Some(ref hvc1) = self.hvc1 {
            hvc1.write_box(writer)?;
        } else if let Some(ref vp09) = self.vp09 {
            vp09.write_box(writer)?;
        } else if let Some(ref mp4a) = self.mp4a {
            mp4a.write_box(writer)?;
        } else if let Some(ref tx3g) = self.tx3g {
            tx3g.write_box(writer)?;
        } else if let Some(ref opus) = self.opus {
            opus.write_box(writer)?;
        }

        Ok(size)
    }
}
