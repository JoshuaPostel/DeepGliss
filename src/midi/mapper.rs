use crate::midi::bender::Bender;
use crate::midi::Note;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::LinkedList;

use rand::prelude::SliceRandom;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum ChordMap {
    #[default]
    Closest,
    Flipped,
    Random,
}

impl ChordMap {
    pub fn from_f32(val: f32) -> Self {
        match (val * 3.0) as u32 {
            0 => ChordMap::Closest,
            1 => ChordMap::Flipped,
            _ => ChordMap::Random,
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            ChordMap::Closest => 0.0,
            ChordMap::Flipped => 1.0 / 3.0,
            ChordMap::Random => 2.0 / 3.0,
        }
    }
}

#[cfg(test)]
mod chord_map_categorical_param {
    use super::ChordMap;

    #[test]
    fn there_and_back_again() {
        for og_val in [0.0, 0.1, 1.0 / 3.0, 2.0 / 3.0, 1.0].iter() {
            let cat = ChordMap::from_f32(*og_val);
            println!("cat: {cat:?}");
            let as_val = cat.as_f64();
            println!("as_val: {as_val}");
            let as_cat_again = ChordMap::from_f32(as_val as f32);
            println!("as_cat_again: {as_cat_again:?}");
            assert_eq!(cat, as_cat_again);
        }
    }
}

