use std::io::{Read, Seek, SeekFrom};

use byteorder::{ReadBytesExt, LE};

use crate::WanError;

use super::ShirenFragmentBytes;


#[derive(Default)]
pub struct ShirenFragmentBytesStore {
    pub fragment_bytes: Vec<ShirenFragmentBytes>
}

impl ShirenFragmentBytesStore {
    pub fn new<T: Read + Seek>(reader: &mut T, nb_fragments: usize) -> Result<Self, WanError> {
        debug!("reading {} fragments byte", nb_fragments);
        let mut pointers = Vec::with_capacity(nb_fragments);
        for _ in 0..nb_fragments {
            pointers.push(reader.read_u32::<LE>()?);
        }
        let mut fragments = Vec::with_capacity(nb_fragments);
        for fragment_pointer in pointers {
            trace!("reading fragment bytes at {}", fragment_pointer);
            reader.seek(SeekFrom::Start(fragment_pointer as u64))?;
            fragments.push(ShirenFragmentBytes::new(reader)?);
        };
        Ok(Self {
            fragment_bytes: fragments
        })
    }
}