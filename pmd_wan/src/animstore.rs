use crate::{Animation, WanError};
use binwrite::BinWrite;
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Debug)]
struct AnimGroupEntry {
    pointer: u32,
    group_lenght: u32,
    id: u16,
}

/// Contain all the [`Animation`], as well as all the animation group (a.k.a animation table in ppmdu sprite editor).
/// Animation group are a list of [`Animation`]. An animation group usually have 8 entry, one per rotation of the monster.
#[derive(PartialEq, Eq, Debug)]
pub struct AnimStore {
    /// some stuff used to ensure perfect reproduçability. You should probably lease this to None
    pub copied_on_previous: Option<Vec<bool>>, //indicate if a sprite can copy on the previous. Will always copy if possible if None
    pub anim_groups: Vec<Vec<Animation>>, //usize1 = start, usize2 = lenght
}

impl AnimStore {
    pub fn new<F: Read + Seek>(
        file: &mut F,
        pointer_animation_groups_table: u64,
        amount_animation_group: u16,
    ) -> Result<(AnimStore, u64), WanError> {
        //TODO: rewrite this function, it seem to be too complicated to understand
        file.seek(SeekFrom::Start(pointer_animation_groups_table))?;
        let mut anim_group_entry: Vec<Option<AnimGroupEntry>> = Vec::new();
        for animation_group_id in 0..amount_animation_group {
            let pointer = file.read_u32::<LE>()?;
            let length = file.read_u32::<LE>()?;
            if pointer != 0 && length != 0 {
                anim_group_entry.push(Some(AnimGroupEntry {
                    pointer,
                    group_lenght: length,
                    id: animation_group_id,
                }));
            } else {
                anim_group_entry.push(None);
            }
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

        // read the Animation from the animation group data
        let mut copied_on_previous = Vec::new();
        let mut anim_groups_result = Vec::new();
        let mut check_last_anim_pos = 0;

        for anim_group in anim_groups {
            match anim_group {
                None => anim_groups_result.push(Vec::new()),
                Some(anim_group_table) => {
                    let mut animation_in_group = Vec::new();
                    for animation in anim_group_table {
                        file.seek(SeekFrom::Start(animation))?;
                        copied_on_previous
                            .push(file.seek(SeekFrom::Current(0))? == check_last_anim_pos);
                        check_last_anim_pos = file.seek(SeekFrom::Current(0))?;
                        animation_in_group.push(Animation::new(file)?);
                    }
                    anim_groups_result.push(animation_in_group)
                }
            };
        }

        Ok((
            AnimStore {
                copied_on_previous: Some(copied_on_previous),
                anim_groups: anim_groups_result,
            },
            particule_table_end,
        ))
    }

    pub fn write<F: Write + Seek>(&self, file: &mut F) -> anyhow::Result<Vec<u64>> {
        let mut animations_pointer = vec![];
        let mut previous_animation: Option<&Animation> = None;
        let mut previous_pointer = None;

        let mut anim_counter = 0;
        for animation_group in &self.anim_groups {
            for animation in animation_group {
                let can_copy_on_previous = match &self.copied_on_previous {
                    None => true,
                    Some(value) => value.get(anim_counter).copied().unwrap_or(true),
                };
                let actual_pointer = file.seek(SeekFrom::Current(0))?;

                if can_copy_on_previous {
                    if let Some(p_anim) = previous_animation {
                        if *p_anim == *animation {
                            //no panic: should never panic, as previous_pointer is defined with previous_animation, and we check previous_pointer for existance
                            animations_pointer.push(previous_pointer.unwrap());
                            anim_counter += 1;
                            continue;
                        }
                    };
                };

                animations_pointer.push(actual_pointer);
                Animation::write(file, animation)?;
                previous_animation = Some(animation);
                previous_pointer = Some(actual_pointer);

                anim_counter += 1;
            }
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
            lenght: u32,
        }

        let mut anim_group_data = Vec::new();
        let mut good_anim_group_meet = false;
        let mut anim_counter = 0;
        for anim_group in &self.anim_groups {
            if anim_group.is_empty() {
                anim_group_data.push(AnimGroupData {
                    pointer: 0,
                    lenght: 0,
                });
                if good_anim_group_meet {
                    0u32.write(file)?;
                }
            } else {
                good_anim_group_meet = true;
                anim_group_data.push(AnimGroupData {
                    pointer: file.seek(SeekFrom::Current(0))? as u32,
                    lenght: anim_group.len() as u32,
                });
                for _ in anim_group {
                    sir0_animation.push(file.seek(SeekFrom::Current(0))?);
                    (animations_pointer[anim_counter] as u32).write(file)?;
                    anim_counter += 1;
                }
            }
        }

        let animation_group_reference_offset = file.seek(SeekFrom::Current(0))?;

        for data in anim_group_data {
            if data.pointer != 0 && data.lenght != 0 {
                sir0_animation.push(file.seek(SeekFrom::Current(0))?);
            }
            (data.pointer, data.lenght).write(file)?;
        }

        Ok((animation_group_reference_offset, sir0_animation))
    }
}
