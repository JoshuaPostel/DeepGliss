use crate::draw::piano;
use crate::draw::timeline::Timeline;
use crate::draw::parameter_editor::draw_parameter_editor;
use crate::midi::mapper::ChordMap;
use crate::midi::paths::{BendPath as BendPather, Path};
use crate::midi::Note;
use crate::state::EditorState;
use crate::state::GlissParam::*;

use std::sync::Arc;
use std::time::Duration;

use baseview::{Size, WindowHandle, WindowOpenOptions, WindowScalePolicy};
use egui_baseview::{EguiWindow, Queue, RenderSettings, Settings};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use vst::editor::Editor;

use egui::{vec2, CtxRef, Pos2, Rect, Color32};
use egui::widgets::Label;

use crate::draw::button::{draw_linesegment, draw_map_button, draw_path_button};
use crate::draw::theme::GLISS_THEME;
use crate::draw::preset::{draw_save_preset, draw_load_preset};
use crate::GLISS_EPOCH;

const WINDOW_WIDTH: usize = 1024;
const WINDOW_HEIGHT: usize = 560;

pub struct GlissEditor {
    pub state: Arc<EditorState>,
    pub window_handle: Option<WindowHandle>,
    pub is_open: bool,
}

pub fn settings() -> Settings {
    Settings {
        window: WindowOpenOptions {
            title: String::from("DeepGliss"),
            size: Size::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64),
            scale: WindowScalePolicy::SystemScaleFactor,
        },
        render_settings: RenderSettings::default(),
    }
}

