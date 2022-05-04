use rand::Rng;

use crate::state::GlissParam::*;
use crate::EditorState;
use crate::midi::Bend;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Path {
    #[default]
    SCurve,
    Linear,
    Sin,
    Step,
    Triangle,
    Saw,
}

impl Path {
    pub fn from_f32(val: f32) -> Self {
        match (val * 6.0) as u32 {
            0 => Path::SCurve,
            1 => Path::Linear,
            2 => Path::Sin,
            3 => Path::Step,
            4 => Path::Triangle,
            _ => Path::Saw,
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Path::SCurve => 0.0,
            Path::Linear => 1.0 / 6.0,
            Path::Sin => 2.0 / 6.0,
            Path::Step => 3.0 / 6.0,
            Path::Triangle => 4.0 / 6.0,
            Path::Saw => 5.0 / 6.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BendPathBuilder {
    // where None representes random
    pub path: Option<Path>,
    pub amplitude: f64,
    pub amplitude_randomness: f64,
    pub periods: f64,
    pub periods_randomness: f64,
    pub s_curve_sharpness: f64,
    pub s_curve_sharpness_randomness: f64,
    pub phase: f64,
    pub phase_randomness: f64,
}

impl Default for BendPathBuilder {
    fn default() -> Self {
        // made to match BendPath.default()
        Self {
            path: Some(Path::default()),
            amplitude: 0.0,
            amplitude_randomness: 0.0,
            periods: 2.0,
            periods_randomness: 0.0,
            s_curve_sharpness: 2.0,
            s_curve_sharpness_randomness: 0.0,
            phase: 0.0,
            phase_randomness: 0.0
        }
    }

}

impl BendPathBuilder {

    pub fn from_state(state: &EditorState) -> Self {
        let path = Path::from_f32(state.get_parameter(BendPath));
        let mut amplitude = 0.0;
        let mut amplitude_randomness = 0.0;
        let mut periods = 0.0;
        let mut periods_randomness = 0.0;
        let mut phase = 0.0;
        let mut phase_randomness = 0.0;
        let mut s_curve_sharpness = 0.0;
        let mut s_curve_sharpness_randomness = 0.0;

        match path {
            Path::SCurve => {
                s_curve_sharpness = state.get_gliss_parameter(SCurveSharpness);
                s_curve_sharpness_randomness = state.get_gliss_parameter(SCurveSharpnessRandomness);
            },
            Path::Step => {
                periods = state.get_gliss_parameter(StepPeriods);
                periods_randomness = state.get_gliss_parameter(StepPeriodsRandomness);
            },
            Path::Sin => {
                amplitude = state.get_gliss_parameter(SinAmplitude);
                amplitude_randomness = state.get_gliss_parameter(SinAmplitudeRandomness);
                periods = state.get_gliss_parameter(SinPeriods);
                periods_randomness = state.get_gliss_parameter(SinPeriodsRandomness);
                phase = state.get_gliss_parameter(SinPhase);
                phase_randomness = state.get_gliss_parameter(SinPhaseRandomness);
            },
            Path::Triangle => {
                amplitude = state.get_gliss_parameter(TriangleAmplitude);
                amplitude_randomness = state.get_gliss_parameter(TriangleAmplitudeRandomness);
                periods = state.get_gliss_parameter(TrianglePeriods);
                periods_randomness = state.get_gliss_parameter(TrianglePeriodsRandomness);
                phase = state.get_gliss_parameter(TrianglePhase);
                phase_randomness = state.get_gliss_parameter(TrianglePhaseRandomness);
            },
            Path::Saw => {
                amplitude = state.get_gliss_parameter(SawAmplitude);
                amplitude_randomness = state.get_gliss_parameter(SawAmplitudeRandomness);
                periods = state.get_gliss_parameter(SawPeriods);
                periods_randomness = state.get_gliss_parameter(SawPeriodsRandomness);
                phase = state.get_gliss_parameter(SawPhase);
                phase_randomness = state.get_gliss_parameter(SawPhaseRandomness);
            },
            Path::Linear => (),
        }
        Self {
            path: Some(path),
            amplitude,
            amplitude_randomness,
            periods,
            periods_randomness,
            s_curve_sharpness,
            s_curve_sharpness_randomness,
            phase,
            phase_randomness,
        }
    }

    pub fn build(&self) -> BendPath {

        let mut rng = rand::thread_rng();
        let path = match self.path {
            Some(p) => p,
            None => Path::from_f32(rng.gen()),
        };

        BendPath {
            path,
            amplitude: rng.gen_range(self.amplitude-self.amplitude_randomness..=self.amplitude+self.amplitude_randomness),
            periods: rng.gen_range(self.periods-self.periods_randomness..=self.periods+self.periods_randomness),
            s_curve_beta: rng.gen_range(self.s_curve_sharpness-self.s_curve_sharpness_randomness..=self.s_curve_sharpness+self.s_curve_sharpness_randomness),
            phase: rng.gen_range(self.phase-self.phase_randomness..=self.phase+self.phase_randomness),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BendPath {
    pub path: Path,
    pub amplitude: f64,
    pub periods: f64,
    // TODO rename to s_curve_sharpness
    pub s_curve_beta: f64,
    pub phase: f64,
}

impl Default for BendPath {
    fn default() -> BendPath {
        BendPath {
            path: Path::default(),
            amplitude: 0.0,
            periods: 2.0,
            s_curve_beta: 2.0,
            phase: 0.0,
        }
    }
}

impl BendPath {

    pub fn bend(
        &self,
        time: f64,
        start_time: f64,
        stop_time: f64,
        start_bend: f64,
        target_bend: f64,
    ) -> Bend {
        match self.path {
            Path::Linear => {
                Bend(
                    BendPath::get_linear_bend(time, start_time, stop_time, start_bend, target_bend)
                        as u16,
                )
            }
            Path::Sin => Bend(BendPath::get_sin_bend(
                time,
                start_time,
                stop_time,
                start_bend,
                target_bend,
                self.amplitude,
                self.periods,
                self.phase,
            ) as u16),
            Path::Step => Bend(BendPath::get_step_bend(
                time,
                start_time,
                stop_time,
                start_bend,
                target_bend,
                self.periods,
            ) as u16),
            Path::Triangle => Bend(BendPath::get_triangle_bend(
                time,
                start_time,
                stop_time,
                start_bend,
                target_bend,
                self.amplitude,
                self.periods,
            ) as u16),
            Path::Saw => Bend(BendPath::get_saw_bend(
                time,
                start_time,
                stop_time,
                start_bend,
                target_bend,
                self.amplitude,
                self.periods,
            ) as u16),
            Path::SCurve => Bend(BendPath::get_s_curve_bend(
                time,
                start_time,
                stop_time,
                start_bend,
                target_bend,
                self.s_curve_beta,
            ) as u16),
        }
    }

    pub fn get_linear_bend(
        time: f64,
        start_time: f64,
        stop_time: f64,
        start_bend: f64,
        target_bend: f64,
    ) -> f64 {
        let t = (time - start_time) / (stop_time - start_time);
        let adj_target = target_bend - start_bend;
        let amount = start_bend + (t * adj_target);
        log::debug!("t: {t}");
        log::debug!("target: {target_bend}");
        log::debug!("adj_target: {adj_target}");
        log::debug!("amount: {amount}");
        amount
    }

    pub fn get_s_curve_bend(
        time: f64,
        start_time: f64,
        stop_time: f64,
        start_bend: f64,
        target_bend: f64,
        s_curve_beta: f64,
    ) -> f64 {
        let range = target_bend - start_bend;
        let t = (time - start_time) / (stop_time - start_time);
        let factor = 1.0 / (1.0 + (t / (1.0 - t)).powf(-s_curve_beta));

        start_bend + (range * factor)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn get_sin_bend(
        time: f64,
        start_time: f64,
        stop_time: f64,
        start_bend: f64,
        target_bend: f64,
        amplitude: f64,
        periods: f64,
        phase: f64,
    ) -> f64 {
        // same as linear bend
        let t = (time - start_time) / (stop_time - start_time);
        let adj_target = target_bend - start_bend;
        let amount = start_bend + (t * adj_target);

        let phase_adj = amplitude * (periods * std::f64::consts::TAU * (phase)).sin();
        let sin_adj = amplitude * (periods * std::f64::consts::TAU * (t + phase)).sin() - phase_adj;
        log::debug!("sin_adj: {sin_adj}");

        amount + sin_adj
    }

    pub fn get_triangle_bend(
        time: f64,
        start_time: f64,
        stop_time: f64,
        start_bend: f64,
        target_bend: f64,
        amplitude: f64,
        periods: f64,
    ) -> f64 {
        // same as linear bend
        let t = (time - start_time) / (stop_time - start_time);
        let adj_target = target_bend - start_bend;
        let amount = start_bend + (t * adj_target);

        let t = (2.0 * t) - 1.0;
        let p = 2.0 / periods;
        log::debug!("p: {p}");
        let triangle_adj = ((4.0 * amplitude / p)
            * (((t - (p / 4.0)).rem_euclid(p)) - (p / 2.0)).abs())
            - amplitude;
        log::debug!("triangle_adj: {triangle_adj}");

        amount + triangle_adj
    }

    pub fn get_saw_bend(
        time: f64,
        start_time: f64,
        stop_time: f64,
        start_bend: f64,
        target_bend: f64,
        amplitude: f64,
        periods: f64,
    ) -> f64 {
        // same as linear bend
        let t = (time - start_time) / (stop_time - start_time);
        let adj_target = target_bend - start_bend;
        let amount = start_bend + (t * adj_target);

        let p = 1.0 / periods;
        log::debug!("p: {p}");
        let saw_adj = amplitude * 2.0 * ((t / p) - ((t / p) + 0.5).floor());
        log::debug!("saw_adj: {saw_adj}");

        amount + saw_adj
    }

    pub fn get_step_bend(
        time: f64,
        start_time: f64,
        stop_time: f64,
        start_bend: f64,
        target_bend: f64,
        periods: f64,
    ) -> f64 {
        let t = (time - start_time) / (stop_time - start_time);
        let x_step = 1.0 / periods;
        let y_step = (target_bend - start_bend) / periods;
        let n_steps = (t / x_step).floor();
        log::debug!("t: {t}");
        log::debug!("x_step: {x_step}");
        log::debug!("y_step: {y_step}");
        log::debug!("n_steps: {n_steps}");
        let bend = start_bend + (n_steps * y_step);
        log::debug!("bend: {bend}");
        bend
    }
}
