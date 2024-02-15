use std::io::{Read, Seek, SeekFrom};

use binread::BinReaderExt;
use byteorder::{ReadBytesExt, LE};

use crate::{wan_read_raw_4, WanError, shiren::ShirenFrameStore};

use super::ShirenFragmentBytesStore;

pub struct ShirenWan {
    pub fragment_bytes_store: ShirenFragmentBytesStore,
    pub frame_store: ShirenFrameStore
}

impl ShirenWan {
    pub fn new<T: Read + Seek>(reader: &mut T) -> Result<Self, WanError> {
        // read sir0 header
        reader.seek(SeekFrom::Start(0))?;
        let sir0_header = wan_read_raw_4(reader)?;
        if sir0_header != [0x53, 0x49, 0x52, 0x30] {
            return Err(WanError::InvalidSir0(sir0_header));
        };
        let wan_header_pointer = reader.read_u32::<LE>()?;
        reader.seek(SeekFrom::Current(4))?;
        let sir0_header_end = reader.read_u32::<LE>()?;
        if sir0_header_end != 0 {
            return Err(WanError::InvalidEndOfSir0Header(
                sir0_header_end.to_le_bytes(),
            ));
        }

        // read main header
        reader.seek(SeekFrom::Start(wan_header_pointer.into()))?;
        let (unk6_pointer, unk7_pointer, _unk8, fragment_bytes_store_pointer, _unk20, unk21): (u32, u32, u32, u32, u32, u32) =
            reader.read_le()?;


        // read fragment bytes store
        let fragment_bytes_store;
        if fragment_bytes_store_pointer != 0 {
            if unk21 == 0 {
                todo!();
            }
            let nb_fragments: usize = ((unk21-fragment_bytes_store_pointer) / 4) as usize;
            reader.seek(SeekFrom::Start(fragment_bytes_store_pointer.into()))?;
            fragment_bytes_store = ShirenFragmentBytesStore::new(reader, nb_fragments)?;
        } else {
            fragment_bytes_store = ShirenFragmentBytesStore::default();
        }

        if unk6_pointer == 0 || unk7_pointer == 0 {
            todo!();
        }
        
        reader.seek(SeekFrom::Start(unk7_pointer as u64))?;
        let unk7_first_entry_pointer = reader.read_u32::<LE>()?;
        let nb_frame_fragment = (unk7_first_entry_pointer - unk6_pointer)/4 - 1;
        reader.seek(SeekFrom::Start(unk6_pointer as u64))?;
        let frame_store = ShirenFrameStore::new(reader, nb_frame_fragment)?;
        
        Ok(Self {
            fragment_bytes_store,
            frame_store
        })
    }
}
