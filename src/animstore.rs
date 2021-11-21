use crate::{Animation, WanError};
use binwrite::BinWrite;
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Debug)]
struct AnimGroupEntry {
    pointer: u32,
    group_lenght: u16,
    _unk16: u16,
    id: u16,
}

pub struct AnimStore {
    pub animations: Vec<Animation>,
    pub copied_on_previous: Option<Vec<bool>>, //indicate if a sprite can copy on the previous. Will always copy if possible if None
    pub anim_groups: Vec<Option<(usize, usize)>>, //usize1 = start, usize2 = lenght
}

impl AnimStore {
    pub fn new<F: Read + Seek>(
        file: &mut F,
        pointer_animation_groups_table: u64,
        amount_animation_group: u16,
        is_for_chara: bool,
    ) -> Result<(AnimStore, u64), WanError> {
        //TODO: rewrite this function, it seem to be too complicated to understand
        file.seek(SeekFrom::Start(pointer_animation_groups_table))?;
        let mut anim_group_entry: Vec<Option<AnimGroupEntry>> = Vec::new();
        let add_for_chara = if is_for_chara { 7 } else { 0 };
        for animation_group_id in 0..amount_animation_group.checked_add(add_for_chara).ok_or(
            WanError::OverflowAddition(
                amount_animation_group as u64,
                add_for_chara as u64,
                "amount animation group",
                "add for chara",
            ),
        )? {
            //HACK: CRITICAL: (why is it ?)
            let pointer = file.read_u32::<LE>()?;
            if pointer == 0 {
                anim_group_entry.push(None);
                continue;
            };
            let group_lenght = file.read_u16::<LE>()?;
            let _unk16 = file.read_u16::<LE>()?;
            anim_group_entry.push(Some(AnimGroupEntry {
                pointer,
                group_lenght,
                _unk16,
                id: animation_group_id,
            }));
        }

        let mut anim_groups: Vec<Option<Vec<u64>>> = Vec::new();
        let mut particule_table_end = None;
        for anim_group_option in anim_group_entry {
            match anim_group_option {
                None => anim_groups.push(None),
                Some(anim_group) => {
                    file.seek(SeekFrom::Start(anim_group.pointer as u64))?;
                    match particule_table_end {
                        Some(value) => {
                            if file.seek(SeekFrom::Current(0))? < value {
                                particule_table_end = Some(file.seek(SeekFrom::Current(0))?);
                            }
                        }
                        None => particule_table_end = Some(file.seek(SeekFrom::Current(0))?),
                    };

                    let mut anim_ref = Vec::new();
                    for _ in 0..anim_group.group_lenght {
                        anim_ref.push(file.read_u32::<LE>()? as u64);
                    }
                    trace!(
                        "reading an animation group entry, id is {}, the pointer is {:?}",
                        anim_group.id,
                        anim_ref
                    );
                    anim_groups.push(Some(anim_ref));
                }
            };
        }

        let particule_table_end = match particule_table_end {
            None => file.seek(SeekFrom::Current(0))?,
            Some(value) => value,
        };

        let mut animations: Vec<Animation> = Vec::new();
        let mut copied_on_previous = Vec::new();
        let mut anim_groups_result = Vec::new();
        let mut check_last_anim_pos = 0;
        for anim_group in anim_groups {
            match anim_group {
                None => anim_groups_result.push(None),
                Some(anim_group_table) => {
                    anim_groups_result.push(Some((animations.len(), anim_group_table.len())));
                    for animation in anim_group_table {
                        file.seek(SeekFrom::Start(animation))?;
                        //TODO: what is this error ?
                        /*if check_last_anim_pos > file.seek(SeekFrom::Current(0))? {
                            bail!("The check for the order of animation haven't verified.")
                        };*/
                        copied_on_previous
                            .push(file.seek(SeekFrom::Current(0))? == check_last_anim_pos);
                        check_last_anim_pos = file.seek(SeekFrom::Current(0))?;
                        animations.push(Animation::new(file)?);
                    }
                }
            };
        }

        Ok((
            AnimStore {
                animations,
                copied_on_previous: Some(copied_on_previous),
                anim_groups: anim_groups_result,
            },
            particule_table_end,
        ))
    }

    pub fn len(&self) -> usize {
        self.animations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn write<F: Write + Seek>(
        file: &mut F,
        anim_store: &AnimStore,
    ) -> Result<Vec<u64>, WanError> {
        let mut animations_pointer = vec![];
        let mut previous_animation: Option<&Animation> = None;
        let mut previous_pointer = None;

        for loop_nb in 0..anim_store.animations.len() {
            let animation = &anim_store.animations[loop_nb];
            let can_copy_on_previous = match &anim_store.copied_on_previous {
                None => true,
                Some(value) => value[loop_nb],
            };
            let actual_pointer = file.seek(SeekFrom::Current(0))?;

            if can_copy_on_previous {
                if let Some(p_anim) = previous_animation {
                    if *p_anim == *animation {
                        animations_pointer.push(previous_pointer.unwrap());
                        continue;
                    }
                };
            };

            animations_pointer.push(actual_pointer);
            Animation::write(file, animation)?;
            previous_animation = Some(animation);
            previous_pointer = Some(actual_pointer);
        }
        Ok(animations_pointer)
    }

    pub fn write_animation_group<F: Write + Seek>(
        &self,
        file: &mut F,
        animations_pointer: &[u64],
    ) -> Result<(u64, Vec<u64>), WanError> {
        let mut sir0_animation = Vec::new();

        struct AnimGroupData {
            pointer: u32,
            lenght: u16,
        }

        let mut anim_group_data: Vec<Option<AnimGroupData>> = Vec::new();
        for anim_group in &self.anim_groups {
            match anim_group {
                None => {
                    anim_group_data.push(None);
                    (0u16).write(file)?;
                }
                Some(value) => {
                    anim_group_data.push(Some(AnimGroupData {
                        pointer: file.seek(SeekFrom::Current(0))? as u32,
                        lenght: value.1 as u16,
                    }));
                    for anim_pos in 0..value.1 {
                        sir0_animation.push(file.seek(SeekFrom::Current(0))?);
                        (animations_pointer[(value.0 as usize) + anim_pos] as u32).write(file)?;
                    }
                }
            }
        }

        let animation_group_reference_offset = file.seek(SeekFrom::Current(0))?;

        for actual_data in anim_group_data {
            match actual_data {
                None => 0u32.write(file)?,
                Some(data) => {
                    sir0_animation.push(file.seek(SeekFrom::Current(0))?);
                    (data.pointer, data.lenght, 0u16).write(file)?;
                }
            };
        }

        Ok((animation_group_reference_offset, sir0_animation))
    }
}
