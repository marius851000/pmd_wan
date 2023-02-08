use std::io::{Seek, Read, SeekFrom};

use binread::{BinRead, BinReaderExt};

use crate::WanError;

#[derive(BinRead, Debug)]
#[allow(dead_code)]
struct ShirenAssemblyEntry {
    pointer_to_bytes: u32,
    bytes_amount: u16,
    unk1: u16,
}

impl ShirenAssemblyEntry {
    pub fn is_empty(&self) -> bool {
        return self.pointer_to_bytes == 0 && self.bytes_amount == 0;
    }
}

#[derive(Debug)]
pub struct ShirenFragmentBytes {
    pub bytes: Vec<u8>
}

impl ShirenFragmentBytes {
    pub fn new<T: Read + Seek>(reader: &mut T) -> Result<Self, WanError> {
        let mut assembly_table = Vec::new();
        let mut total_size: usize = 0;
        loop {
            let assembly_entry: ShirenAssemblyEntry = reader.read_le()?;
            trace!("assembly_entry: {:?}", assembly_entry);
            if assembly_entry.is_empty() {
                break;
            } else {
                total_size += assembly_entry.bytes_amount as usize;
                assembly_table.push(assembly_entry);
            }
        }
        let mut bytes = vec![0; total_size];
        let mut position = 0;
        for entry in assembly_table.iter() {
            if entry.pointer_to_bytes != 0 {
                reader.seek(SeekFrom::Start(entry.pointer_to_bytes as u64))?;
                reader.read(&mut bytes[position..position+entry.bytes_amount as usize])?;
            }
            position += entry.bytes_amount as usize;
        };
        Ok(Self {
            bytes
        })
    }
}