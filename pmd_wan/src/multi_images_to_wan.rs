//TODO: add handling for symetric fragment
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    convert::TryInto,
};

use crate::{
    encode_fragment_pixels, find_fragments_in_images, fragment_finder::FragmentUse,
    pad_seven_pixel, Fragment, FragmentBytes, FragmentFinderData, FragmentFlip, OamShape,
    Frame, GeneralResolution, NormalizedBytes, SpriteType, VariableNormalizedBytes, WanImage,
};
use anyhow::{bail, Context};

#[derive(Debug, Clone, Copy)]
struct ImageStartDelta {
    delta_x: i8,
    delta_y: i8,
}

impl ImageStartDelta {
    fn new(selected_x: i32, selected_y: i32) -> Self {
        fn get_appropriate_value(value: i32) -> i8 {
            if value % 8 == 0 {
                0
            } else {
                -8 + ((value % 8) as i8)
            }
        }
        Self {
            delta_x: get_appropriate_value(selected_x),
            delta_y: get_appropriate_value(selected_y),
        }
    }
}

pub fn create_wan_from_multiple_images(
    images: &[(&[u8], GeneralResolution)],
    sprite_type: SpriteType,
) -> anyhow::Result<WanImage> {
    //high level overview of how this work :
    //1. Get fragments (8 by 8) usage stats
    //2. For each images, get the most used fragment with at least 75% of non-null surface covered. If none are found, remove the 75% requirement. Otherwise, this a fully transparent frame.
    //3. Tile the other fragments
    //4. Optimise the allocation by putting together fragments to form bigger fragment
    //5. Assemble all of this
    if images.len() >= u16::MAX as usize {
        bail!(
            "The max number of image is {}, but it is {}.",
            u16::MAX,
            images.len()
        )
    }
    // step 1 and 2
    let images_deltas =
        get_images_delta(images).context("while trying to get the images deltas")?;

    // step 3
    let mut bigger_fragment_finder_builder = BiggerFragmentFinderBuilder::new(images.len() as u16);
    for (image_id, (start_delta, (image_bytes, image_resolution))) in
        images_deltas.iter().zip(images).enumerate()
    {
        bigger_fragment_finder_builder.add_from_image(
            image_bytes,
            image_resolution.clone(),
            *start_delta,
            image_id as u16,
        );
    }
    let bigger_fragment_finder = bigger_fragment_finder_builder.build();

    // initialise wan and Frames
    let mut wan = WanImage::new(sprite_type);
    wan.frame_store.frames = vec![Frame::default(); images.len()];

    // step 4 and 5 are combined
    bigger_fragment_finder.find_and_apply_on_wan(&mut wan);

    wan.fix_empty_frames();
    Ok(wan)
}

fn get_images_delta(images: &[(&[u8], GeneralResolution)]) -> anyhow::Result<Vec<ImageStartDelta>> {
    let fragments_use: FragmentFinderData = find_fragments_in_images(images)
        .context("Trying to find statistic about fragments usage")?;
    let fragment_ordered_by_usage = fragments_use.order_by_usage();
    let mut result = Vec::new();
    for (image_id, _image) in images.iter().enumerate() {
        fn find_most_used_tile(
            ordered: &[(&NormalizedBytes, &Vec<FragmentUse>)],
            image_id: u16,
            limit: bool,
        ) -> Option<(NormalizedBytes, FragmentUse)> {
            for (fragment, all_usage) in ordered.iter().map(|(x, y)| (*x, *y)) {
                for usage in all_usage {
                    if usage.image_id == image_id
                        && !limit
                        && fragment.0.iter().filter(|x| **x != 0).count() > (64 / 4) * 3
                    {
                        return Some((*fragment, *usage));
                    }
                }
            }
            None
        }

        let mut appropriate_fragment =
            find_most_used_tile(&fragment_ordered_by_usage, image_id as u16, true);
        if appropriate_fragment.is_none() {
            appropriate_fragment =
                find_most_used_tile(&fragment_ordered_by_usage, image_id as u16, false)
        }
        let base_coordinates = if let Some((_, fragment_use)) = appropriate_fragment {
            (fragment_use.x, fragment_use.y)
        } else {
            (0, 0)
        };
        result.push(ImageStartDelta::new(base_coordinates.0, base_coordinates.1))
    }

    Ok(result)
}

