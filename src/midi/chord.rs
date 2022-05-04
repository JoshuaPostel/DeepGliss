use std::time::{Duration, Instant};

use vst::event::MidiEvent;

use crate::midi::bender::{Bender, RenderedBender};
use crate::midi::mapper::ChordMapper;
use crate::midi::paths::{BendPathBuilder, BendPath};
use crate::midi::Note;
use crate::GLISS_EPOCH;

pub enum ChordAppendError {
    Early,
    Late,
    Exists,
}

// cannot be be empty
// notes are sorted by midi number
#[derive(Debug, Clone)]
pub struct Chord {
    pub notes: Vec<Note>,
    pub start_time: f64,
    pub capture_duration: f64,
    pub sent_to_bender: bool,
}

impl Chord {
    pub fn new(note: Note, capture_duration: f64) -> Self {
        Self {
            notes: vec![note],
            start_time: note.daw_time,
            capture_duration,
            sent_to_bender: false,
        }
    }

    pub fn from_notes(mut notes: Vec<Note>, capture_duration: f64) -> Result<Self, Vec<Note>> {
        notes.sort_by(|l, r| l.midi_number.cmp(&r.midi_number));
        let earliest_time = notes.iter().map(|n| n.daw_time).min_by(|l, r| {
            l.partial_cmp(r)
                .expect("Note does not contain NANs daw_times")
        });

        if let Some(start_time) = earliest_time {
            Ok(Self {
                notes,
                start_time,
                capture_duration,
                sent_to_bender: false,
            })
        } else {
            Err(notes)
        }
    }

    pub fn done_capturing(&self, now: f64) -> bool {
        now > self.start_time + self.capture_duration
    }

