use crate::{
    encode_fragment_pixels, get_opt_le, wan_read_raw_4, AnimationStore, CompressionMethod, Fragment,
    FragmentFlip, FragmentResolution, Frame, ImageBytes, ImageBytesToImageError,
};
use crate::{FrameStore, ImageStore, Palette, SpriteType, WanError};

use anyhow::Context;
use binwrite::BinWrite;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use image::{ImageBuffer, Rgba};
use pmd_sir0::write_sir0_footer;
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(PartialEq, Eq, Debug)]
pub struct WanImage {
    pub image_store: ImageStore,
    pub frames: FrameStore,
    pub anim_store: AnimationStore,
    pub palette: Palette,
    pub raw_particule_table: Vec<u8>,
    /// true if the picture have 256 color, false if it only have 16
    pub is_256_color: bool,
    pub sprite_type: SpriteType,
    pub unk2: u16,
    /// How the imagebytes should be compressed
    pub compression: CompressionMethod,
}

impl WanImage {
    /// Create an empty 16 color sprite for the given [`SpriteType`]
    pub fn new(sprite_type: SpriteType) -> Self {
        Self {
            image_store: ImageStore::default(),
            frames: FrameStore::default(),
            anim_store: AnimationStore::default(),
            palette: Palette::default(),
            raw_particule_table: Vec::new(),
            is_256_color: false,
            sprite_type,
            unk2: 0,
            compression: sprite_type.default_compression_method(),
        }
    }

    /// parse an image in the wan/wat format stored in the input file
    /// It assume that the file is decompressed
    pub fn decode_wan<F: Read + Seek>(mut file: F) -> Result<WanImage, WanError> {
        let source_file_lenght = file.seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(0))?;
        debug!("start to decode a wan image");

        // first step: decode the sir0 header
        trace!("decoding the sir0 header");
        let sir0_header = wan_read_raw_4(&mut file)?;
        if sir0_header != [0x53, 0x49, 0x52, 0x30] {
            return Err(WanError::InvalidSir0(sir0_header));
        };
        let sir0_pointer_header = file.read_u32::<LE>()? as u64;
        let _sir0_pointer_offset = file.read_u32::<LE>()? as u64;

        let sir0_header_end = wan_read_raw_4(&mut file)?;
        if sir0_header_end != [0, 0, 0, 0] {
            return Err(WanError::InvalidEndOfSir0Header(sir0_header_end));
        };

        // second step: decode the wan header
        trace!("reading the wan header");
        file.seek(SeekFrom::Start(sir0_pointer_header))?;
        let pointer_to_anim_info = file.read_u32::<LE>()? as u64;
        let pointer_to_image_data_info = file.read_u32::<LE>()? as u64;
        let sprite_type = match file.read_u16::<LE>()? {
            0 => SpriteType::PropsUI,
            1 => SpriteType::Chara,
            3 => SpriteType::Unknown,
            value => return Err(WanError::TypeOfSpriteUnknown(value)),
        };
        //unk #12

        // third step: decode animation info block
        trace!("reading the animation info block");
        file.seek(SeekFrom::Start(pointer_to_anim_info))?;
        let pointer_fragment_reference_table = file.read_u32::<LE>()? as u64;
        if pointer_fragment_reference_table > source_file_lenght {
            return Err(WanError::PostFilePointer("meta frame reference table"));
        }
        let pointer_particule_offset_table = file.read_u32::<LE>()? as u64;
        if pointer_particule_offset_table > source_file_lenght {
            return Err(WanError::PostFilePointer("particule offset table"));
        };
        let pointer_animation_groups_table = file.read_u32::<LE>()? as u64;
        if pointer_animation_groups_table > source_file_lenght {
            return Err(WanError::PostFilePointer("animation groups table"));
        }
        let amount_animation_group = file.read_u16::<LE>()?;

        let _size_to_allocate_for_max_frame = file.read_u32::<LE>()?;

        // fourth: decode image data info
        trace!("reading the image data info");
        file.seek(SeekFrom::Start(pointer_to_image_data_info))?;
        let pointer_image_data_pointer_table = file.read_u32::<LE>()? as u64;
        let pointer_palette = file.read_u32::<LE>()? as u64;
        file.read_u16::<LE>()?; //unk
        let is_256_color = match file.read_u16::<LE>()? {
            0 => false,
            1 => true,
            color_id => return Err(WanError::InvalidColorNumber(color_id)),
        };
        let unk2 = file.read_u16::<LE>()?;
        let amount_images = file.read_u16::<LE>()?;

        trace!("parsing the palette");

        file.seek(SeekFrom::Start(pointer_palette))?;
        let palette = Palette::new_from_bytes(&mut file)?;

        // decode fragments
        trace!("decoding meta-frame");
        let fragment_reference_end_pointer: u64 = match pointer_particule_offset_table {
            0 => match WanImage::find_first_non_null_animation_seq_entry(
                &mut file,
                pointer_animation_groups_table,
            ) {
                Some(v) => v,
                // Fall back to animation group offset
                None => pointer_animation_groups_table,
            },
            value => value,
        };