pub fn update() -> impl FnMut(&egui::CtxRef, &mut Queue, &mut Arc<EditorState>) {
    |egui_ctx: &CtxRef, queue: &mut Queue, state: &mut Arc<EditorState>| {
        egui::CentralPanel::default().show(egui_ctx, |ui| {
            queue.request_repaint();

            //let mut keyboard_focus = state.keyboard_focus.lock().unwrap();
            let chord_bender = state.chord_bender.lock().unwrap();
            let ui_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("non negative time")
                - *GLISS_EPOCH;

            // TODO pass notes around by reference to avoid this clone?
            let notes: Vec<Note> = chord_bender
                .chords
                .iter()
                .filter(|chord| chord.sent_to_bender)
                .flat_map(|chord| chord.notes.clone())
                .collect();
            log::debug!("n_notes: {}", notes.len());

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add(
                        egui::widgets::Label::new("DeepGliss")
                            .strong()
                            .underline()
                            .italics()
                            .heading(),
                    );
                    let response = ui.add(egui::widgets::Button::new("Settings"));
                    if response.clicked() {
                        let mut editor_params = state.editor_params.lock().unwrap();
                        *editor_params = vec![PitchBendRange, ChordCaptureDuration];
                    }
                    if response.double_clicked() {
                        state.set_parameter_to_default(PitchBendRange);
                        state.set_parameter_to_default(ChordCaptureDuration);
                    }
                    let response = ui.add(egui::widgets::Button::new("Presets"));
                    if response.clicked() {
                        let mut editor_params = state.editor_params.lock().unwrap();
                        *editor_params = vec![];
                    }
                });
                let mut x1 = 92.0;
                let mut x2 = x1 + 50.0;
                let y1 = 25.0;
                let y2 = 75.0;
                ui.vertical(|ui| {
                    ui.label("Bend Mapping");
                    // determins start of timeline
                    ui.add_space(70.0);
                    ui.horizontal(|ui| {
                        let val = ChordMap::from_f32(state.get_parameter(BendMapping));
                        // v1
                        let from_rect = egui::Rect::from_x_y_ranges(0.0..=6.0, 0.0..=6.0);

                        //closest v1
                        let to_rect = egui::Rect::from_x_y_ranges(x1..=x2, y1..=y2);
                        let line1 = [Pos2::new(1.0, 1.0), Pos2::new(5.0, 1.0)];
                        let line2 = [Pos2::new(1.0, 3.0), Pos2::new(5.0, 3.0)];
                        let line3 = [Pos2::new(1.0, 5.0), Pos2::new(5.0, 5.0)];
                        let lines = vec![line1, line2, line3];
                        draw_map_button(
                            ui,
                            "",
                            lines.clone(),
                            lines.iter().flatten().collect(),
                            from_rect,
                            to_rect,
                            state,
                            ChordMap::Closest,
                            val == ChordMap::Closest,
                        );

                        // flipped v1
                        x1 += 60.0;
                        x2 += 60.0;
                        let to_rect = egui::Rect::from_x_y_ranges(x1..=x2, y1..=y2);
                        let line1 = [Pos2::new(1.0, 1.0), Pos2::new(5.0, 5.0)];
                        let line2 = [Pos2::new(1.0, 3.0), Pos2::new(5.0, 3.0)];
                        let line3 = [Pos2::new(1.0, 5.0), Pos2::new(5.0, 1.0)];
                        let lines = vec![line1, line2, line3];
                        draw_map_button(
                            ui,
                            "",
                            lines.clone(),
                            lines.iter().flatten().collect(),
                            from_rect,
                            to_rect,
                            state,
                            ChordMap::Flipped,
                            val == ChordMap::Flipped,
                        );

                        //random v1
                        x1 += 60.0;
                        x2 += 60.0;
                        let to_rect = egui::Rect::from_x_y_ranges(x1..=x2, y1..=y2);
                        let line1 = [Pos2::new(1.0, 1.0), Pos2::new(2.0, 2.0)];
                        let line2 = [Pos2::new(1.0, 3.0), Pos2::new(2.0, 3.0)];
                        let line3 = [Pos2::new(1.0, 5.0), Pos2::new(2.0, 4.0)];
                        let line4 = [Pos2::new(4.0, 2.0), Pos2::new(5.0, 1.0)];
                        let line5 = [Pos2::new(4.0, 3.0), Pos2::new(5.0, 3.0)];
                        let line6 = [Pos2::new(4.0, 4.0), Pos2::new(5.0, 5.0)];
                        let lines = vec![line1, line2, line3, line4, line5, line6];
                        let notes = vec![
                            &line1[0], &line2[0], &line3[0], &line4[1], &line5[1], &line6[1],
                        ];
                        draw_map_button(
                            ui,
                            "？",
                            lines,
                            notes,
                            from_rect,
                            to_rect,
                            state,
                            ChordMap::Random,
                            val == ChordMap::Random,
                        );
                    });
                });

                ui.vertical(|ui| {
                    ui.label("Bend Path");
                    ui.horizontal(|ui| {
                        let val = Path::from_f32(state.get_parameter(BendPath));

                        let n_points = 50;
                        let from_rect = egui::Rect::from_x_y_ranges(0.0..=6.0, 0.0..=6.0);

                        // S-curve
                        x1 += 60.0;
                        x2 += 60.0;
                        let to_rect = egui::Rect::from_x_y_ranges(x1..=x2, y1..=y2);
                        let mut keyboard_focus = state.keyboard_focus.lock().unwrap();
                        let p1 = Pos2::new(1.0, 5.0);
                        let p2 = Pos2::new(5.0, 1.0);
                        let curve = (0..=n_points)
                            .map(|point| {
                                let time = point as f64 / n_points as f64;
                                Pos2::new(
                                    1.0 + time as f32 * 4.0,
                                    BendPather::get_s_curve_bend(
                                        1.0 + time * 4.0,
                                        1.0,
                                        5.0,
                                        5.0,
                                        1.0,
                                        state.get_gliss_parameter(SCurveSharpness),
                                    ) as f32,
                                )
                            })
                            .collect();
                        draw_path_button(
                            ui,
                            state,
                            from_rect,
                            to_rect,
                            curve,
                            vec![&p1, &p2],
                            val == Path::SCurve,
                            Path::SCurve,
                            vec![SCurveSharpness],
                            &mut keyboard_focus,
                        );

                        // linear
                        x1 += 60.0;
                        x2 += 60.0;
                        let to_rect = egui::Rect::from_x_y_ranges(x1..=x2, y1..=y2);
                        let p1 = Pos2::new(1.0, 5.0);
                        let p2 = Pos2::new(5.0, 1.0);
                        draw_path_button(
                            ui,
                            state,
                            from_rect,
                            to_rect,
                            vec![],
                            vec![&p1, &p2],
                            val == Path::Linear,
                            Path::Linear,
                            vec![],
                            &mut keyboard_focus,
                        );
                        draw_linesegment(
                            ui,
                            [p1, p2],
                            from_rect,
                            to_rect,
                            val == Path::Linear,
                            GLISS_THEME.channel_colors[0],
                        );

                        // sin
                        x1 += 60.0;
                        x2 += 60.0;
                        let to_rect = egui::Rect::from_x_y_ranges(x1..=x2, y1..=y2);
                        let p1 = Pos2::new(1.0, 5.0);
                        let p2 = Pos2::new(5.0, 1.0);
                        let curve = (0..=n_points)
                            .map(|point| {
                                let time = point as f64 / n_points as f64;
                                Pos2::new(
                                    1.0 + time as f32 * 4.0,
                                    BendPather::get_sin_bend(
                                        1.0 + time * 4.0,
                                        1.0,
                                        5.0,
                                        5.0,
                                        1.0,
                                        state.get_gliss_parameter(SinAmplitude) / 1_000.0,
                                        state.get_gliss_parameter(SinPeriods),
                                        state.get_gliss_parameter(SinPhase),
                                    ) as f32,
                                )
                            })
                            .collect();
                        draw_path_button(
                            ui,
                            state,
                            from_rect,
                            to_rect,
                            curve,
                            vec![&p1, &p2],
                            val == Path::Sin,
                            Path::Sin,
                            vec![SinAmplitude, SinPeriods, SinPhase],
                            &mut keyboard_focus,
                        );

                        // step
                        x1 += 60.0;
                        x2 += 60.0;
                        let to_rect = egui::Rect::from_x_y_ranges(x1..=x2, y1..=y2);
                        let p1 = Pos2::new(1.0, 5.0);
                        let p2 = Pos2::new(5.0, 1.0);
                        let curve = (0..=n_points)
                            .map(|point| {
                                let time = point as f64 / n_points as f64;
                                Pos2::new(
                                    1.0 + time as f32 * 4.0,
                                    BendPather::get_step_bend(
                                        1.0 + time * 4.0,
                                        1.0,
                                        5.0,
                                        5.0,
                                        1.0,
                                        state.get_gliss_parameter(StepPeriods),
                                    ) as f32,
                                )
                            })
                            .collect();
                        draw_path_button(
                            ui,
                            state,
                            from_rect,
                            to_rect,
                            curve,
                            vec![&p1, &p2],
                            val == Path::Step,
                            Path::Step,
                            vec![StepPeriods],
                            &mut keyboard_focus,
                        );

                        // triangle
                        x1 += 60.0;
                        x2 += 60.0;
                        let to_rect = egui::Rect::from_x_y_ranges(x1..=x2, y1..=y2);
                        let p1 = Pos2::new(1.0, 5.0);
                        let p2 = Pos2::new(5.0, 1.0);
                        let curve = (0..=n_points)
                            .map(|point| {
                                let time = point as f64 / n_points as f64;
                                Pos2::new(
                                    1.0 + time as f32 * 4.0,
                                    BendPather::get_triangle_bend(
                                        1.0 + time * 4.0,
                                        1.0,
                                        5.0,
                                        5.0,
                                        1.0,
                                        state.get_gliss_parameter(TriangleAmplitude) / 1_000.0,
                                        state.get_gliss_parameter(TrianglePeriods),
                                        state.get_gliss_parameter(TrianglePhase),
                                        // TODO
                                        //state.get_gliss_parameter(SinPhase),
                                    ) as f32,
                                )
                            })
                            .collect();
                        draw_path_button(
                            ui,
                            state,
                            from_rect,
                            to_rect,
                            curve,
                            vec![&p1, &p2],
                            val == Path::Triangle,
                            Path::Triangle,
                            vec![TriangleAmplitude, TrianglePeriods, TrianglePhase],
                            &mut keyboard_focus,
                        );

                        // saw
                        x1 += 60.0;
                        x2 += 60.0;
                        let to_rect = egui::Rect::from_x_y_ranges(x1..=x2, y1..=y2);
                        let p1 = Pos2::new(1.0, 5.0);
                        let p2 = Pos2::new(5.0, 1.0);
                        let curve = (0..=n_points)
                            .map(|point| {
                                let time = point as f64 / n_points as f64;
                                Pos2::new(
                                    1.0 + time as f32 * 4.0,
                                    BendPather::get_saw_bend(
                                        1.0 + time * 4.0,
                                        1.0,
                                        5.0,
                                        5.0,
                                        1.0,
                                        state.get_gliss_parameter(SawAmplitude) / 1_000.0,
                                        state.get_gliss_parameter(SawPeriods),
                                        state.get_gliss_parameter(SawPhase),
                                        // TODO
                                        //state.get_gliss_parameter(SinPhase),
                                    ) as f32,
                                )
                            })
                            .collect();
                        draw_path_button(
                            ui,
                            state,
                            from_rect,
                            to_rect,
                            curve,
                            vec![&p1, &p2],
                            val == Path::Saw,
                            Path::Saw,
                            vec![SawAmplitude, SawPeriods, SawPhase],
                            &mut keyboard_focus,
                        );
                    });
                });

                // TODO add dark and light mode?
                //egui::widgets::global_dark_light_mode_switch(ui);
                ui.vertical(|ui| {
                    ui.label("Parameters");
                    let parameter_editor_rect = Rect::from_x_y_ranges(630.0..=900.0, 15.0..=150.0);

                    let editor_params = state.editor_params.lock().unwrap().to_vec();
                    if editor_params.is_empty() {
                        ui.horizontal(|ui|{
                            if let Err(e) = draw_save_preset(ui, state) {
                                let mut error_state = state.error_state.lock().unwrap();
                                *error_state = Some(e.to_string());
                            };

                        });
                        if let Err(e) = draw_load_preset(ui, state) {
                            let mut error_state = state.error_state.lock().unwrap();
                            *error_state = Some(e.to_string());
                        };
                    } else {
                        draw_parameter_editor(ui, state, editor_params, parameter_editor_rect);
                    }
                });
            });

            // ui layout
            let desired_size = ui.available_width() * vec2(1.0, 0.4);
            let (_id, rect) = ui.allocate_space(desired_size);
            let to_screen =
                emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0), rect);

            let tl = to_screen * Pos2::new(0.0, 0.0);
            let br = to_screen * Pos2::new(0.9, 1.0);
            let timeline_rect = Rect::from_two_pos(tl, br);

            let tl = to_screen * Pos2::new(0.9, 0.0);
            let br = to_screen * Pos2::new(1.0, 1.0);
            let piano_rect = Rect::from_two_pos(tl, br);

            // draw elements
            let mut shapes = vec![];
            // TODO make this progromatically = BendDuration.max config
            let history_duration = Duration::from_secs(8);
            let bend_duration = Duration::from_nanos(chord_bender.bend_duration as u64);
            let timeline = Timeline::new(timeline_rect, 3..=4, history_duration, bend_duration);
            let mut timeline_shapes = timeline.draw(ui_now, notes);
            shapes.append(&mut timeline_shapes);
            timeline.draw_control_pin(state, ui);
            timeline.draw_hold_pin(state, ui);

            let midi_notes = timeline.midi_notes;
            let min_midi = midi_notes.clone().min().unwrap() as f32;
            let max_midi = midi_notes.max().unwrap() as f32;
            //let midi_range = min_midi - 0.5..=max_midi + 0.5;
            let start_time = (ui_now - timeline.history_duration).as_secs_f32();
            let end_time = (ui_now + timeline.bend_duration).as_secs_f32();
            //let time_range = start_time..=end_time;
            // TODO reimplement logic in correct orientation?
            // flipping upsidedown
            let midi_number_x_time = Rect::from_min_max(Pos2::new(start_time, max_midi + 0.5), Pos2::new(end_time, min_midi - 0.5));
            let midi_number_x_time_to_screen =
                emath::RectTransform::from_to(midi_number_x_time, timeline_rect);
            let mut rendered_benders = state.rendered_benders.lock().unwrap();

            // drop old rendered benders
            // TODO LOW PRIORITY
            // TODO do this check less often?
            // TODO this is not working as expected
            //rendered_benders.retain(start_time);

            rendered_benders.render(ui, midi_number_x_time_to_screen);

            let active_notes: Vec<u8> = match chord_bender.chords.last() {
                Some(chord) => chord
                    .notes
                    .iter()
                    .filter(|note| !note.key_released)
                    .map(|note| note.midi_number)
                    .collect(),
                None => vec![],
            };
            let mut piano = piano::draw(piano_rect, 3..=4, active_notes);
            shapes.append(&mut piano);

            ui.painter().extend(shapes);

            // inform user of errors
            if let Some(error_state) = &*state.error_state.lock().unwrap() {
                let error_msg = Label::new(format!("error: {error_state}")).background_color(Color32::DARK_RED);
                ui.add(error_msg);
            }
        });
    }
}