#[derive(Debug)]
struct BiggerFragmentFinderBuilder {
    presence: HashMap<NormalizedBytes, (Vec<bool>, BTreeSet<FragmentUse>)>,
    number_of_images: u16,
}

impl BiggerFragmentFinderBuilder {
    fn new(number_of_images: u16) -> Self {
        Self {
            presence: HashMap::default(),
            number_of_images,
        }
    }

    fn add_from_image(
        &mut self,
        image_bytes: &[u8],
        resolution: GeneralResolution,
        delta: ImageStartDelta,
        image_id: u16,
    ) {
        let (padded_image, padded_resolution) =
            pad_seven_pixel(image_bytes, resolution.clone()).unwrap();
        let pixel_start_in_padded_image = (
            if delta.delta_x == 0 {
                0
            } else {
                delta.delta_x + 7
            },
            if delta.delta_y == 0 {
                0
            } else {
                delta.delta_y + 7
            },
        );
        let loop_number_by_side = (
            (-delta.delta_x as u32 + resolution.x + 7) / 8,
            (-delta.delta_y as u32 + resolution.y + 7) / 8,
        );
        for global_fragment_position_y in 0..loop_number_by_side.1 {
            let fragment_start_y =
                global_fragment_position_y * 8 + pixel_start_in_padded_image.1 as u32;
            for global_fragment_position_x in 0..loop_number_by_side.0 {
                let fragment_start_x =
                    global_fragment_position_x * 8 + pixel_start_in_padded_image.0 as u32;
                let mut fragment_buffer = [0; 64];
                for special_line in 0..8 {
                    let pixel_base =
                        (special_line + fragment_start_y) * padded_resolution.x + fragment_start_x;
                    fragment_buffer[special_line as usize * 8..special_line as usize * 8 + 8]
                        .copy_from_slice(
                            &padded_image[pixel_base as usize..pixel_base as usize + 8],
                        );
                }

                if fragment_buffer == [0; 64] {
                    continue;
                }

                let (normalized_bytes, flip) = NormalizedBytes::new(fragment_buffer);

                self.add_use(
                    normalized_bytes,
                    FragmentUse {
                        x: fragment_start_x as i32 - 7,
                        y: fragment_start_y as i32 - 7,
                        image_id,
                        flip,
                    },
                );
            }
        }
    }

    fn add_use(&mut self, bytes: NormalizedBytes, usage: FragmentUse) {
        let image_nb = self.number_of_images;
        let entry = self
            .presence
            .entry(bytes)
            .or_insert_with(|| (vec![false; image_nb as usize], BTreeSet::new()));
        entry.0[usage.image_id as usize] = true;
        entry.1.insert(usage);
    }

    fn build(self) -> BiggerFragmentFinder {
        let mut usage_by_image = BTreeMap::new();
        for (normalized_bytes, (presence, usages)) in self.presence {
            usage_by_image
                .entry(presence)
                .or_insert_with(HashMap::new)
                .insert(normalized_bytes, usages);
        }

        BiggerFragmentFinder { usage_by_image }
    }
}

#[derive(Debug, Clone)]
struct BiggerFragmentFinder {
    //TODO: somehow use a bitset instead
    usage_by_image: BTreeMap<Vec<bool>, HashMap<NormalizedBytes, BTreeSet<FragmentUse>>>,
}