    pub fn append(&mut self, note: Note) -> Result<(), ChordAppendError> {
        if note.daw_time < self.start_time - self.capture_duration {
            return Err(ChordAppendError::Early);
        }
        if self.start_time + self.capture_duration < note.daw_time {
            return Err(ChordAppendError::Late);
        }
        match self
            .notes
            .binary_search_by(|n| n.midi_number.cmp(&note.midi_number))
        {
            Ok(_) => Err(ChordAppendError::Exists),
            Err(idx) => {
                self.notes.insert(idx, note);
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct ChordBender {
    pub init_time: Instant,
    pub bend_duration: f64,
    pub hold_duration: f64,
    pub chord_capture_duration: f64,
    pub chords: Vec<Chord>,
    pub channels: Vec<Bender>,
    pub bend_path: BendPathBuilder,
    pub chord_mapper: ChordMapper,
}

impl ChordBender {
    pub fn new(
        init_time: Instant,
        bend_duration: f64,
        hold_duration: f64,
        chord_capture_duration: f64,
    ) -> Self {
        log::info!("creating ChordBender");

        Self {
            init_time,
            bend_duration,
            hold_duration,
            chord_capture_duration,
            // We only ever need two chords?
            // so use a different struct?
            // array or ringbuffer sort of thing?
            chords: vec![],
            channels: vec![],
            bend_path: BendPathBuilder::default(),
            chord_mapper: ChordMapper::default(),
        }
    }

    // TODO
    // not sure why we cant call &self here so that bend_duraion and hold_duration are implicit
    //pub fn new_channel(&self, channels: &mut Vec<Bender>, note: &mut Note, now: f64) -> Option<MidiEvent> {
    fn new_channel(
        channels: &mut Vec<Bender>,
        note: &mut Note,
        now: f64,
        bend_duration: f64,
        hold_duration: f64,
        bend_path: BendPath,
    ) -> Option<(MidiEvent, RenderedBender)> {
        let channel: u8 = match channels.iter().map(|bender| bender.note.channel).max() {
            Some(max_channel) if (2..=16).contains(&max_channel) => max_channel + 1,
            None => 2,
            Some(max_channel) => {
                log::warn!("attempted to create a new_channel, but max_channel {max_channel} out of bounds");
                return None;
            }
        };
        note.channel = channel;
        note.new_note_on = true;
        log::info!("new_channel called with bend_path: {bend_path:?}");
        let (bender, new_note_event) =
            Bender::new(note, now, bend_duration, hold_duration, bend_path);
            //Bender::new(note, now, bend_duration, hold_duration, new_path);
        let renderable = bender.get_render();
        channels.push(bender);
        Some((new_note_event, renderable))
    }

    fn sort_channels(&mut self) {
        log::info!("channels before sort: {:?}", self.channels);
        self.channels.sort_by(|a, b| {
            a.current_midi()
                .partial_cmp(&b.current_midi())
                .expect("no NANs")
        });
        log::info!("channels after sort: {:?}", self.channels);
    }

    pub fn push_event(&mut self, event: MidiEvent, host_time: f64) {
        match event.data[0] {
            // midi note on
            144..=159 => {
                if let Ok(note) = Note::new(event.data, host_time) {
                    log::info!("push_event called with: {:?}", event.data);
                    match self.chords.last_mut() {
                        None => {
                            log::info!("in None branch");
                            let chord = Chord::new(note, self.chord_capture_duration);
                            self.chords.push(chord);
                        }
                        Some(previous_chord) => {
                            log::info!("in Some branch: {previous_chord:?}");
                            match previous_chord.append(note) {
                                Ok(_) => (),
                                Err(ChordAppendError::Late) => {
                                    log::info!("attempted to append Late chord");
                                    let chord = Chord::new(note, self.chord_capture_duration);
                                    self.chords.push(chord);
                                }
                                Err(ChordAppendError::Early) => {
                                    log::info!("attempted to append Early chord")
                                }
                                Err(ChordAppendError::Exists) => {
                                    log::info!("attempted to append existing note")
                                }
                            }
                        }
                    }
                }
            }
            // TODO this is not midi note on, what is it?
            // midi note on
            128..=143 => match self.chords.last_mut() {
                None => (),
                Some(previous_chord) => {
                    for note in previous_chord
                        .notes
                        .iter_mut()
                        .filter(|note| note.midi_number == event.data[1])
                    {
                        note.key_released = true;
                    }
                }
            },
            _ => (),
        }
    }

    // TODO return Renerers
    fn update_target_chord(&mut self, now: f64) -> Result<(Vec<MidiEvent>, Vec<RenderedBender>), String> {
        //fn update_target_chord(&mut self, now: f64) -> Vec<MidiEvent> {
        self.sort_channels();
        let mut chord = self.chords.last_mut().expect("chords to be non-enpty");
        chord.sent_to_bender = true;
        let note_on_time = chord.start_time + chord.capture_duration;
        let note_on_time_dur = Duration::from_nanos(note_on_time as u64) - *GLISS_EPOCH;
        for note in chord.notes.iter_mut() {
            note.ui_time = note_on_time_dur;
        }

        let mut midi_events = vec![];
        let mut renderables = vec![];
        log::info!("update_target_chord called with: {chord:?}");
        let n_channels = self.channels.len();
        let n_notes = chord.notes.len();
        log::info!("n_notes: {n_notes}, n_channels: {n_channels}");

        log::info!("notes before mapper: {:?}", chord.notes);
        let (target_note_indicies, new_note_indicies) =
            self.chord_mapper.get_mapping(&self.channels, &chord.notes);

        // for testing how total randomness sounds
        //self.bend_path.path = None;

        let (bend_duration, hold_duration) = if target_note_indicies.is_empty() {
            (self.hold_duration, 0.0)
        } else {
            (self.bend_duration, self.hold_duration)
        };
        for new_note_idx in new_note_indicies {
            if let Some((new_midi_event, renderable)) = ChordBender::new_channel(
                &mut self.channels,
                &mut chord.notes[new_note_idx],
                now,
                bend_duration,
                hold_duration,
                BendPath::default(),
            ) {
                //new_midi_events.push(new_midi_event);
                midi_events.push(new_midi_event);
                renderables.push(renderable);
            }
            //midi_events.append(&mut new_midi_events);
        }
        log::info!(
            "chord_bender bend_path pre channel update: {:?}",
            self.bend_path
        );
        //for (channel, note) in self.channels.iter_mut().zip(notes.into_iter()) {
        for (channel, target_note_idx) in self.channels.iter_mut().zip(target_note_indicies) {
            //        for (channel, note) in mapping {
            let renderable = channel.update_target(
                &chord.notes[target_note_idx],
                now,
                self.bend_duration,
                self.hold_duration,
                self.bend_path.build(),
            )?;
            renderables.push(renderable);
        }

        log::info!("done update_target_chord:\n{:?}", self);
        Ok((midi_events, renderables))
    }

    // TODO return Renerers
    //  -> (Vec<MidiEvent>, Vec<Renderable>) {
    pub fn bend(&mut self, time: f64) -> Result<(Vec<MidiEvent>, Vec<RenderedBender>), String> {
        let mut events = vec![];
        let mut renderables = vec![];

        if let Some(chord) = self.chords.last() {
            if !chord.sent_to_bender && chord.done_capturing(time) {
                let (mut new_events, mut new_renderables) = self.update_target_chord(time)?;
                events.append(&mut new_events);
                renderables.append(&mut new_renderables);
            }
        }

        for channel in &mut self.channels {
            log::debug!("channel: {channel:?}");
            if let Some(event) = channel.bend(time) {
                log::debug!("bend: {:?}", event.data);
                events.push(event);
            }
        }
        //self.channels.retain(|&bender| bender.active);
        self.channels.retain(|bender| bender.active);
        Ok((events, renderables))
    }
}
