use crate::{CompressionMethod, FragmentBytes, WanError};
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(PartialEq, Eq, Debug, Default)]
pub struct FragmentBytesStore {
    pub fragment_bytes: Vec<FragmentBytes>,
}

impl FragmentBytesStore {
    pub fn new_from_bytes<F: Read + Seek>(
        file: &mut F,
        amount_fragments_bytes: u32,
    ) -> Result<FragmentBytesStore, WanError> {
        trace!("will read {} FragmentBytes", amount_fragments_bytes);
        let mut fragment_bytes_pointers: Vec<u64> = Vec::new(); //list of reference to FragmentBytes
        for _ in 0..amount_fragments_bytes {
            let current_pointer = file.read_u32::<LE>()? as u64;
            if current_pointer == 0 {
                return Err(WanError::NullFragmentBytesPointer);
            };
            fragment_bytes_pointers.push(current_pointer);
        }

        trace!("reading the FragmentBytes table");
        let mut fragment_bytes = Vec::new();

        for (fragment_bytes_id, fragment_bytes_addr) in fragment_bytes_pointers.iter().enumerate() {
            trace!(
                "reading FragmentBytes n°{} at {}",
                fragment_bytes_id,
                fragment_bytes_addr
            );
            file.seek(SeekFrom::Start(*fragment_bytes_addr))?;
            let img = FragmentBytes::new_from_bytes(file)?;
            fragment_bytes.push(img);
        }

        Ok(FragmentBytesStore { fragment_bytes })
    }

    pub fn len(&self) -> usize {
        self.fragment_bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn write<F: Write + Seek>(
        &self,
        file: &mut F,
        compression: &CompressionMethod,
    ) -> Result<(Vec<u64>, Vec<u64>), WanError> {
        let mut fragment_bytes_addr = vec![];
        let mut sir0_pointer_fragments_bytes = vec![];

        for fragment_bytes in &self.fragment_bytes {
            trace!(
                "fragment bytes wrote at {}",
                file.seek(SeekFrom::Current(0))?
            );
            let (assembly_table_offset, sir0_img_pointer) =
                fragment_bytes.write(file, compression)?;
            for pointer in sir0_img_pointer {
                sir0_pointer_fragments_bytes.push(pointer)
            }
            fragment_bytes_addr.push(assembly_table_offset);
        }
        Ok((fragment_bytes_addr, sir0_pointer_fragments_bytes))
    }
}
