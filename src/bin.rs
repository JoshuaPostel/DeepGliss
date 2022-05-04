#![feature(div_duration)]
#![feature(map_first_last)]
#![feature(derive_default_enum)]
#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

mod draw;
mod midi;
mod state;
mod ui;

use state::EditorState;

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use egui::CtxRef;
use egui_baseview::{EguiWindow, Queue};

lazy_static! {
    pub static ref GLISS_EPOCH: Duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("not before the 70s")
        - Duration::from_secs(600);
}

//pub const PITCH_BEND_RANGE: u8 = 48;
//pub const PITCH_BEND_RANGE: u8 = 24;

fn main() {
    let state = Arc::new(EditorState::new());

    // TODO is it possible to capture midi via midir and use Gliss::capture_chords lookalike to update EditorState?
    // move capture_chords to EditorState?
    //
    // first: test in daw

    let _window_handle = EguiWindow::open_blocking(
        ui::settings(),
        state,
        |_egui_ctx: &CtxRef, _queue: &mut Queue, _state: &mut Arc<EditorState>| {},
        ui::update(),
    );
}
