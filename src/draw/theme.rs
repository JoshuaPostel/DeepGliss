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

        // 1
        // purple
        Color32::from_rgb(106, 51, 194),
        // 2
        // yellow
        Color32::from_rgb(255, 215, 0),
        // 3
        // green
        Color32::from_rgb(51, 160, 44),
        // 4
        // red
        Color32::from_rgb(255, 0, 0),
        // 5
        // blue
        Color32::from_rgb(31, 120, 200),
        // 6
        // orange
        Color32::from_rgb(255, 127, 0),
        // 7
        // pink
        Color32::from_rgb(251, 100, 150),
        // 8
        // teal
        Color32::from_rgb(0, 226, 229),
        // 9
        // bright purple
        Color32::from_rgb(200, 20, 250),
        // 10 
        // yellow gold
        Color32::from_rgb(218, 165, 32),
        // 11
        // bright green
        Color32::from_rgb(18, 200, 40),
        // 12
        // dark red
        Color32::from_rgb(153, 15, 38),
        // 13
        // blue green
        Color32::from_rgb(0, 241, 154),
        // 14
        // orange 2
        Color32::from_rgb(255, 80, 25),
        // 15
        // saturated pink
        Color32::from_rgb(250, 0, 135),
    ],
    piano: PianoTheme {
        //good for thatched keys
        //white: Color32::from_rgb(192, 192, 192),
        //good for solid keys
        white: Color32::from_rgb(120, 120, 120),
        black: Color32::from_rgb(64, 64, 64),
        white_active: Color32::from_rgb(150, 129, 184),
        black_active: Color32::from_rgb(106, 51, 194),
    },
};
