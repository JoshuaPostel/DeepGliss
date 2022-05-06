use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::time::Duration;
//use std::collections::HashMap;

use ordered_float::OrderedFloat;

use vst::event::MidiEvent;

use egui::{Pos2, Rect, Shape, Stroke, Ui};

use crate::draw::theme::GLISS_THEME;
use crate::midi::paths::{BendPath, Path};
use crate::midi::{Bend, Note};
use crate::GLISS_EPOCH;

pub struct RenderedBenders {
    map: BTreeMap<OrderedFloat<f32>, Vec<RenderedBender>>,
}

impl Default for RenderedBenders {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderedBenders {
    pub fn new() -> Self {
        RenderedBenders {
            map: BTreeMap::new(),
        }
    }

    pub fn append(&mut self, rbs: Vec<RenderedBender>) {
        for rb in rbs.into_iter() {
            self.insert(rb);
        }
    }

    fn insert(&mut self, new_rb: RenderedBender) {
        let start_time = OrderedFloat(new_rb.start_time);
        match self.map.get_mut(&start_time) {
            Some(vec) => vec.push(new_rb),
            None => {
                self.map.insert(start_time, vec![new_rb]);
            }
        }
    }

    pub fn retain(&mut self, start_time: f32) {
        // TODO
        // should use end_time not key
        // refactor to BTreeMap<OrderedFloat<f32>, (end_time: Duration, Vec<RenderedBender>)> ?
        // its not actually doing anything right now
        let max_ui_window_duration = 5.0;
        //        log::info!("retain call - start_time: {start_time}");
        //        if let Some(example) = self.map.first_key_value() {
        //            log::info!("retain call - example_key: {example:?}");
        //        }
        //        log::info!("retain call - start_time: {start_time}");
        let start_time = OrderedFloat(start_time + max_ui_window_duration);
        //log::info!("retain call - start_time2: {start_time}");
        //let len_before = self.map.len();
        self.map.retain(|&key, _| key <= start_time);
        //        let len_after = self.map.len();
        //        if len_before != len_after {
        //            log::info!("retain call did something: {len_before} {len_after}");
        //        }
    }