impl BiggerFragmentFinder {
    fn find_and_apply_on_wan(self, wan: &mut WanImage) {
        for (_, group) in self.usage_by_image.into_iter() {
            FindBiggerFragmentOnSingleGroupStruct::process(group, wan);
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct FragmentPosition {
    x: i32,
    y: i32,
    image_id: u16,
}

impl FragmentPosition {
    fn to_fragment_use(&self, flip: FragmentFlip) -> FragmentUse {
        FragmentUse {
            x: self.x,
            y: self.y,
            image_id: self.image_id,
            flip,
        }
    }
}

struct FindBiggerFragmentOnSingleGroupStruct<'a> {
    group: HashMap<NormalizedBytes, BTreeSet<FragmentUse>>,
    lookup_by_use: HashMap<FragmentPosition, (NormalizedBytes, FragmentFlip)>,
    wan: &'a mut WanImage,
}

impl<'a> FindBiggerFragmentOnSingleGroupStruct<'a> {
    fn process(group: HashMap<NormalizedBytes, BTreeSet<FragmentUse>>, wan: &'a mut WanImage) {
        let mut lookup_by_use = HashMap::new();
        for (key, value) in group.iter() {
            for usage in value {
                lookup_by_use.insert(
                    FragmentPosition {
                        x: usage.x,
                        y: usage.y,
                        image_id: usage.image_id,
                    },
                    (*key, usage.flip),
                );
            }
        }

        let mut s = Self {
            group,
            lookup_by_use,
            wan,
        };

        for (shape_indice, size_indice) in [
            (0, 3),
            (2, 3),
            (1, 3),
            (0, 2),
            (2, 2),
            (1, 2),
            (0, 1),
            (2, 1),
            (1, 1),
            (1, 0),
            (2, 0),
        ].into_iter() {
            let resolution = OamShape::new(shape_indice, size_indice).unwrap();
            s.process_resolution(resolution);
        }

        //TODO: execute a second time, for those who don’t have duplicate. May further reduce the file size.

        for (bytes, use_of_this_byte) in s.group.into_iter() {
            // TODO: this is mostly copy–pasted from the process_resolution function
            // add the bytes
            let image_bytes_index = s.wan.fragment_bytes_store.len();
            s.wan
                .fragment_bytes_store
                .fragment_bytes
                .push(FragmentBytes {
                    mixed_pixels: encode_fragment_pixels(&bytes.0, OamShape::new(0, 0).unwrap().size())
                        .unwrap(),
                    z_index: 0,
                });
            // and their usage
            for usage in use_of_this_byte {
                let frame = &mut s.wan.frame_store.frames[usage.image_id as usize];
                frame.fragments.push(Fragment {
                    unk1: 0,
                    unk3_4: None,
                    unk5: false,
                    fragment_bytes_index: image_bytes_index,
                    offset_y: usage.y.try_into().unwrap(),
                    offset_x: usage.x.try_into().unwrap(),
                    flip: usage.flip,
                    is_mosaic: false,
                    pal_idx: 0,
                    resolution: OamShape::new(0, 0).unwrap(),
                });
            }
        }
    }

    fn process_resolution(&mut self, resolution: OamShape) {
        //TODO: better optimisation
        let mut remaining_fragments_to_check = BTreeSet::new();
        for key in self.group.keys() {
            remaining_fragments_to_check.insert(*key);
        }

        let nb_chunk_x = resolution.size().x / 8;
        let nb_chunk_y = resolution.size().y / 8;

        let max_unused_chunk = if nb_chunk_x * nb_chunk_y <= 2 {
            0
        } else {
            (nb_chunk_x * nb_chunk_y) / 2
        };

        let mut normal_chunk_line = vec![vec![0; 64]; nb_chunk_x as usize];
        let mut bigger_fragment: Vec<u8> = Vec::with_capacity(resolution.size().nb_pixels() as usize);
        'next_fragment: while let Some(possible_fragment) = {
            //NOTE: use pop_last (or pop_first) when stabilized
            if let Some(selected) = { remaining_fragments_to_check.iter().next().copied() } {
                remaining_fragments_to_check.remove(&selected);
                Some(selected)
            } else {
                None
            }
        } {
            for relative_chunk_start_x in 0..nb_chunk_x {
                // for each possible horizontal placement of this 8×8 fragment in the bigger fragment
                let relative_start_x = relative_chunk_start_x as i32 * -8;
                'skip_fragment_positon: for relative_chunk_start_y in 0..nb_chunk_y {
                    // idem for vertical
                    let relative_start_y = relative_chunk_start_y as i32 * -8;
                    let mut base_bigger_fragment: Option<VariableNormalizedBytes> = None;
                    let mut all_big_fragment: Vec<(FragmentPosition, FragmentFlip)> = Vec::new();
                    let mut used_fragments: HashMap<NormalizedBytes, BTreeSet<FragmentUse>> =
                        HashMap::new();
                    let mut nb_unused_chunk = 0;
                    // let’s check one possible placement
                    for usage in self.group.get(&possible_fragment).unwrap() {
                        bigger_fragment.clear();
                        for small_fragment_line in 0..nb_chunk_y {
                            for small_fragment_row in 0..nb_chunk_x {
                                let target_fragment_position = FragmentPosition {
                                    x: usage.x + relative_start_x + small_fragment_row as i32 * 8,
                                    y: usage.y + relative_start_y + small_fragment_line as i32 * 8,
                                    image_id: usage.image_id,
                                };
                                if let Some((norm_bytes, flip)) =
                                    self.lookup_by_use.get(&target_fragment_position)
                                {
                                    flip.apply(
                                        &norm_bytes.0,
                                        GeneralResolution::new(8, 8),
                                        &mut normal_chunk_line[small_fragment_row as usize],
                                    )
                                    .unwrap();
                                    used_fragments
                                        .entry(*norm_bytes)
                                        .or_insert_with(BTreeSet::new)
                                        .insert(target_fragment_position.to_fragment_use(*flip));
                                } else {
                                    normal_chunk_line[small_fragment_row as usize] = vec![0; 64];
                                    nb_unused_chunk += 1;
                                    if nb_unused_chunk > max_unused_chunk {
                                        continue 'skip_fragment_positon;
                                    }
                                }
                            }
                            for inner_line in 0..8 {
                                for inner_fragment in &normal_chunk_line {
                                    bigger_fragment.extend_from_slice(
                                        &inner_fragment[8 * inner_line..8 * inner_line + 8],
                                    );
                                }
                            }
                        }

                        let (normalized_bigger_fragment, bigger_flip) =
                            VariableNormalizedBytes::new(&bigger_fragment, resolution.size());
                        if let Some(base_bigger_fragment) = &base_bigger_fragment {
                            if &normalized_bigger_fragment != base_bigger_fragment {
                                continue 'skip_fragment_positon;
                            }
                        } else {
                            base_bigger_fragment = Some(normalized_bigger_fragment)
                        }

                        all_big_fragment.push((
                            FragmentPosition {
                                x: usage.x + relative_start_x,
                                y: usage.y + relative_start_y,
                                image_id: usage.image_id,
                            },
                            bigger_flip,
                        ))
                    }

                    // Lastly, make sure there are no fragment in this big fragment that is also used outside of it.
                    for (bytes, used) in &used_fragments {
                        if self.group.get(bytes).unwrap() != used {
                            continue 'skip_fragment_positon;
                        }
                    }
                    // Yay, we found a bunch of big fragment we can finally push that to Wan
                    // push the bytes
                    let image_bytes_index = self.wan.fragment_bytes_store.len();
                    self.wan
                        .fragment_bytes_store
                        .fragment_bytes
                        .push(FragmentBytes {
                            mixed_pixels: encode_fragment_pixels(
                                &base_bigger_fragment.unwrap().0,
                                resolution.size(),
                            )
                            .unwrap(),
                            z_index: 0,
                        });
                    // and their usage
                    for (position, flip) in all_big_fragment {
                        self.wan.frame_store.frames[position.image_id as usize]
                            .fragments
                            .push(Fragment {
                                unk1: 0,
                                unk3_4: None,
                                unk5: false,
                                fragment_bytes_index: image_bytes_index,
                                offset_y: position.y.try_into().unwrap(),
                                offset_x: position.x.try_into().unwrap(),
                                flip,
                                is_mosaic: false,
                                pal_idx: 0,
                                resolution,
                            });
                    }
                    // And let’s no forget to clean all this!
                    for (bytes, used) in &used_fragments {
                        remaining_fragments_to_check.remove(bytes);
                        self.group.remove(bytes);
                        for usage in used {
                            self.lookup_by_use.remove(&FragmentPosition {
                                x: usage.x,
                                y: usage.y,
                                image_id: usage.image_id,
                            });
                        }
                    }
                    continue 'next_fragment;
                }
            }
        }
    }
}
