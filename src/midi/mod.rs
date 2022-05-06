pub mod bender;
pub mod chord;
pub mod mapper;
pub mod paths;

use std::time::Duration;

use vst::event::MidiEvent;

use crate::GLISS_EPOCH;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Bend(pub u16);

// default in no pitch bend
impl Default for Bend {
    fn default() -> Self {
        Bend(8192)
    }
}

impl Bend {
    // midi uses 7-bits of two bytes to represent the amount of pitch bend as a u14
    pub fn new(msb: u8, lsb: u8) -> Self {
        let mut u14 = u16::from_be_bytes([msb, 0]);
        u14 >>= 1;
        u14 |= u16::from_be_bytes([0, lsb]);
        Bend(u14)
    }

    pub fn as_midi_event(&self, channel: u8) -> MidiEvent {
        if self.0 <= 16_383 {
            let mut u14 = self.0;
            u14 <<= 1;
            let [msb, mut lsb] = u14.to_be_bytes();
            lsb >>= 1;

            MidiEvent {
                data: [223 + channel, lsb, msb],
                delta_frames: 0,
                live: false,
                note_length: None,
                note_offset: None,
                detune: 0,
                note_off_velocity: 0,
            }
        } else {
            // if value exceed max of u14, send max pitch bend
            MidiEvent {
                data: [223 + channel, 127, 127],
                delta_frames: 0,
                live: false,
                note_length: None,
                note_offset: None,
                detune: 0,
                note_off_velocity: 0,
            }
        }
    }

    pub fn semitones(self, pitch_bend_range: f32) -> (i8, i8) {
        let percentage_of_range = (self.0 as f32 - 8_192.0) / 8_192.0;
        let semitones = pitch_bend_range * percentage_of_range;
        let bend_remainder = semitones.fract() / pitch_bend_range * 8_192.0;
        let whole_semitones = semitones.trunc() as i8;
        let whole_bend_remainder = bend_remainder.trunc() as i8;
        (whole_semitones, whole_bend_remainder)
    }

    pub fn continuous_semitones(self, pitch_bend_range: f32) -> f32 {
        let percentage_of_range = (self.0 as f32 - 8_192.0) / 8_192.0;
        pitch_bend_range as f32 * percentage_of_range
    }
}

#[cfg(test)]
mod u14_pitch_bend_io {
    use super::Bend;

    #[test]
    fn from_lsb_120_msb_95() {
        let msb = 95;
        let lsb = 120;
        let bend = Bend::new(msb, lsb);
        assert_eq!(Bend(12_280u16), bend);
    }

    #[test]
    fn to_midi_lsb_120_msb_95() {
        let midi_event = Bend(12_280u16).as_midi_event(1);
        assert_eq!(midi_event.data, [224, 120, 95]);
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Note {
    pub channel: u8,
    pub midi_number: u8,
    pub daw_time: f64,
    pub bend_duration: f64,
    pub ui_time: Duration,
    pub new_note_on: bool,
    pub key_released: bool,
}

impl Note {
    //pub fn new(midi_data: [u8; 3], daw_time: f64, ui_time: Duration) -> Result<Self, String> {
    pub fn new(midi_data: [u8; 3], daw_time: f64, bend_duration: f64) -> Result<Self, String> {
        if daw_time.is_nan() {
            return Err("Note does not allow NAN time".to_string());
        }
        let channel: u8 = match midi_data[0] {
            // do we ever need to handle this case?
            // note off
            //128..=143 => channel = midi_data[0] - 127,
            // note on
            144..=159 => midi_data[0] - 143,
            _ => return Err(format!("midi_data: {midi_data:?} is not note on")),
        };
        log::info!(
            "creatd note(channel: {channel}, midi_number: {midi_number})  from: {midi_data:?}",
            midi_number = midi_data[1]
        );
        //let ui_time = Duration::from_nanos(daw_time as u64);
        let ui_time = Duration::from_nanos(daw_time as u64) - *GLISS_EPOCH;
        Ok(Self {
            channel,
            midi_number: midi_data[1],
            daw_time,
            bend_duration,
            ui_time,
            new_note_on: false,
            key_released: false,
        })
    }

    pub fn as_midi_event(&self) -> MidiEvent {
        MidiEvent {
            // TODO need to capture velocity instead of default 64
            data: [143 + self.channel, self.midi_number, 64],
            delta_frames: 0,
            live: false,
            note_length: None,
            note_offset: None,
            detune: 0,
            note_off_velocity: 0,
        }
    }

    // TODO or just return Bend to max?
    pub fn bend_to(&self, target: &Note, pitch_bend_range: f32) -> Result<Bend, String> {
        let n_semitones = target.midi_number as f32 - self.midi_number as f32;
        log::info!("bend_to n_semitones {}", n_semitones);
        let pitch_bend_ratio = n_semitones / pitch_bend_range;
        log::info!("bend_to pitch_bend_ratio {}", pitch_bend_ratio);
        match pitch_bend_ratio {
            ratio if (-1.0..=1.0).contains(&ratio) => {
                // map pitch_bend_ratio
                let midi_bend = (8192.0 * (ratio + 1.0)) as u16;
                log::info!("bend_to midi_bend: {}", midi_bend);
                Ok(Bend(midi_bend))
            }
            _ => Err(format!(
                "attempted to bend {n_semitones} semitones when max is {pitch_bend_range}"
            )),
        }
    }
}
