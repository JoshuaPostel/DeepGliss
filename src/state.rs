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

const GLISS_PARAMETERS: [GlissParam; 26] = [
    GlissParam::BendDuration,
    GlissParam::HoldDuration,
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
        self.min + daw_value as f64 * (self.max - self.min)
    }

    pub fn map_to_gliss(&self, daw_value: f32) -> f64 {
        self.map_to_ui(daw_value) * self.ui_to_gliss_scalar
    }
}

impl GlissParam {
    pub fn get_randomness_param(&self) -> Option<GlissParam> {
        match self {
            GlissParam::SCurveSharpness => {
                Some(GlissParam::SCurveSharpnessRandomness)
            },
            GlissParam::StepPeriods => {
                Some(GlissParam::StepPeriodsRandomness)
            },
            GlissParam::SinAmplitude => {
                Some(GlissParam::SinAmplitudeRandomness)
            },
            GlissParam::SinPeriods => {
                Some(GlissParam::SinPeriodsRandomness)
            },
            GlissParam::SinPhase => {
                Some(GlissParam::SinPhaseRandomness)
            },
            GlissParam::TriangleAmplitude => {
                Some(GlissParam::TriangleAmplitudeRandomness)
            },
            GlissParam::TrianglePeriods => {
                Some(GlissParam::TrianglePeriodsRandomness)
            }
            GlissParam::TrianglePhase => {
                Some(GlissParam::TrianglePhaseRandomness)
            },
            GlissParam::SawAmplitude => {
                Some(GlissParam::SawAmplitudeRandomness)
            }
            GlissParam::SawPeriods => {
                Some(GlissParam::SawPeriodsRandomness)
            },
            GlissParam::SawPhase => {
                Some(GlissParam::SawPhaseRandomness)
            },
            _ => None,

        }
    }

    pub fn get_config(&self) -> ParamConfig {
        match self {
            GlissParam::BendDuration => {
                let min = 0.03;
                let max = 8.0;
                ParamConfig {
                    min,
                    max,
                    default: 2.0,
                    ui_to_gliss_scalar: Nano::SECOND,
                    speed: (max - min) / 100.0,
                    unit: "secs", 
                    ui_name: "Bend Time",
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
                    default: 1.0,
                    ui_to_gliss_scalar: Nano::SECOND,
                    speed: (max - min) / 100.0,
                    unit: "secs", 
                    ui_name: "Hold Time",
                    daw_name: "Hold Duration",
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
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "", 
                    ui_name: "Phase Randomness",
                    daw_name: "Sin Phase Randomness",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::SinAmplitude => {
                let min = 0.0;
                let max = 12.0;
                ParamConfig {
                    min,
                    max,
                    default: 4.0,
                    ui_to_gliss_scalar: 8192.0 / PITCH_BEND_RANGE as f64,
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
                    ui_to_gliss_scalar: 8192.0 / PITCH_BEND_RANGE as f64,
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
                    default: 2.0,
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
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "", 
                    ui_name: "Phase",
                    daw_name: "Sin Phase",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::SinPhaseRandomness => {
                let min = 0.0;
                let max = 5.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "", 
                    ui_name: "Phase Randomness",
                    daw_name: "Sin Phase Randomness",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::TriangleAmplitude => {
                let min = 0.0;
                let max = 12.0;
                ParamConfig {
                    min,
                    max,
                    default: 4.0,
                    ui_to_gliss_scalar: 8192.0 / PITCH_BEND_RANGE as f64,
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
                    ui_to_gliss_scalar: 8192.0 / PITCH_BEND_RANGE as f64,
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
                    default: 2.0,
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
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "", 
                    ui_name: "Phase",
                    daw_name: "Triangle Phase",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::TrianglePhaseRandomness => {
                let min = 0.0;
                let max = 5.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "", 
                    ui_name: "Phase Randomness",
                    daw_name: "Triangle Phase Randomness",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::SawAmplitude => {
                let min = 0.0;
                let max = 12.0;
                ParamConfig {
                    min,
                    max,
                    default: 4.0,
                    ui_to_gliss_scalar: 8192.0 / PITCH_BEND_RANGE as f64,
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
                    ui_to_gliss_scalar: 8192.0 / PITCH_BEND_RANGE as f64,
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
                    default: 2.0,
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
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "", 
                    ui_name: "Phase",
                    daw_name: "Saw Phase",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::SawPhaseRandomness => {
                let min = 0.0;
                let max = 5.0;
                ParamConfig {
                    min,
                    max,
                    default: 0.0,
                    ui_to_gliss_scalar: 1.0,
                    speed: (max - min) / 100.0,
                    unit: "", 
                    ui_name: "Phase Randomness",
                    daw_name: "Saw Phase Randomness",
                    daw_display: &|value| format!("{:.2}", value),
                }
            }
            GlissParam::StepPeriods => {
                let min = 0.0;
                let max = 20.0;
                ParamConfig {
                    min,
                    max,
                    default: 2.0,
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

pub struct EditorState {
    pub params: Arc<ParameterTransfer>,
    pub editor_params: Arc<Mutex<Vec<GlissParam>>>,
    pub chord_bender: Arc<Mutex<ChordBender>>,
    pub rendered_benders: Arc<Mutex<RenderedBenders>>,
    pub keyboard_focus: Arc<Mutex<Option<Path>>>,
}

impl Default for EditorState {
    fn default() -> Self {
        let init_time = std::time::Instant::now();
        EditorState {
            // TODO i dont think we need to clone anymore
            params: Arc::new(ParameterTransfer::new(26)),
            editor_params: Arc::new(Mutex::new(vec![GlissParam::SCurveSharpness])),
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
        for param in GLISS_PARAMETERS {
            self.set_parameter_to_default(param);
        }
        //        self.set_parameter_to_default(GlissParam::BendDuration);
        //        self.set_parameter_to_default(GlissParam::HoldDuration);
        //        self.set_parameter_to_default(GlissParam::BendMapping);
        //        self.set_parameter_to_default(GlissParam::BendPath);
        //        self.set_parameter_to_default(GlissParam::BendPathAmplitude);
        //        self.set_parameter_to_default(GlissParam::BendPathPeriods);
        //        self.set_parameter_to_default(GlissParam::BendPathSCurveSharpness);
        //        self.set_parameter_to_default(GlissParam::BendPathPhase);
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
        let value = self.params.get_parameter(index as usize);
        (GLISS_PARAMETERS[index as usize].get_config().daw_display)(value)
//        match index {
//            0 => format!("{:.2} secs", self.params.get_parameter(0)),
//            1 => format!("{:.2} secs", self.params.get_parameter(1)),
//            2 => format!("{:?}", ChordMap::from_f32(self.params.get_parameter(2))),
//            3 => format!("{:?}", Path::from_f32(self.params.get_parameter(3))),
//            4 => format!("{:.2} semitones", self.params.get_parameter(4)),
//            5 => format!("{:.2} periods", self.params.get_parameter(5)),
//            6 => format!("{:.2}", self.params.get_parameter(6)),
//            7 => format!("{:.2}", self.params.get_parameter(7)),
//            _ => "".to_string(),
//        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        GLISS_PARAMETERS[index as usize].get_config().daw_name.to_string()
        //        match index {
        //            0 => "Bend Duration",
        //            1 => "Hold Duration",
        //            2 => "Bend Mapping",
        //            3 => "Bend Path",
        //            4 => "Bend Path Amplitude",
        //            5 => "Bend Path Periods",
        //            6 => "Bend Path S-Curve Sharpness",
        //            7 => "Bend Path Phase",
        //            _ => "",
        //        }
        //        .to_string()
    }
}
