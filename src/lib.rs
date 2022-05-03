#![feature(div_duration)]
#![feature(map_first_last)]
#![feature(derive_default_enum)]
#![allow(dead_code)]

#[macro_use]
extern crate vst;
#[macro_use]
extern crate lazy_static;

pub mod draw;
pub mod midi;
pub mod state;
pub mod ui;

use crate::midi::mapper::ChordMap;
use crate::midi::paths::Path;
use crate::state::EditorState;
use crate::state::GlissParam::{
    BendDuration, BendMapping, BendPath, BendPathAmplitude, BendPathPeriods,
    BendPathSCurveSharpness, HoldDuration, BendPathPhase,
};
use crate::ui::GlissEditor;

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::vst::host::Host;
use vst::buffer::{AudioBuffer, SendEventBuffer};
use vst::editor::Editor;
use vst::event::{Event, MidiEvent};
use vst::plugin::{CanDo, Category, HostCallback, Info, Plugin, PluginParameters};

//pub const PITCH_BEND_RANGE: u8 = 48;
pub const PITCH_BEND_RANGE: u8 = 24;

lazy_static! {
    pub static ref GLISS_EPOCH: Duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("not before the 70s")
        - Duration::from_secs(600);
}

struct Nano;

impl Nano {
    pub const SECOND: f64 = 1_000_000_000.0;
}

struct Gliss {
    state: Arc<EditorState>,
    editor: Option<GlissEditor>,
    host: HostCallback,
    send_buffer: SendEventBuffer,
    events_buffer: Vec<vst::event::MidiEvent>,
}

impl Default for Gliss {
    fn default() -> Self {
        let host = HostCallback::default();
        Self::new(host)
    }
}

fn get_note_events(events: &vst::api::Events) -> Vec<MidiEvent> {
    let mut notes = vec![];
    for e in events.events() {
        if let Event::Midi(midi_event) = e {
            match midi_event.data[0] {
                // note off
                128..=143 => notes.push(midi_event),
                // note on
                144..=159 => notes.push(midi_event),
                _ => (),
            }
            //            if let 144..=159 = midi_event.data[0] {
            //                notes.push(midi_event)
            //            }
        }
    }
    notes
}

impl Plugin for Gliss {
    fn new(host: HostCallback) -> Self {
        let state = Arc::new(EditorState::new());
        Self {
            state: state.clone(),
            editor: Some(GlissEditor {
                state,
                window_handle: None,
                is_open: false,
            }),
            host,
            send_buffer: SendEventBuffer::default(),
            events_buffer: vec![],
        }
    }

    fn get_info(&self) -> Info {
        log::info!("called get_info");
        Info {
            name: "DeepGliss".to_string(),
            vendor: "JoshuaPostel".to_string(),
            unique_id: 243123073,
            version: 3,
            inputs: 2,
            outputs: 2,
            parameters: 8,
            category: Category::Effect,
            midi_outputs: 1,
            ..Default::default()
        }
    }

    // TODO can we set defaults here?
    fn init(&mut self) {
        let log_folder = ::dirs::home_dir().expect("to get home dir").join("tmp");

        let _ = ::std::fs::create_dir(log_folder.clone());

        let log_file = ::std::fs::File::create(log_folder.join("DeepGliss.log"))
            .expect("can create DeepGliss.log");

        let log_config = simplelog::ConfigBuilder::new()
            .set_time_to_local(true)
            .set_time_format_str("%M:%S:%.3f")
            .set_location_level(simplelog::LevelFilter::Info)
            .set_max_level(simplelog::LevelFilter::Info)
            .build();

        let _ = simplelog::WriteLogger::init(simplelog::LevelFilter::Info, log_config, log_file);

        log_panics::init();

        log::info!("init");
        self.state.set_parameters_to_default();
        log::info!("set default params");
    }

    fn process_events(&mut self, events: &vst::api::Events) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as f64;
        let mut chord_bender = self.state.chord_bender.lock().unwrap();
        for event in get_note_events(events) {
            chord_bender.push_event(event, now);
        }
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        log::info!("called get_editor");
        if let Some(editor) = self.editor.take() {
            Some(Box::new(editor) as Box<dyn Editor>)
        } else {
            None
        }
    }

    fn process(&mut self, _buffer: &mut AudioBuffer<f32>) {
        for (param, value) in self.state.params.iterate(true) {
            self.host.automate(param as i32, value);
        }
        self.events_buffer = vec![];

        //        let time_info = self.host.get_time_info(1).unwrap();
        //        let host_time = time_info.nanoseconds;
        let host_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as f64;

        let mut chord_bender = self.state.chord_bender.lock().unwrap();
        chord_bender.bend_duration = self.state.get_gliss_parameter(BendDuration);
        chord_bender.hold_duration = self.state.get_gliss_parameter(HoldDuration);
        chord_bender.bend_path.amplitude = self.state.get_gliss_parameter(BendPathAmplitude);
        chord_bender.bend_path.periods = self.state.get_gliss_parameter(BendPathPeriods);
        chord_bender.bend_path.s_curve_sharpness =
            self.state.get_gliss_parameter(BendPathSCurveSharpness);
        chord_bender.bend_path.path = Some(Path::from_f32(self.state.get_parameter(BendPath)));
        chord_bender.chord_mapper.chord_map =
            ChordMap::from_f32(self.state.get_parameter(BendMapping));
        chord_bender.bend_path.phase = self.state.get_gliss_parameter(BendPathPhase);

        let (events, new_rendered_benders) = chord_bender.bend(host_time);

        let mut rendered_benders = self.state.rendered_benders.lock().unwrap();
        // TODO use new method
        rendered_benders.append(new_rendered_benders);

        log::debug!(
            "sending events: {:?}",
            events.iter().map(|e| e.data).collect::<Vec<[u8; 3]>>()
        );
        self.send_buffer.send_events(&events, &mut self.host);
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        log::info!("called get_parameter_object");
        Arc::clone(&self.state) as Arc<dyn PluginParameters>
    }

    fn can_do(&self, can_do: CanDo) -> vst::api::Supported {
        log::info!("called can_do: {can_do:?}");
        use vst::api::Supported::*;
        use vst::plugin::CanDo::*;

        match can_do {
            SendEvents
            | SendMidiEvent
            | ReceiveEvents
            | ReceiveMidiEvent
            | MidiSingleNoteTuningChange
            | MidiKeyBasedInstrumentControl
            | MidiProgramNames
            | ReceiveSysExEvent => Yes,
            // TODO figure out the subset of stuff we care about
            _ => Yes,
        }
    }
}

plugin_main!(Gliss);