    pub fn render(&mut self, ui: &Ui, to_screen: emath::RectTransform) {
        let mut pairs = self.map.iter_mut().peekable();
        while let Some((_, render_benders)) = pairs.next() {
            let shapes: Vec<Shape> = render_benders
                .iter()
                .flat_map(|rb| rb.render(to_screen))
                .collect();
            if let Some((next_start_time, _)) = pairs.peek() {
                // TODO
                // dont under stand the double clone incured by peek
                // refactor?
                // https://stackoverflow.com/questions/62186871/how-to-correctly-use-peek-in-rust
                //let cutoff_time = next_start_time.clone().clone().into();
                //let cutoff_time = <&ordered_float::OrderedFloat<f32>>::clone(next_start_time).clone().into();
                let cutoff_time =
                    (*<&ordered_float::OrderedFloat<f32>>::clone(next_start_time)).into();
                let cutoff = (to_screen * Pos2::new(cutoff_time, 0.0)).x;
                ui.painter()
                    .sub_region(Rect::everything_left_of(cutoff))
                    .extend(shapes);
            } else {
                ui.painter().extend(shapes);
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct RenderedBender {
    bend: Vec<Pos2>,
    hold: (Pos2, Pos2),
    pub start_time: f32,
    pub end_time: f32,
    pub stroke: Stroke,
}

impl RenderedBender {
    fn new(bend: Vec<Pos2>, hold: (Pos2, Pos2), stroke: Stroke) -> Self {
        let start_time = bend.first().expect("non-empty").x;
        let end_time = hold.1.x;
        RenderedBender {
            bend,
            hold,
            start_time,
            end_time,
            stroke,
        }
    }

    pub fn render(&self, to_screen: emath::RectTransform) -> [Shape; 2] {
        let bend_shape: Shape = if self.bend.len() == 2 {
            let p1 = to_screen * self.bend[0];
            let p2 = to_screen * self.bend[1];
            Shape::line_segment([p1, p2], self.stroke)
        } else {
            // TODO is there a way to avoid this clone?
            let points: Vec<Pos2> = self
                .bend
                .clone()
                .into_iter()
                .map(|p| to_screen * p)
                .collect();
            Shape::line(points, self.stroke)
        };

        let p1 = to_screen * self.hold.0;
        let p2 = to_screen * self.hold.1;
        let hold_shape = Shape::line_segment([p1, p2], self.stroke);

        [bend_shape, hold_shape]
    }

    // may not be nessisary, look for masking/cutoff egui utils
    pub fn truncate(&mut self, _next_bender: RenderedBender) {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Bender {
    pub active: bool,
    pub note: Note,
    pub start_time: f64,
    pub stop_time: f64,
    pub note_off_time: f64,
    pub start_bend: Bend,
    pub target_bend: Bend,
    pub current_bend: Bend,
    pub bend_path: BendPath,
    pub pitch_bend_range: f32,
}

impl Bender {
    pub fn new(
        note: &Note,
        now: f64,
        bend_duration: f64,
        hold_duration: f64,
        pitch_bend_range: f32,
        bend_path: BendPath,
    ) -> (Self, MidiEvent) {
        let bender = Self {
            active: true,
            note: *note,
            start_time: now,
            stop_time: now + bend_duration,
            note_off_time: now + bend_duration + hold_duration,
            bend_path,
            pitch_bend_range,
            ..Default::default()
        };

        (bender, note.as_midi_event())
    }

    pub fn get_render(&self) -> RenderedBender {
        let og_note = self.note.midi_number;
        let bend_start = self
            .get_bend(self.start_time)
            .expect("response due to time limits");
        let bend_stop = self
            .get_bend(self.stop_time)
            .expect("response due to time limits");

        let start_time = Duration::from_nanos(self.start_time as u64) - *GLISS_EPOCH;
        let stop_time = Duration::from_nanos(self.stop_time as u64) - *GLISS_EPOCH;
        let continuous_note1 = og_note as f32 + bend_start.continuous_semitones(self.pitch_bend_range);
        let continuous_note2 = og_note as f32 + bend_stop.continuous_semitones(self.pitch_bend_range);
        let bend: Vec<Pos2> = match self.bend_path.path {
            Path::Linear => {
                let p1 = Pos2::new(start_time.as_secs_f32(), continuous_note1);
                let p2 = Pos2::new(stop_time.as_secs_f32(), continuous_note2);

                vec![p1, p2]
            }
            _ => {
                let p1 = Pos2::new(start_time.as_secs_f32(), continuous_note1);
                let p2 = Pos2::new(stop_time.as_secs_f32(), continuous_note2);

                log::info!("old method_endpoints: {:?}", vec![p1, p2]);

                //let n_points = 50;
                let n_points = 500;
                let step = (self.stop_time - self.start_time) / n_points as f64;
                let points: Vec<Pos2> = (0..(n_points + 1))
                    .into_iter()
                    .map(|i| self.start_time + i as f64 * step)
                    .map(|t| {
                        log::debug!("time t:     {}", t);
                        (
                            (Duration::from_nanos(t as u64) - *GLISS_EPOCH).as_secs_f32(),
                            og_note as f32
                                + self
                                    .get_bend(t)
                                    .expect("response due to time limits")
                                    .continuous_semitones(self.pitch_bend_range),
                        )
                    })
                    .map(|(x, y)| Pos2::new(x, y))
                    .collect();
                log::debug!("start_time: {}", self.start_time);
                log::debug!("stop_time:  {}", self.stop_time);
                log::debug!("points: {:?}", points);
                points
            }
        };

        let continuous_note2 = og_note as f32 + bend_stop.continuous_semitones(self.pitch_bend_range);

        let stop_time = Duration::from_nanos(self.stop_time as u64) - *GLISS_EPOCH;

        let note_off_time = Duration::from_nanos(self.note_off_time as u64) - *GLISS_EPOCH;
        let p1 = Pos2::new(stop_time.as_secs_f32(), continuous_note2);
        let p2 = Pos2::new(note_off_time.as_secs_f32(), continuous_note2);
        let hold = (p1, p2);





        // TODO error check on index here or assert > 15 channles doesnt get this far?
        log::info!("self.note.channel: {}", self.note.channel);
        let color = GLISS_THEME.channel_colors.get(self.note.channel as usize - 2);
        let stroke = Stroke::new(0.5, *color.unwrap_or(&egui::Color32::WHITE));
        //let stroke = Stroke::new(0.5, Color32::GOLD);
        //let stroke = Stroke::new(0.5, Color32::from_additive_luminance(100));

        RenderedBender::new(bend, hold, stroke)
    }

    pub fn update_target(
        &mut self,
        target: &Note,
        now: f64,
        bend_duration: f64,
        hold_duration: f64,
        bend_path: BendPath,
    ) -> Result<RenderedBender, String> {
        //log::info!("update_target called with target: {target:?}");
        //log::info!("pre update_target: {self:?}");
        self.target_bend = self.note.bend_to(target, self.pitch_bend_range)?;
        self.start_bend = self.current_bend;
        self.start_time = now;
        self.stop_time = now + bend_duration;
        self.note_off_time = now + bend_duration + hold_duration;
        self.bend_path = bend_path;
        log::info!("post update_target: {self:?}");
        Ok(self.get_render())
    }

    pub fn get_bend(&self, time: f64) -> Option<Bend> {
        let start_bend = self.start_bend.0 as f64;
        let target_bend = self.target_bend.0 as f64;
        if self.start_time <= time && time <= self.stop_time {
            log::debug!("calling get_bend with: {:?}", self.bend_path);
            let bend = BendPath::bend(
                &self.bend_path,
                time,
                self.start_time,
                self.stop_time,
                start_bend,
                target_bend,
            );
            Some(bend)
        } else {
            None
        }
    }

    pub fn bend(&mut self, time: f64) -> Option<MidiEvent> {
        if self.start_time <= time && time <= self.stop_time {
            let bend = self
                .get_bend(time)
                .expect("not some due to identical time checks");
            self.current_bend = bend;
            Some(bend.as_midi_event(self.note.channel))
        } else if self.note_off_time <= time {
            log::info!("sending note off for: {}", self.note.midi_number);
            self.active = false;
            Some(MidiEvent {
                // note off
                data: [127 + self.note.channel, self.note.midi_number, 0],
                delta_frames: 0,
                live: false,
                note_length: None,
                note_offset: None,
                detune: 0,
                note_off_velocity: 0,
            })
        } else {
            None
        }
    }

    pub fn current_midi(&self) -> f32 {
        self.note.midi_number as f32 + self.current_bend.continuous_semitones(self.pitch_bend_range)
    }
}

// TODO simple write simple tests of Ord for Bender
impl Ord for Bender {
    fn cmp(&self, other: &Self) -> Ordering {
        let (left_semitones, left_remainder_bend) = self.current_bend.semitones(self.pitch_bend_range);
        let left_midi_number = self.note.midi_number as i8 + left_semitones;

        let (right_semitones, right_remainder_bend) = other.current_bend.semitones(self.pitch_bend_range);
        let right_midi_number = other.note.midi_number as i8 + right_semitones;

        match left_midi_number.cmp(&right_midi_number) {
            Ordering::Equal => left_remainder_bend.cmp(&right_remainder_bend),
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        }
    }
}

impl PartialOrd for Bender {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Bender {
    fn eq(&self, other: &Self) -> bool {
        (self.note.midi_number == other.note.midi_number)
            && (self.current_bend == other.current_bend)
    }
}

impl Eq for Bender {}
