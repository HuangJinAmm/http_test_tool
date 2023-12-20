use eframe::egui::Color32;

#[derive(Clone)]
pub struct ThemePalette {
    pub black: Color32,
    pub red: Color32,
    pub green: Color32,
    pub yellow: Color32,
    pub blue: Color32,
    pub magenta: Color32,
    pub cyan: Color32,
    pub white: Color32,
}

impl ThemePalette {
    pub const DARK: Self = Self {
        black: Color32::from_rgb(0, 0, 0),
        red: Color32::from_rgb(255, 69, 58),
        green: Color32::from_rgb(50, 215, 75),
        yellow: Color32::from_rgb(255, 214, 10),
        blue: Color32::from_rgb(10, 132, 255),
        magenta: Color32::from_rgb(191, 90, 242),
        cyan: Color32::from_rgb(90, 200, 245),
        white: Color32::from_rgb(255, 255, 255),
    };

    pub const LIGHT: Self = Self {
        black: Color32::from_rgb(0, 0, 0),
        red: Color32::from_rgb(255, 59, 48),
        green: Color32::from_rgb(40, 205, 65),
        yellow: Color32::from_rgb(255, 204, 0),
        blue: Color32::from_rgb(10, 132, 255),
        magenta: Color32::from_rgb(175, 82, 222),
        cyan: Color32::from_rgb(85, 190, 240),
        white: Color32::from_rgb(255, 255, 255),
    };

    // todo: passing the is_dark_mode aram doesn't feel like good data modeling
    pub fn as_array(is_dark_mode: bool) -> Vec<(String, Color32)> {
        let palette = if is_dark_mode { ThemePalette::DARK } else { ThemePalette::LIGHT };

        vec![
            ("magenta".to_string(), palette.magenta),
            ("blue".to_string(), palette.blue),
            ("cyan".to_string(), palette.cyan),
            ("green".to_string(), palette.green),
            ("yellow".to_string(), palette.yellow),
            ("red".to_string(), palette.red),
            ("fg".to_string(), if is_dark_mode { palette.white } else { palette.black }),
        ]
    }
}


#[derive(Clone)]
pub struct DrawingPalette {
    pub black: Color32,
    pub red: Color32,
    pub green: Color32,
    pub yellow: Color32,
    pub blue: Color32,
    pub magenta: Color32,
    pub cyan: Color32,
    pub white: Color32,
}

impl DrawingPalette {
    pub const DARK: Self = Self {
        black: Color32::from_rgb(0, 0, 0),
        red: Color32::from_rgb(255, 69, 58),
        green: Color32::from_rgb(50, 215, 75),
        yellow: Color32::from_rgb(255, 214, 10),
        blue: Color32::from_rgb(10, 132, 255),
        magenta: Color32::from_rgb(191, 90, 242),
        cyan: Color32::from_rgb(90, 200, 245),
        white: Color32::from_rgb(255, 255, 255),
    };

    pub const LIGHT: Self = Self {
        black: Color32::from_rgb(0, 0, 0),
        red: Color32::from_rgb(255, 59, 48),
        green: Color32::from_rgb(40, 205, 65),
        yellow: Color32::from_rgb(255, 204, 0),
        blue: Color32::from_rgb(0, 122, 255),
        magenta: Color32::from_rgb(175, 82, 222),
        cyan: Color32::from_rgb(85, 190, 240),
        white: Color32::from_rgb(255, 255, 255),
    };

}