// too combinitorial?
// for MVP yes
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum MappingPreference {
    #[default]
    Closest,
    Furthest,
    Top,
    Center,
    Bottom,
    Random,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ChordMapper {
    pub chord_map: ChordMap,
    pub preference: MappingPreference,
}

impl ChordMapper {
    // TODO retrun indicies of bend_to_notes and new_notes
    pub fn get_mapping(
        self,
        channels: &[Bender],
        target_notes: &[Note],
    ) -> (Vec<usize>, Vec<usize>) {
        if channels.is_empty() {
            return (vec![], (0..target_notes.len()).collect());
        }
        //        let &mut ordered_channels  = channels.clone();
        //        ordered_channels.sort_by(|a, b| {
        //            a.current_midi().partial_cmp(&b.current_midi()).expect("no NANs")
        //        });
        // TODO handle this here?
        if target_notes.len() == 1 {
            return ((0..channels.len()).map(|_| 0).collect(), vec![]);
        }
        match self.chord_map {
            ChordMap::Random => {
                log::info!("in mapper branch random");
                get_random_mapping(channels, target_notes)
            }
            ChordMap::Flipped => get_flipped_mapping(channels, target_notes),
            ChordMap::Closest => get_closest_mapping(channels, target_notes),
        }
    }
}

fn get_random_mapping(channels: &[Bender], target_notes: &[Note]) -> (Vec<usize>, Vec<usize>) {
    let n_channels = channels.len();
    let n_target_notes = target_notes.len();
    let mut target_indicies: Vec<usize> = (0..n_target_notes).collect();
    let mut rng = rand::thread_rng();
    target_indicies.shuffle(&mut rng);
    match n_channels.cmp(&n_target_notes) {
        Less => {
            let target_notes = target_indicies[..n_channels].to_vec();
            let new_notes = target_indicies[n_channels..].to_vec();
            (target_notes, new_notes)
        }
        Equal => (target_indicies.to_vec(), vec![]),
        Greater => (
            target_indicies
                .into_iter()
                .cycle()
                .take(n_channels)
                .collect(),
            vec![],
        ),
    }
}

fn get_flipped_mapping(channels: &[Bender], target_notes: &[Note]) -> (Vec<usize>, Vec<usize>) {
    let n_channels = channels.len();
    let n_target_notes = target_notes.len();
    match n_channels.cmp(&n_target_notes) {
        Less => {
            let mut target_indicies: LinkedList<usize> = (0..n_target_notes).collect();
            let mut top_target_notes = vec![];
            let mut bot_target_notes = vec![];
            for i in 0..n_channels {
                if i % 2 == 0 {
                    top_target_notes.push(target_indicies.pop_back().expect("safe in Less branch"));
                } else {
                    bot_target_notes
                        .push(target_indicies.pop_front().expect("safe in Less branch"));
                }
            }
            for note in bot_target_notes.into_iter().rev() {
                top_target_notes.push(note);
            }
            //top_target_notes.append(&mut bot_target_notes);
            let new_notes = target_indicies.into_iter().collect();
            log::info!("target_notes: {top_target_notes:?}");
            log::info!("new_notes: {new_notes:?}");
            (top_target_notes, new_notes)
        }
        Equal => ((0..n_target_notes).into_iter().rev().collect(), vec![]),
        Greater => {
            let half: usize = n_target_notes / 2;
            let top_targets: Vec<usize> = (0..half).collect();
            let mid_targets: Vec<usize>;
            let bot_targets: Vec<usize>;
            if n_target_notes % 2 == 0 {
                mid_targets = (half - 1..half + 1).collect();
                bot_targets = (half..n_target_notes).collect();
            } else {
                mid_targets = vec![half];
                bot_targets = (half + 1..n_target_notes).collect();
            }
            log::info!("top_targets: {top_targets:?}");
            log::info!("mid_targets: {mid_targets:?}");
            log::info!("bot_targets: {bot_targets:?}");

            let mut top_channels = vec![];
            let mut bot_channels = vec![];

            log::info!("n_channels: {n_channels}");
            log::info!("n_target_notes: {n_target_notes}");
            let group_size: usize =
                if (n_target_notes % 2 != 0) && (n_channels % n_target_notes != 0) {
                    n_channels / (n_target_notes - 1)
                } else {
                    n_channels / n_target_notes
                };
            log::info!("group_size: {group_size}");

            for target in top_targets {
                for _ in 0..group_size {
                    bot_channels.push(target);
                }
            }
            for target in bot_targets {
                for _ in 0..group_size {
                    top_channels.push(target);
                }
            }
            log::info!("top_channels: {top_channels:?}");
            log::info!("bot_channels: {bot_channels:?}");

            let n_mid_channels = (n_channels - top_channels.len()) - bot_channels.len();
            log::info!("n_mid_channels: {n_mid_channels}");

            let mut mid_channels: Vec<usize> = mid_targets
                .into_iter()
                .cycle()
                .take(n_mid_channels)
                .collect();

            log::info!("mid_channels: {mid_channels:?}");

            top_channels.append(&mut mid_channels);
            top_channels.append(&mut bot_channels);
            log::info!("output_channels: {top_channels:?}");
            (top_channels, vec![])
        }
    }
}

fn get_closest_mapping(channels: &[Bender], target_notes: &[Note]) -> (Vec<usize>, Vec<usize>) {
    log::info!("channels: {channels:?}");
    log::info!("target_notes: {target_notes:?}");
    let mut n_channels = channels.len();
    let mut n_target_notes = target_notes.len();
    match n_channels.cmp(&n_target_notes) {
        Less => {
            let mut target_note_indicies: Vec<usize> = vec![];
            let channel_midis: Vec<f32> = channels
                .iter()
                .map(|bender| bender.current_midi())
                .collect();
            let mut target_midis: Vec<(usize, f32)> = target_notes
                .iter()
                .map(|note| note.midi_number as f32)
                .enumerate()
                .collect();
            for midi in channel_midis {
                log::info!("target_midis before: {:?}", target_midis);
                log::info!("looking for closes to: {}", midi);
                let (closest_index, _min_distance) = target_midis
                    .iter()
                    .map(|(idx, target_midi)| (idx, (midi - target_midi).abs()))
                    // using reduce to sort f32
                    // TODO revisit, migth be able to do with sort_by:
                    // https://doc.rust-lang.org/std/primitive.slice.html#method.sort_by
                    .reduce(|(current_i, current_min), (i, midi_distance)| {
                        if current_min <= midi_distance {
                            (current_i, current_min)
                        } else {
                            (i, midi_distance)
                        }
                    })
                    .expect("minimum to exist");
                let closest_index = *closest_index;
                log::info!("closest_index: {closest_index}");
                target_note_indicies.push(closest_index);
                target_midis.retain(|(i, _)| i != &closest_index);
                log::info!("target_midis after: {:?}", target_midis);
            }
            let new_note_indicies = target_midis.into_iter().map(|(i, _)| i).collect();
            log::info!("target_note_indicies: {:?}", target_note_indicies);
            log::info!("new_note_indicies: {:?}", new_note_indicies);
            (target_note_indicies, new_note_indicies)
        }
        Equal => ((0..n_target_notes).collect(), vec![]),
        Greater => {
            let mut target_notes = vec![];
            for note_idx in 0..n_target_notes {
                let group_size = n_channels / n_target_notes;
                for _ in 0..group_size {
                    target_notes.push(note_idx);
                }
                n_channels -= group_size;
                n_target_notes -= 1;
            }
            log::info!("closest target_notes: {target_notes:?}");
            (target_notes, vec![])
        }
    }
}
