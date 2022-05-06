use std::ops::RangeInclusive;

use egui::Pos2;
use egui::{Color32, Rect, Shape, Stroke};

use crate::draw::theme::GLISS_THEME;

const WINDOW_WIDTH: usize = 1024;
const WINDOW_HEIGHT: usize = 512;

pub fn draw(rect: Rect, octive_range: RangeInclusive<u8>, active_notes: Vec<u8>) -> Vec<Shape> {
    let mut shapes = vec![];

    let octave_height = (rect.max.y - rect.min.y) / octive_range.len() as f32;
    // flipped y because egui origin is top left
    for (i, octave) in octive_range.rev().enumerate() {
        let mut active_octave_notes = vec![];
        let octave_midi_notes = ((octave + 1) * 12)..((octave + 2) * 12);
        for note in &active_notes {
            if octave_midi_notes.contains(note) {
                active_octave_notes.push(note - octave_midi_notes.start);
            }
        }

        let top_left = Pos2::new(rect.min.x, rect.min.y + i as f32 * octave_height);
        let bot_right = Pos2::new(rect.max.x, rect.min.y + (1.0 + i as f32) * octave_height);
        let octave_rect = Rect::from_two_pos(top_left, bot_right);
        let mut octave_shapes = draw_octave(octave_rect, active_octave_notes);
        shapes.append(&mut octave_shapes);
    }
    shapes
}

pub fn draw_octave(rect: Rect, active_notes: Vec<u8>) -> Vec<Shape> {
    let white = GLISS_THEME.piano.white;
    let black = GLISS_THEME.piano.black;
    let white_active = GLISS_THEME.piano.white_active;
    let black_active = GLISS_THEME.piano.black_active;

    let mut colors = [
        white, black, white, black, white, white, black, white, black, white, black, white,
    ];

    for active_note in active_notes {
        match active_note {
            1 | 3 | 6 | 8 | 10 => colors[active_note as usize] = black_active,
            _ => colors[active_note as usize] = white_active,
        }
    }

    let mut keys = vec![];

    let key_height = (rect.max.y - rect.min.y) / 12.0;

    for (i, color) in colors.iter().enumerate() {
        // flipped y because egui origin is top left
        let top_left = Pos2::new(rect.min.x, rect.max.y - i as f32 * key_height);
        let bot_right = Pos2::new(rect.max.x, rect.max.y - (1.0 + i as f32) * key_height + 2.0);

        let key_rect = Rect::from_two_pos(top_left, bot_right);

        let key = draw_key(key_rect, *color);
        keys.push(key);

        //let mut key = draw_thatched_key(key_rect, *color);
        //keys.append(&mut key);
    }
    keys
}

fn draw_key(rect: Rect, color: Color32) -> Shape {
    Shape::rect_filled(rect, 2.5, color)
}

fn draw_thatched_key(bounds: Rect, color: Color32) -> Vec<Shape> {
    let to_screen =
        emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0), bounds);
    let mut shapes = vec![];
    let stroke = Stroke::new(0.1, color);

    // TODO what is the dynamic adjustment to diagonals given the boarder rounding?
    let eps = 0.02;
    let p1 = to_screen * Pos2::new(0.0 + eps, 0.0 + eps);
    let p2 = to_screen * Pos2::new(1.0 - eps, 1.0 - eps);
    let diag = Shape::line_segment([p1, p2], stroke);
    shapes.push(diag);
    let p1 = to_screen * Pos2::new(1.0 - eps, 0.0 + eps);
    let p2 = to_screen * Pos2::new(0.0 + eps, 1.0 - eps);
    let diag = Shape::line_segment([p1, p2], stroke);
    shapes.push(diag);
    let border = Shape::rect_stroke(bounds, 5.0, Stroke::new(0.25, color));
    shapes.push(border);

    let nlines = 6;
    let step = 1.0 / nlines as f32;
    for i in 1..nlines {
        let dist = i as f32 * step;
        let p1 = to_screen * Pos2::new(dist, 0.0);
        let p2 = to_screen * Pos2::new(1.0, 1.0 - dist);
        let quad1_diag = Shape::line_segment([p1, p2], stroke);
        shapes.push(quad1_diag);
        let p1 = to_screen * Pos2::new(dist, 0.0);
        let p2 = to_screen * Pos2::new(0.0, dist);
        let quad2_diag = Shape::line_segment([p1, p2], stroke);
        shapes.push(quad2_diag);
        let p1 = to_screen * Pos2::new(0.0, dist);
        let p2 = to_screen * Pos2::new(1.0 - dist, 1.0);
        let quad3_diag = Shape::line_segment([p1, p2], stroke);
        shapes.push(quad3_diag);
        let p1 = to_screen * Pos2::new(dist, 1.0);
        let p2 = to_screen * Pos2::new(1.0, dist);
        let quad4_diag = Shape::line_segment([p1, p2], stroke);
        shapes.push(quad4_diag);
    }
    shapes
}