        let amount_fragments_raw = fragment_reference_end_pointer
            .checked_sub(pointer_fragment_reference_table)
            .ok_or(WanError::OverflowSubstraction(
                fragment_reference_end_pointer as u64,
                pointer_fragment_reference_table as u64,
                "meta frame reference end pointer",
                "pointer meta frame reference table",
            ))?;

        let amount_fragments = amount_fragments_raw / 4;

        file.seek(SeekFrom::Start(pointer_fragment_reference_table))?;
        let fragments_store = FrameStore::new_from_bytes(&mut file, amount_fragments)?;

        // decode image
        trace!("reading the image data pointer table");
        file.seek(SeekFrom::Start(pointer_image_data_pointer_table))?;
        trace!(
            "start of the image part (source) : {}",
            pointer_image_data_pointer_table
        );
        let image_store = ImageStore::new_from_bytes(&mut file, amount_images as u32)?;

        // decode animation
        let (anim_store, particule_table_end) = AnimationStore::new(
            &mut file,
            pointer_animation_groups_table,
            amount_animation_group,
        )?;

        let mut raw_particule_table: Vec<u8>;
        if pointer_particule_offset_table > 0 {
            if particule_table_end > source_file_lenght {
                return Err(WanError::PostFilePointer("particle table end"));
            };
            trace!(
                "copying the raw particle table (from {} to end at {})",
                pointer_particule_offset_table,
                particule_table_end
            );
            file.seek(SeekFrom::Start(pointer_particule_offset_table))?;
            raw_particule_table = vec![
                0;
                particule_table_end
                    .checked_sub(pointer_particule_offset_table)
                    .ok_or(WanError::OverflowSubstraction(
                        particule_table_end,
                        pointer_particule_offset_table,
                        "particule table end",
                        "pointer particule offset table"
                    ))? as usize
            ];
            file.read_exact(&mut raw_particule_table)?;
        } else {
            raw_particule_table = Vec::new();
        }