impl Editor for GlissEditor {
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn size(&self) -> (i32, i32) {
        (WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
    }

    fn open(&mut self, parent: *mut ::std::ffi::c_void) -> bool {
        log::info!("Editor open");
        if self.is_open {
            return false;
        }

        self.is_open = true;

        let window_handle = EguiWindow::open_parented(
            &VstParent(parent),
            settings(),
            self.state.clone(),
            |_egui_ctx: &CtxRef, _queue: &mut Queue, _state: &mut Arc<EditorState>| {},
            update(),
        );

        self.window_handle = Some(window_handle);

        true
    }

    fn is_open(&mut self) -> bool {
        self.is_open
    }

    fn close(&mut self) {
        self.is_open = false;
        if let Some(mut window_handle) = self.window_handle.take() {
            window_handle.close();
        }
    }
}

pub struct VstParent(pub *mut ::std::ffi::c_void);

#[cfg(target_os = "macos")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::macos::MacOSHandle;

        RawWindowHandle::MacOS(MacOSHandle {
            ns_view: self.0 as *mut ::std::ffi::c_void,
            ..MacOSHandle::empty()
        })
    }
}

#[cfg(target_os = "windows")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::windows::WindowsHandle;

        RawWindowHandle::Windows(WindowsHandle {
            hwnd: self.0,
            ..WindowsHandle::empty()
        })
    }
}

#[cfg(target_os = "linux")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::unix::XcbHandle;

        RawWindowHandle::Xcb(XcbHandle {
            window: self.0 as u32,
            ..XcbHandle::empty()
        })
    }
}
