use std::io::BufRead;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use vst::plugin::PluginParameters;
use vst::util::ParameterTransfer;

use anyhow::Result;

use crate::midi::bender::RenderedBenders;
use crate::midi::chord::ChordBender;
use crate::midi::mapper::ChordMap;
use crate::midi::paths::Path;

struct Nano;

impl Nano {
    pub const SECOND: f64 = 1_000_000_000.0;
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum GlissParam {
    PitchBendRange,
    BendDuration,
    HoldDuration,
    ChordCaptureDuration,
    BendMapping,
    BendPath,
    SCurveSharpness,
    SCurveSharpnessRandomness,
    StepPeriods,
    StepPeriodsRandomness,
    SinAmplitude,
    SinAmplitudeRandomness,
    SinPeriods,
    SinPeriodsRandomness,
    SinPhase,
    SinPhaseRandomness,
    TriangleAmplitude,
    TriangleAmplitudeRandomness,
    TrianglePeriods,
    TrianglePeriodsRandomness,
    TrianglePhase,
    TrianglePhaseRandomness,
    SawAmplitude,
    SawAmplitudeRandomness,
    SawPeriods,
    SawPeriodsRandomness,
    SawPhase,
    SawPhaseRandomness,
}

pub const GLISS_PARAMETERS: [GlissParam; 28] = [
    GlissParam::PitchBendRange,
    GlissParam::BendDuration,
    GlissParam::HoldDuration,
    GlissParam::ChordCaptureDuration,
    GlissParam::BendMapping,
    GlissParam::BendPath,
    GlissParam::SCurveSharpness,
    GlissParam::SCurveSharpnessRandomness,
    GlissParam::StepPeriods,
    GlissParam::StepPeriodsRandomness,
    GlissParam::SinAmplitude,
    GlissParam::SinAmplitudeRandomness,
    GlissParam::SinPeriods,
    GlissParam::SinPeriodsRandomness,
    GlissParam::SinPhase,
    GlissParam::SinPhaseRandomness,
    GlissParam::TriangleAmplitude,
    GlissParam::TriangleAmplitudeRandomness,
    GlissParam::TrianglePeriods,
    GlissParam::TrianglePeriodsRandomness,
    GlissParam::TrianglePhase,
    GlissParam::TrianglePhaseRandomness,
    GlissParam::SawAmplitude,
    GlissParam::SawAmplitudeRandomness,
    GlissParam::SawPeriods,
    GlissParam::SawPeriodsRandomness,
    GlissParam::SawPhase,
    GlissParam::SawPhaseRandomness,
];

pub struct ParamConfig {
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub is_integer: bool,
    pub is_semitone: bool,
    pub ui_to_gliss_scalar: f64,
    pub speed: f64,
    pub unit: &'static str,
    pub ui_name: &'static str,
    pub daw_name: &'static str,
    pub daw_display: &'static dyn Fn(f32) -> String,
}

impl ParamConfig {
    pub fn map_to_daw(&self, gliss_value: f64) -> f32 {
        ((gliss_value - self.min) / (self.max - self.min)) as f32
    }

    pub fn map_to_ui(&self, daw_value: f32) -> f64 {
        let value = self.min + daw_value as f64 * (self.max - self.min);
        if self.is_integer {
            value.round()
        } else {
            value
        }
    }

    pub fn map_to_gliss(&self, daw_value: f32) -> f64 {
        let value = self.map_to_ui(daw_value) * self.ui_to_gliss_scalar;
        if self.is_integer {
            value.round()
        } else {
            value
        }
    }
}

impl GlissParam {
    pub fn get_randomness_param(&self) -> Option<GlissParam> {
        match self {
            GlissParam::SCurveSharpness => Some(GlissParam::SCurveSharpnessRandomness),
            GlissParam::StepPeriods => Some(GlissParam::StepPeriodsRandomness),
            GlissParam::SinAmplitude => Some(GlissParam::SinAmplitudeRandomness),
            GlissParam::SinPeriods => Some(GlissParam::SinPeriodsRandomness),
            GlissParam::SinPhase => Some(GlissParam::SinPhaseRandomness),
            GlissParam::TriangleAmplitude => Some(GlissParam::TriangleAmplitudeRandomness),
            GlissParam::TrianglePeriods => Some(GlissParam::TrianglePeriodsRandomness),
            GlissParam::TrianglePhase => Some(GlissParam::TrianglePhaseRandomness),
            GlissParam::SawAmplitude => Some(GlissParam::SawAmplitudeRandomness),
            GlissParam::SawPeriods => Some(GlissParam::SawPeriodsRandomness),
            GlissParam::SawPhase => Some(GlissParam::SawPhaseRandomness),
            _ => None,
        }
    }

