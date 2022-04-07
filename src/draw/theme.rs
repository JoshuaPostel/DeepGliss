use egui::Color32;

pub struct PianoTheme {
    pub white: Color32,
    pub black: Color32,
    pub white_active: Color32,
    pub black_active: Color32,
}

pub struct Theme {
    pub channel_colors: [Color32; 15],
    pub piano: PianoTheme,
}

// TODO::UI pick 12 distinct colors
pub const GLISS_THEME: Theme = Theme {
    channel_colors: [
        Color32::from_rgba_premultiplied(123, 78, 242, 100),
        Color32::from_rgba_premultiplied(66, 252, 251, 100),
        Color32::from_rgba_premultiplied(147, 230, 72, 100),
        Color32::from_rgba_premultiplied(252, 190, 66, 100),
        Color32::from_rgba_premultiplied(242, 63, 64, 100),
        Color32::from_rgba_premultiplied(123, 78, 242, 60),
        Color32::from_rgba_premultiplied(66, 252, 251, 60),
        Color32::from_rgba_premultiplied(147, 230, 72, 60),
        Color32::from_rgba_premultiplied(252, 190, 66, 60),
        Color32::from_rgba_premultiplied(242, 63, 64, 60),
        Color32::from_rgba_premultiplied(123, 78, 242, 40),
        Color32::from_rgba_premultiplied(66, 252, 251, 40),
        Color32::from_rgba_premultiplied(147, 230, 72, 40),
        Color32::from_rgba_premultiplied(252, 190, 66, 40),
        Color32::from_rgba_premultiplied(242, 63, 64, 40),
    ],
    // green for accent color?
    piano: PianoTheme {
        white: Color32::from_rgb(192, 192, 192),
        black: Color32::from_rgb(64, 64, 64),
        white_active: Color32::from_rgb(193, 238, 197),
        black_active: Color32::from_rgb(80, 102, 82),
    },
};
