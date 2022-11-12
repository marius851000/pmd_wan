use crate::{CompressionMethod, FragmentBytes, WanError};
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(PartialEq, Eq, Debug, Default)]
pub struct FragmentStore {
    pub fragment_bytes: Vec<FragmentBytes>,
}

impl FragmentStore {
    pub fn new_from_bytes<F: Read + Seek>(
        file: &mut F,
        amount_fragments_bytes: u32,
    ) -> Result<FragmentStore, WanError> {
        trace!("will read {} image", amount_fragments_bytes);
        let mut image_pointers: Vec<u64> = Vec::new(); //list of reference to image
        for _ in 0..amount_fragments_bytes {
            let current_pointer = file.read_u32::<LE>()? as u64;
            if current_pointer == 0 {
                return Err(WanError::NullFragmentBytesPointer);
            };
            image_pointers.push(current_pointer);
        }

        trace!("reading the image table");
        let mut fragment_bytes = Vec::new();

        for (image_id, image) in image_pointers.iter().enumerate() {
            trace!("reading image n°{} at {}", image_id, image);
            file.seek(SeekFrom::Start(*image))?;
            let img = FragmentBytes::new_from_bytes(file)?;
            fragment_bytes.push(img);
        }

        Ok(FragmentStore { fragment_bytes })
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
        let mut image_offset = vec![];
        let mut sir0_pointer_images = vec![];

        for image in &self.fragment_bytes {
            trace!(
                "fragment bytes wrote at {}",
                file.seek(SeekFrom::Current(0))?
            );
            let (assembly_table_offset, sir0_img_pointer) = image.write(file, compression)?;
            for pointer in sir0_img_pointer {
                sir0_pointer_images.push(pointer)
            }
            image_offset.push(assembly_table_offset);
        }
        Ok((image_offset, sir0_pointer_images))
    }
}