    pub fn get_config(&self) -> ParamConfig {
        match self {
            GlissParam::PitchBendRange => {
                let min = 2.0;
                let max = 48.0;
                ParamConfig {
                    min,
                    max,
                    default: 24.0,
                    is_integer: true,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "semitones",
                    ui_name: "Pitch Bend Range",
                    daw_name: "Pitch Bend Range",
                    daw_display: &|value| format!("{:.2} semitones", value),
                }
            }
            GlissParam::BendDuration => {
                let min = 0.03;
                let max = 8.0;
                ParamConfig {
                    min,
                    max,
                    default: 2.0,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: Nano::SECOND,
                    speed: (max - min) / 100.0,
                    unit: "seconds",
                    ui_name: "Bend Duration",
                    daw_name: "Bend Duration",
                    daw_display: &|value| format!("{:.2} secs", value),
                }
            }
            GlissParam::HoldDuration => {
                let min = 0.10;
                let max = 8.0;
                ParamConfig {
                    min,
                    max,
                    default: 2.0,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: Nano::SECOND,
                    speed: (max - min) / 100.0,
                    unit: "seconds",
                    ui_name: "Hold Duration",
                    daw_name: "Hold Duration",
                    daw_display: &|value| format!("{:.2} secs", value),
                }
            }
            GlissParam::ChordCaptureDuration => {
                let min = 0.00001;
                let max = 1.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.2,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: Nano::SECOND,
                    speed: (max - min) / 100.0,
                    unit: "seconds",
                    ui_name: "Chord Capture Time",
                    daw_name: "Chord Capture Duration",
                    daw_display: &|value| format!("{:.2} secs", value),
                }
            }
            GlissParam::BendMapping => {
                let min = 0.0;
                let max = 1.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "N/A",
                    ui_name: "Mapping",
                    daw_name: "Mapping",
                    daw_display: &|value| format!("{:?}", ChordMap::from_f32(value)),
                }
            }
            GlissParam::BendPath => {
                let min = 0.0;
                let max = 1.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "N/A",
                    ui_name: "Path",
                    daw_name: "Path",
                    daw_display: &|value| format!("{:?}", Path::from_f32(value)),
                }
            }
            GlissParam::SCurveSharpness => {
                let min = 1.0;
                let max = 5.0;
                ParamConfig {
                    min,
                    max,
                    default: 2.0,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "",
                    ui_name: "Sharpness",
                    daw_name: "S-Curve Sharpness",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::SCurveSharpnessRandomness => {
                let min = 0.0;
                let max = 5.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "",
                    ui_name: "Sharpness Randomness",
                    daw_name: "Sin Sharpness Randomness",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::SinAmplitude => {
                let min = 0.0;
                let max = 12.0;
                ParamConfig {
                    min,
                    max,
                    default: 1.0,
                    is_integer: false,
                    is_semitone: true,
                    ui_to_gliss_scalar: 8192.0, // / PITCH_BEND_RANGE as f64,
                    speed: (max - min) / 100.0,
                    unit: "semitones",
                    ui_name: "Amplitude",
                    daw_name: "Sin Amplitude",
                    daw_display: &|value| format!("{:.2} semitones", value),
                }
            }
            GlissParam::SinAmplitudeRandomness => {
                let min = 0.0;
                let max = 6.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: false,
                    is_semitone: true,
                    ui_to_gliss_scalar: 8192.0, // / PITCH_BEND_RANGE as f64,
                    speed: (max - min) / 100.0,
                    unit: "semitones",
                    ui_name: "Amplitude Randomness",
                    daw_name: "Sin Amplitude Randomness",
                    daw_display: &|value| format!("{:.2} semitones", value),
                }
            }
            GlissParam::SinPeriods => {
                let min = 0.0;
                let max = 20.0;
                ParamConfig {
                    min,
                    max,
                    default: 4.0,
                    is_integer: true,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "",
                    ui_name: "Periods",
                    daw_name: "Sin Periods",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::SinPeriodsRandomness => {
                let min = 0.0;
                let max = 10.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: true,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "",
                    ui_name: "Periods Randomness",
                    daw_name: "Sin Periods Randomness",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::SinPhase => {
                let min = 0.0;
                let max = 1.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "periods",
                    ui_name: "Phase",
                    daw_name: "Sin Phase",
                    daw_display: &|value| format!("{:.2} periods", value),
                }
            }
            GlissParam::SinPhaseRandomness => {
                let min = 0.0;
                let max = 1.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "periods",
                    ui_name: "Phase Randomness",
                    daw_name: "Sin Phase Randomness",
                    daw_display: &|value| format!("{:.2} periods", value),
                }
            }
            GlissParam::TriangleAmplitude => {
                let min = 0.0;
                let max = 12.0;
                ParamConfig {
                    min,
                    max,
                    default: 1.0,
                    is_integer: false,
                    is_semitone: true,
                    ui_to_gliss_scalar: 8192.0, // / PITCH_BEND_RANGE as f64,
                    speed: (max - min) / 100.0,
                    unit: "semitones",
                    ui_name: "Amplitude",
                    daw_name: "Triangle Amplitude",
                    daw_display: &|value| format!("{:.2} semitones", value),
                }
            }
            GlissParam::TriangleAmplitudeRandomness => {
                let min = 0.0;
                let max = 6.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: false,
                    is_semitone: true,
                    ui_to_gliss_scalar: 8192.0, // / PITCH_BEND_RANGE as f64,
                    speed: (max - min) / 100.0,
                    unit: "semitones",
                    ui_name: "Amplitude Randomness",
                    daw_name: "Triangle Amplitude Randomness",
                    daw_display: &|value| format!("{:.2} semitones", value),
                }
            }
            GlissParam::TrianglePeriods => {
                let min = 0.0;
                let max = 20.0;
                ParamConfig {
                    min,
                    max,
                    default: 4.0,
                    is_integer: true,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "",
                    ui_name: "Periods",
                    daw_name: "Triangle Periods",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::TrianglePeriodsRandomness => {
                let min = 0.0;
                let max = 10.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: true,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "",
                    ui_name: "Periods Randomness",
                    daw_name: "Triangle Periods Randomness",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::TrianglePhase => {
                let min = 0.0;
                let max = 1.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "periods",
                    ui_name: "Phase",
                    daw_name: "Triangle Phase",
                    daw_display: &|value| format!("{:.2} periods", value),
                }
            }
            GlissParam::TrianglePhaseRandomness => {
                let min = 0.0;
                let max = 1.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "periods",
                    ui_name: "Phase Randomness",
                    daw_name: "Triangle Phase Randomness",
                    daw_display: &|value| format!("{:.2} periods", value),
                }
            }
            GlissParam::SawAmplitude => {
                let min = 0.0;
                let max = 12.0;
                ParamConfig {
                    min,
                    max,
                    default: 1.0,
                    is_integer: false,
                    is_semitone: true,
                    ui_to_gliss_scalar: 8192.0, // / PITCH_BEND_RANGE as f64,
                    speed: (max - min) / 100.0,
                    unit: "semitones",
                    ui_name: "Amplitude",
                    daw_name: "Saw Amplitude",
                    daw_display: &|value| format!("{:.2} semitones", value),
                }
            }
            GlissParam::SawAmplitudeRandomness => {
                let min = 0.0;
                let max = 6.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: false,
                    is_semitone: true,
                    ui_to_gliss_scalar: 8192.0, // / PITCH_BEND_RANGE as f64,
                    speed: (max - min) / 100.0,
                    unit: "semitones",
                    ui_name: "Amplitude Randomness",
                    daw_name: "Saw Amplitude Randomness",
                    daw_display: &|value| format!("{:.2} semitones", value),
                }
            }
            GlissParam::SawPeriods => {
                let min = 0.0;
                let max = 20.0;
                ParamConfig {
                    min,
                    max,
                    default: 4.0,
                    is_integer: true,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "",
                    ui_name: "Periods",
                    daw_name: "Saw Periods",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::SawPeriodsRandomness => {
                let min = 0.0;
                let max = 10.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: true,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "",
                    ui_name: "Periods Randomness",
                    daw_name: "Saw Periods Randomness",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::SawPhase => {
                let min = 0.0;
                let max = 1.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "periods",
                    ui_name: "Phase",
                    daw_name: "Saw Phase",
                    daw_display: &|value| format!("{:.2} periods", value),
                }
            }
            GlissParam::SawPhaseRandomness => {
                let min = 0.0;
                let max = 1.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: false,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "periods",
                    ui_name: "Phase Randomness",
                    daw_name: "Saw Phase Randomness",
                    daw_display: &|value| format!("{:.2} periods", value),
                }
            }
            GlissParam::StepPeriods => {
                let min = 0.0;
                let max = 20.0;
                ParamConfig {
                    min,
                    max,
                    default: 4.0,
                    is_integer: true,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "",
                    ui_name: "Periods",
                    daw_name: "Step Periods",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::StepPeriodsRandomness => {
                let min = 0.0;
                let max = 10.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    is_integer: true,
                    is_semitone: false,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "",
                    ui_name: "Periods Randomness",
                    daw_name: "Step Periods Randomness",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
        }
    }
}

fn get_parameter_index(parameter: GlissParam) -> usize {
    GLISS_PARAMETERS
        .iter()
        .position(|&p| p == parameter)
        .expect("parameter in parameters")
    // TODO this is more efficent?
    //    match parameter {
    //        GlissParam::BendDuration => 0,
    //        GlissParam::HoldDuration => 1,
    //        ...
    //    }
}

pub struct ErrorState {
    pub message: String,
    pub time: std::time::SystemTime,
}

impl ErrorState {
    pub fn new(message: String) -> Self {
        Self {
            message,
            time: std::time::SystemTime::now(),
        }
    }
}

pub struct EditorState {
    pub params: Arc<ParameterTransfer>,
    pub editor_params: Arc<Mutex<Vec<GlissParam>>>,
    pub chord_bender: Arc<Mutex<ChordBender>>,
    pub rendered_benders: Arc<Mutex<RenderedBenders>>,
    pub keyboard_focus: Arc<Mutex<Option<Path>>>,
    pub error_state: Arc<Mutex<Option<ErrorState>>>,
    pub preset_filename: Arc<Mutex<String>>,
}

impl Default for EditorState {
    fn default() -> Self {
        let init_time = std::time::Instant::now();
        EditorState {
            // TODO i dont think we need to clone anymore
            params: Arc::new(ParameterTransfer::new(GLISS_PARAMETERS.len())),
            editor_params: Arc::new(Mutex::new(vec![GlissParam::SCurveSharpness])),
            chord_bender: Arc::new(Mutex::new(ChordBender::new(
                init_time,
                Nano::SECOND * GlissParam::BendDuration.get_config().default,
                Nano::SECOND * GlissParam::HoldDuration.get_config().default,
                GlissParam::PitchBendRange.get_config().default as f32,
                Nano::SECOND * GlissParam::ChordCaptureDuration.get_config().default,
            ))),
            rendered_benders: Arc::new(Mutex::new(RenderedBenders::new())),
            keyboard_focus: Arc::new(Mutex::new(None)),
            error_state: Arc::new(Mutex::new(None)),
            preset_filename: Arc::new(Mutex::new("my_filename".to_string())),
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
        if config.is_semitone {
            config.map_to_gliss(self.get_parameter(parameter))
                / self.get_gliss_parameter(GlissParam::PitchBendRange)
        } else {
            config.map_to_gliss(self.get_parameter(parameter))
        }
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
        for param in GLISS_PARAMETERS {
            self.set_parameter_to_default(param);
        }
    }

    pub fn save_parameters(&self, mut file: std::fs::File) -> Result<()> {
        for param in GLISS_PARAMETERS {
            let value = self.get_parameter(param);
            writeln!(file, "{value}")?;
            log::info!("writing param: {param:?} to: {value}");
        }
        Ok(())
    }

    pub fn load_parameters(&self, file: std::fs::File) -> Result<()> {
        let mut reader = std::io::BufReader::new(file);
        for param in GLISS_PARAMETERS {
            let mut line = String::new();
            reader.read_line(&mut line)?;
            line.pop();
            log::info!("read line after newline removed: {line}");
            let value = line.parse::<f32>()?;
            log::info!("setting param: {param:?} to parsed value: {value}");
            let index = get_parameter_index(param);
            self.params.set_parameter(index, value)
        }
        Ok(())
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
        let value = self.params.get_parameter(index as usize);
        (GLISS_PARAMETERS[index as usize].get_config().daw_display)(value)
    }

    fn get_parameter_name(&self, index: i32) -> String {
        GLISS_PARAMETERS[index as usize]
            .get_config()
            .daw_name
            .to_string()
    }
}