        Ok(WanImage {
            image_store,
            frames: fragments_store,
            anim_store,
            palette,
            raw_particule_table,
            is_256_color,
            sprite_type,
            unk2,
            compression: sprite_type.default_compression_method(),
        })
    }

    /// If the file doesn't have an entity effect particle list, we ned to instead search
    /// for the pointer to the first animation sequence, to get the end of the meta frame table.
    fn find_first_non_null_animation_seq_entry<F: Read + Seek>(
        file: &mut F,
        pointer_animation_groups_table: u64,
    ) -> Option<u64> {
        file.seek(SeekFrom::Start(pointer_animation_groups_table))
            .ok()?;
        while let Ok(pntr) = file.read_u32::<LE>() {
            if pntr != 0 {
                return Some(pntr as u64);
            }
        }
        None
    }

    pub fn create_wan<F: Write + Seek>(&self, file: &mut F) -> anyhow::Result<()> {
        let opt_le = get_opt_le();
        debug!("start creating a wan image");

        let mut sir0_offsets: Vec<u32> = vec![];
        // create the sir0 header
        trace!("creating the sir0 header");
        file.write_all(&[0x53, 0x49, 0x52, 0x30])?;

        let sir0_pointer_header = file.seek(SeekFrom::Current(0))?;
        sir0_offsets.push(sir0_pointer_header as u32);
        0u32.write(file)?; //sir0_pointer_header

        let sir0_pointer_offset = file.seek(SeekFrom::Current(0))?;
        sir0_offsets.push(sir0_pointer_offset as u32);

        file.write_all(&[0; 8])?; //magic

        // write frames
        trace!(
            "start of frames reference: {}",
            file.seek(SeekFrom::Current(0))?
        );
        let (fragments_references, size_to_allocate_for_max_frame) = self.frames.write(file)?;

        trace!(
            "start of the animation offset: {}",
            file.seek(SeekFrom::Current(0))?
        );
        let animations_pointer = self.anim_store.write(file)?;

        while file.seek(SeekFrom::Current(0))? % 4 != 0 {
            file.write_all(&[0xAA])?;
        }

        trace!(
            "start of the image offset: {}",
            file.seek(SeekFrom::Current(0))?
        );

        let (image_offset, sir0_pointer_images) =
            self.image_store.write(file, &self.compression)?;

        for pointer in sir0_pointer_images {
            sir0_offsets.push(pointer as u32);
        }

        trace!("start of the palette: {}", file.seek(SeekFrom::Current(0))?);
        let pointer_palette = self
            .palette
            .write(file)
            .context("Failed to write the palette")?;
        //sir0_offsets.push(pointer_palette);

        sir0_offsets.push(pointer_palette as u32);

        trace!(
            "start of the fragment reference offset: {}",
            file.seek(SeekFrom::Current(0))?
        );
        let fragment_reference_offset = file.seek(SeekFrom::Current(0))?;
        for reference in fragments_references {
            sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
            file.write_u32::<LE>(reference)?;
        }

        let particule_offset = if !self.raw_particule_table.is_empty() {
            let particule_offset = file.seek(SeekFrom::Current(0))?;
            trace!(
                "start of the particule offset: {}",
                file.seek(SeekFrom::Current(0))?
            );
            //HACK: particule offset table parsing is not implement (see the psycommando code of ppmdu)
            file.write_all(&self.raw_particule_table)?;
            sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
            Some(particule_offset)
        } else {
            None
        };

        trace!(
            "start of the animation group reference: {}",
            file.seek(SeekFrom::Current(0))?
        );
        let (animation_group_reference_offset, sir0_animation_pointer) = self
            .anim_store
            .write_animation_group(file, &animations_pointer)
            .context("failed to write animations groups")?;
        for pointer in sir0_animation_pointer {
            sir0_offsets.push(pointer as u32);
        }

        //image offset
        let pointer_image_data_pointer_table = file.seek(SeekFrom::Current(0))?;
        trace!(
            "start of the image offset: {}",
            file.seek(SeekFrom::Current(0))?
        );
        for offset in image_offset {
            sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
            file.write_u32::<LE>(offset as u32)?;
        }

        // animation header
        let animation_info_offset = file.seek(SeekFrom::Current(0))?;
        trace!(
            "start of the animation header: {}",
            file.seek(SeekFrom::Current(0))?
        );
        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        file.write_u32::<LE>(fragment_reference_offset as u32)?;

        if let Some(particule_offset) = particule_offset {
            sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
            file.write_u32::<LE>(particule_offset as u32)?;
        } else {
            file.write_all(&[0; 4])?;
        }

        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        file.write_u32::<LE>(animation_group_reference_offset as u32)?;

        file.write_u16::<LE>(self.anim_store.anim_groups.len() as u16)?;

        file.write_u32::<LE>(size_to_allocate_for_max_frame as u32)?;
        file.write_all(&[0; 6])?;

        // images header
        trace!(
            "start of the images header: {}",
            file.seek(SeekFrom::Current(0))?
        );
        let image_info_offset = file.seek(SeekFrom::Current(0))?;
        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        file.write_u32::<LE>(pointer_image_data_pointer_table as u32)?;

        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        (
            pointer_palette as u32,
            0u16,
            if self.is_256_color { 1u16 } else { 0u16 },
            self.unk2,
            self.image_store.len() as u16,
        )
            .write_options(file, &opt_le)?;

        // wan header
        let wan_header_pos = file.seek(SeekFrom::Current(0))?;
        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        file.write_u32::<LE>(animation_info_offset as u32)?;
        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        file.write_u32::<LE>(image_info_offset as u32)?;
        file.write_u16::<LE>(self.sprite_type.get_id() as u16)?;

        file.write_all(&[0, 0])?;

        while file.seek(SeekFrom::Current(0))? % 16 != 0 {
            file.write_all(&[0xAA])?;
        }

        let sir0_offset_pos = file.seek(SeekFrom::Current(0))?;
        // write the sir0 ending

        trace!(
            "start of the sir0 list: {}",
            file.seek(SeekFrom::Current(0))?
        );
        write_sir0_footer(file, &sir0_offsets).context("failed to write the Sir0 footer")?;

        //padding
        file.write_all(&[0x00])?;
        while file.seek(SeekFrom::Current(0))? % 16 != 0 {
            file.write_all(&[0xAA])?;
        }

        // write the sir0 header
        file.seek(SeekFrom::Start(sir0_pointer_header))?;
        file.write_u32::<LE>(wan_header_pos as u32)?;

        file.seek(SeekFrom::Start(sir0_pointer_offset))?;
        file.write_u32::<LE>(sir0_offset_pos as u32)?;

        file.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    /// Return the image corresponding to the resolution and the palette of given meta-frame.
    /// Doesn't perform flipping or any other transformation other than the resolution and the palette.
    pub fn get_image_for_fragment(
        &self,
        fragment: &Fragment,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, ImageBytesToImageError> {
        let image_bytes = match self.image_store.images.get(fragment.image_bytes_index) {
            Some(b) => b,
            None => {
                return Err(ImageBytesToImageError::NoImageBytes(
                    fragment.image_bytes_index,
                ))
            }
        };

        image_bytes.get_image(&self.palette, &fragment.resolution, fragment.pal_idx)
    }

    pub fn fix_empty_frames(&mut self) {
        let collected: Vec<&mut Frame> = self
            .frames
            .frames
            .iter_mut()
            .filter(|x| (*x).fragments.is_empty())
            .collect();
        if collected.is_empty() {
            return;
        }
        let image_bytes_index = self.image_store.images.len();
        let resolution = FragmentResolution { x: 8, y: 8 };
        self.image_store.images.push(ImageBytes {
            // no panic: We guarantee input parameters are valid
            mixed_pixels: encode_fragment_pixels(&[0; 256], resolution).unwrap(),
            z_index: 0,
        });
        for empty_frame in collected {
            empty_frame.fragments.push(Fragment {
                unk1: 0,
                unk3_4: None,
                unk5: false,
                image_bytes_index,
                offset_y: 0,
                offset_x: 0,
                flip: FragmentFlip::Standard,
                is_mosaic: false,
                pal_idx: 0,
                resolution,
            })
        }
    }
}
