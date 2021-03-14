use crate::{AnimStore, wan_read_raw_4};
use crate::{ImageStore, MetaFrameStore, Palette, SpriteType, WanError};

use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom};

pub struct WanImage {
    pub image_store: ImageStore,
    pub meta_frame_store: MetaFrameStore,
    pub anim_store: AnimStore,
    pub palette: Palette,
    pub raw_particule_table: Vec<u8>,
    /// true if the picture have 256 color, false if it only have 16
    pub is_256_color: bool,
    pub sprite_type: SpriteType,
    pub unk_1: u32,
}

impl WanImage {
    pub fn new<F: Read + Seek>(b: F) -> Result<WanImage, WanError> {
        WanImage::decode_wan(b)
    }

    /// parse an image in the wan/wat format stored in the input file
    /// It assume that the file is decompressed
    pub fn decode_wan<F: Read + Seek>(mut file: F) -> Result<WanImage, WanError> {
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
        //TODO
        /*if file.seek(SeekFrom::Current(0))? != sir0_pointer_header+10 { // an assertion
            bail!("we are not at the good position after the wan header!!!");
        }*/

        // third step: decode animation info block
        trace!("reading the animation info block");
        file.seek(SeekFrom::Start(pointer_to_anim_info))?;
        let pointer_meta_frame_reference_table = file.read_u32::<LE>()? as u64;
        let pointer_particule_offset_table = file.read_u32::<LE>()? as u64;
        let pointer_animation_groups_table = file.read_u32::<LE>()? as u64;
        let amount_animation_group = file.read_u16::<LE>()?;

        //TODO
        /*if file.seek(SeekFrom::Current(0))? != pointer_to_anim_info + 14 {
            bail!("we are not at the good position after the animation info block!!!");
        };*/
        let unk_1 = file.read_u32::<LE>()?;

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
        file.read_u16::<LE>()?; //unk
        let amount_images = file.read_u16::<LE>()?;

        trace!("parsing the palette");
        //TODO
        /*if pointer_palette == 0 {
            bail!("the palette pointer is equal to 0 !!!");
        };*/
        file.seek(SeekFrom::Start(pointer_palette))?;
        let palette = Palette::new_from_bytes(&mut file)?;

        // decode meta-frame
        trace!("decoding meta-frame");
        let meta_frame_reference_end_pointer: u64 = match pointer_particule_offset_table {
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

        let amount_meta_frame_raw = meta_frame_reference_end_pointer
            .checked_sub(pointer_meta_frame_reference_table)
            .ok_or(WanError::OverflowSubstraction(
                meta_frame_reference_end_pointer as u64,
                pointer_meta_frame_reference_table as u64,
                "meta frame reference end pointer",
                "pointer meta frame reference table",
            ))?;

        let amount_meta_frame = amount_meta_frame_raw / 4;

        file.seek(SeekFrom::Start(pointer_meta_frame_reference_table))?;
        let meta_frame_store = MetaFrameStore::new_from_bytes(&mut file, amount_meta_frame)?;

        // decode image
        trace!("reading the image data pointer table");
        file.seek(SeekFrom::Start(pointer_image_data_pointer_table))?;
        let image_store = ImageStore::new_from_bytes(
            &mut file,
            amount_images as u32,
            &meta_frame_store,
            &palette,
        )?;

        // decode animation
        let (anim_store, particule_table_end) = AnimStore::new(
            &mut file,
            pointer_animation_groups_table,
            amount_animation_group,
            sprite_type == SpriteType::Chara,
        )?;

        let mut raw_particule_table: Vec<u8>;
        if pointer_particule_offset_table > 0 {
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
            meta_frame_store,
            anim_store,
            palette,
            raw_particule_table,
            is_256_color,
            sprite_type,
            unk_1,
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

    //TODO:
    /*pub fn create_wan<F: Write + Seek>(wanimage: &WanImage, file: &mut F) -> Result<(), WanError> {
        //TODO: transform all unwrap to chain_error error
        debug!("start creating a wan image");

        let mut sir0_offsets: Vec<u32> = vec![];
        // create the sir0 header
        trace!("creating the sir0 header");
        file.write(&[0x53, 0x49, 0x52, 0x30]).unwrap(); // sir0 magic

        let sir0_pointer_header = file.seek(SeekFrom::Current(0))?;
        sir0_offsets.push(sir0_pointer_header as u32);
        wan_write_u32(file, 0)?; //sir0_pointer_header

        let sir0_pointer_offset = file.seek(SeekFrom::Current(0))?;
        sir0_offsets.push(sir0_pointer_offset as u32);
        wan_write_u32(file, 0)?;

        wan_write_u32(file, 0)?; // magic

        // write meta-frame
        trace!("start of meta frame reference: {}", file.seek(SeekFrom::Current(0))?);
        let meta_frame_references = MetaFrameStore::write(file, &wanimage.meta_frame_store)?;

        trace!("start of the animation offset: {}", file.seek(SeekFrom::Current(0))?);
        let animations_pointer = AnimStore::write(file, &wanimage.anim_store)?;

        while file.seek(SeekFrom::Current(0))? % 4 != 0 {
            file.write_all(&[0xAA])?;
        };


        trace!("start of the image offset: {}", file.seek(SeekFrom::Current(0))?);
        let (image_offset, sir0_pointer_images) = ImageStore::write(file, &wanimage)?;

        for pointer in sir0_pointer_images {
            sir0_offsets.push(pointer as u32);
        };


        trace!("start of the palette: {}", file.seek(SeekFrom::Current(0))?);
        let pointer_palette = wanimage.palette.write(file).unwrap();
        //sir0_offsets.push(pointer_palette);

        sir0_offsets.push(pointer_palette as u32);

        trace!("start of the meta_frame reference offset: {}", file.seek(SeekFrom::Current(0))?);
        let meta_frame_reference_offset = file.seek(SeekFrom::Current(0))?;
        for reference in meta_frame_references {
            sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
            wan_write_u32(file, reference).unwrap();
        };

        let particule_offset = file.seek(SeekFrom::Current(0))?;
        trace!("start of the particule offset: {}", file.seek(SeekFrom::Current(0))?);
        //HACK: particule offset table parsing is not implement (see the psycommand code of spriteditor)
        file.write_all(&wanimage.raw_particule_table).unwrap();
        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);

        trace!("start of the animation group reference: {}", file.seek(SeekFrom::Current(0))?);
        let (animation_group_reference_offset, sir0_animation_pointer) = wanimage.anim_store.write_animation_group(file, &animations_pointer).unwrap();
        for pointer in sir0_animation_pointer {
            sir0_offsets.push(pointer as u32);
        };

        //image offset
        let pointer_image_data_pointer_table = file.seek(SeekFrom::Current(0))?;
        trace!("start of the image offset: {}", file.seek(SeekFrom::Current(0))?);
        for offset in image_offset {
            sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
            wan_write_u32(file, offset as u32).unwrap();
        };

        // animation header
        let animation_info_offset = file.seek(SeekFrom::Current(0))?;
        trace!("start of the animation header: {}", file.seek(SeekFrom::Current(0))?);
        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        wan_write_u32(file, meta_frame_reference_offset as u32).unwrap();
        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        wan_write_u32(file, particule_offset as u32).unwrap();
        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        wan_write_u32(file, animation_group_reference_offset as u32).unwrap();
        let is_for_chara = wanimage.sprite_type == SpriteType::Chara;
        wan_write_u16(file, (wanimage.anim_store.anim_groups.len()-if is_for_chara {7} else {0}) as u16).unwrap(); //HACK:

        // HACK: check what does this mean
        wan_write_u32(file, wanimage.unk_1).unwrap();
        wan_write_u32(file, 0)?;
        wan_write_u16(file, 0)?;

        // images header
        trace!("start of the images header: {}", file.seek(SeekFrom::Current(0))?);
        let image_info_offset = file.seek(SeekFrom::Current(0))?;
        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        wan_write_u32(file, pointer_image_data_pointer_table as u32).unwrap();
        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        wan_write_u32(file, pointer_palette as u32).unwrap();
        wan_write_u16(file, 0)?; //HACK: unknow
        wan_write_u16(file, if wanimage.is_256_color {
            1
        } else {
            0
        }).unwrap();

        wan_write_u16(file, 1).unwrap(); //HACK: unknow
        wan_write_u16(file, wanimage.image_store.len() as u16).unwrap();

        // wan header
        let wan_header_pos = file.seek(SeekFrom::Current(0))?;
        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        wan_write_u32(file, animation_info_offset as u32).unwrap();
        sir0_offsets.push(file.seek(SeekFrom::Current(0))? as u32);
        wan_write_u32(file, image_info_offset as u32).unwrap();
        wan_write_u16(file, wanimage.sprite_type.get_id() as u16).unwrap();


        wan_write_u16(file, 0)?;

        while file.seek(SeekFrom::Current(0))? % 32 != 0 {
            file.write_all(&[0xAA])?;
        }

        let sir0_offset_pos = file.seek(SeekFrom::Current(0))?;
        // write the sir0 ending

        trace!("start of the sir0 list: {}", file.seek(SeekFrom::Current(0))?);
        write_sir0_footer(file, sir0_offsets).unwrap();

        while file.seek(SeekFrom::Current(0))? % 32 != 0 {
            file.write_all(&[0xAA])?;
        }

        // write the sir0 header
        file.seek(SeekFrom::Start(sir0_pointer_header))?;
        wan_write_u32(file, wan_header_pos as u32).unwrap();

        file.seek(SeekFrom::Start(sir0_pointer_offset))?;
        wan_write_u32(file, sir0_offset_pos as u32)?;

        file.seek(SeekFrom::Start(0))?;
        Ok(())
    }*/
}
