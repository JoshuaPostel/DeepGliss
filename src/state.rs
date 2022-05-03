use std::sync::Arc;
use std::sync::Mutex;

use vst::plugin::PluginParameters;
use vst::util::ParameterTransfer;

use crate::midi::bender::RenderedBenders;
use crate::midi::chord::ChordBender;
use crate::midi::mapper::ChordMap;
use crate::midi::paths::Path;
use crate::PITCH_BEND_RANGE;

struct Nano;

impl Nano {
    pub const SECOND: f64 = 1_000_000_000.0;
}

#[derive(PartialEq, Copy, Clone)]
pub enum GlissParam {
    BendDuration,
    HoldDuration,
    BendMapping,
    BendPath,
    BendPathAmplitude,
    BendPathPeriods,
    BendPathSCurveSharpness,
    BendPathPhase,
}

pub struct ParamConfig {
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub ui_to_gliss_scalar: f64,
    pub speed: f64,
    pub ui_display: &'static str,
}

impl ParamConfig {
    fn new(
        min: f64,
        max: f64,
        default: f64,
        ui_to_gliss_scalar: f64,
        ui_display: &'static str,
    ) -> ParamConfig {
        ParamConfig {
            min,
            max,
            default,
            ui_to_gliss_scalar,
            speed: (max - min) / 100.0,
            ui_display,
        }
    }

    pub fn map_to_daw(&self, gliss_value: f64) -> f32 {
        ((gliss_value - self.min) / (self.max - self.min)) as f32
    }

    pub fn map_to_ui(&self, daw_value: f32) -> f64 {
        self.min + daw_value as f64 * (self.max - self.min)
    }

    pub fn map_to_gliss(&self, daw_value: f32) -> f64 {
        self.map_to_ui(daw_value) * self.ui_to_gliss_scalar
    }
}

impl GlissParam {
    pub fn get_config(&self) -> ParamConfig {
        match self {
            GlissParam::BendDuration => ParamConfig::new(0.03, 8.0, 2.0, Nano::SECOND, "B"),
            GlissParam::HoldDuration => ParamConfig::new(0.10, 8.0, 1.0, Nano::SECOND, "D"),
            // TODO categorical parameters dont fit this pattern well
            GlissParam::BendMapping => ParamConfig::new(0.0, 1.0, 0.0, 1.0, "M"),
            // TODO categorical parameters dont fit this pattern well
            GlissParam::BendPath => ParamConfig::new(0.0, 1.0, 0.0, 1.0, "P"),
            // TODO figure out the proper scalar for a semitone
            GlissParam::BendPathAmplitude => {
                ParamConfig::new(0.0, 12.0, 4.0, 8192.0 / PITCH_BEND_RANGE as f64, "A")
            }
            GlissParam::BendPathPeriods => ParamConfig::new(0.0, 20.0, 2.0, 1.0, "P"),
            GlissParam::BendPathSCurveSharpness => ParamConfig::new(1.0, 5.0, 2.0, 1.0, "S"),
            GlissParam::BendPathPhase => ParamConfig::new(0.0, 1.0, 0.0, 1.0, "Z"),
        }
    }
}

fn get_parameter_index(parameter: GlissParam) -> usize {
    match parameter {
        GlissParam::BendDuration => 0,
        GlissParam::HoldDuration => 1,
        GlissParam::BendMapping => 2,
        GlissParam::BendPath => 3,
        GlissParam::BendPathAmplitude => 4,
        GlissParam::BendPathPeriods => 5,
        GlissParam::BendPathSCurveSharpness => 6,
        GlissParam::BendPathPhase => 7,
    }
}

pub struct EditorState {
    pub params: Arc<ParameterTransfer>,
    pub chord_bender: Arc<Mutex<ChordBender>>,
    pub rendered_benders: Arc<Mutex<RenderedBenders>>,
    pub keyboard_focus: Arc<Mutex<Option<Path>>>,
}

impl Default for EditorState {
    fn default() -> Self {
        let init_time = std::time::Instant::now();
        EditorState {
            // TODO i dont think we need to clone anymore
            params: Arc::new(ParameterTransfer::new(8)),
            chord_bender: Arc::new(Mutex::new(ChordBender::new(
                init_time,
                Nano::SECOND * 2.0,
                Nano::SECOND * 2.0,
                Nano::SECOND / 5.0,
            ))),
            rendered_benders: Arc::new(Mutex::new(RenderedBenders::new())),
            keyboard_focus: Arc::new(Mutex::new(None)),
        }
    }
}

impl EditorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_parameter(&self, parameter: GlissParam) -> f32 {
        let index = get_parameter_index(parameter);
        self.params.get_parameter(index)
    }

    pub fn get_ui_parameter(&self, parameter: GlissParam) -> f64 {
        let config = parameter.get_config();
        config.map_to_ui(self.get_parameter(parameter))
    }

    pub fn get_gliss_parameter(&self, parameter: GlissParam) -> f64 {
        let config = parameter.get_config();
        config.map_to_gliss(self.get_parameter(parameter))
    }

    pub fn set_parameter(&self, parameter: GlissParam, val: f64) {
        let daw_value = parameter.get_config().map_to_daw(val);
        let index = get_parameter_index(parameter);
        self.params.set_parameter(index, daw_value)
    }

    pub fn set_parameter_to_default(&self, parameter: GlissParam) {
        let config = parameter.get_config();
        let value = config.map_to_daw(config.default);
        let index = get_parameter_index(parameter);
        self.params.set_parameter(index, value)
    }

    pub fn set_parameters_to_default(&self) {
        self.set_parameter_to_default(GlissParam::BendDuration);
        self.set_parameter_to_default(GlissParam::HoldDuration);
        self.set_parameter_to_default(GlissParam::BendMapping);
        self.set_parameter_to_default(GlissParam::BendPath);
        self.set_parameter_to_default(GlissParam::BendPathAmplitude);
        self.set_parameter_to_default(GlissParam::BendPathPeriods);
        self.set_parameter_to_default(GlissParam::BendPathSCurveSharpness);
        self.set_parameter_to_default(GlissParam::BendPathPhase);
    }
}

pub struct DawParameters {
    pub bend_duration: Mutex<f32>,
}

impl Default for DawParameters {
    fn default() -> DawParameters {
        DawParameters {
            bend_duration: Mutex::new(0.5),
        }
    }
}

impl PluginParameters for EditorState {
    fn get_parameter(&self, index: i32) -> f32 {
        self.params.get_parameter(index as usize)
    }

    fn set_parameter(&self, index: i32, val: f32) {
        self.params.set_parameter(index as usize, val);
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.2} secs", self.params.get_parameter(0)),
            1 => format!("{:.2} secs", self.params.get_parameter(1)),
            2 => format!("{:?}", ChordMap::from_f32(self.params.get_parameter(2))),
            3 => format!("{:?}", Path::from_f32(self.params.get_parameter(3))),
            4 => format!("{:.2} semitones", self.params.get_parameter(4)),
            5 => format!("{:.2} periods", self.params.get_parameter(5)),
            6 => format!("{:.2}", self.params.get_parameter(6)),
            7 => format!("{:.2}", self.params.get_parameter(7)),
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Bend Duration",
            1 => "Hold Duration",
            2 => "Bend Mapping",
            3 => "Bend Path",
            4 => "Bend Path Amplitude",
            5 => "Bend Path Periods",
            6 => "Bend Path S-Curve Sharpness",
            7 => "Bend Path Phase",
            _ => "",
        }
        .to_string()
    }
}
