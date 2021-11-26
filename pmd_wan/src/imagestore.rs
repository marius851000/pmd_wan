use crate::{ImageBytes, MetaFrameStore, WanError, WanImage};
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(PartialEq, Eq, Debug)]
pub struct ImageStore {
    pub images: Vec<ImageBytes>,
}

impl ImageStore {
    pub fn new_from_bytes<F: Read + Seek>(
        file: &mut F,
        amount_images: u32,
        meta_frame_store: &MetaFrameStore,
    ) -> Result<ImageStore, WanError> {
        trace!("will read {} image", amount_images);
        let mut image_pointers: Vec<u64> = Vec::new(); //list of reference to image
        for _ in 0..amount_images {
            let current_pointer = file.read_u32::<LE>()? as u64;
            if current_pointer == 0 {
                return Err(WanError::NullImagePointer);
            };
            image_pointers.push(current_pointer);
        }

        trace!("reading the image table");
        let mut images = Vec::new();

        for (image_id, image) in image_pointers.iter().enumerate() {
            trace!("reading image n°{} at {}", image_id, image);
            let (resolution, _pal_idx) =
                meta_frame_store.find_resolution_and_pal_idx_image(image_id as u32)?;
            let resolution = match resolution {
                None => return Err(WanError::ImageWithoutResolution),
                Some(value) => value,
            };
            file.seek(SeekFrom::Start(*image))?;
            let img = ImageBytes::new_from_bytes(file)?;
            images.push(img);
        }

        Ok(ImageStore { images })
    }

    pub fn len(&self) -> usize {
        self.images.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn write<F: Write + Seek>(
        file: &mut F,
        wanimage: &WanImage,
    ) -> Result<(Vec<u64>, Vec<u64>), WanError> {
        let mut image_offset = vec![];
        let mut sir0_pointer_images = vec![];

        for image in &wanimage.image_store.images {
            trace!(
                "image wrote at {}",
                file.seek(SeekFrom::Current(0)).unwrap()
            );
            let (assembly_table_offset, sir0_img_pointer) = image.write(file)?;
            for pointer in sir0_img_pointer {
                sir0_pointer_images.push(pointer)
            }
            image_offset.push(assembly_table_offset);
        }
        Ok((image_offset, sir0_pointer_images))
    }
}
