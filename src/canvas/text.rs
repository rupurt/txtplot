use colored::Color;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextIntensity {
    #[default]
    Normal,
    Bold,
    Dim,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextStyle {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub intensity: TextIntensity,
}

impl TextStyle {
    pub const fn new() -> Self {
        Self {
            foreground: None,
            background: None,
            intensity: TextIntensity::Normal,
        }
    }

    pub const fn with_foreground(mut self, color: Color) -> Self {
        self.foreground = Some(color);
        self
    }

    pub const fn with_background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub const fn with_intensity(mut self, intensity: TextIntensity) -> Self {
        self.intensity = intensity;
        self
    }

    pub const fn bold(mut self) -> Self {
        self.intensity = TextIntensity::Bold;
        self
    }

    pub const fn dim(mut self) -> Self {
        self.intensity = TextIntensity::Dim;
        self
    }

    pub const fn normal(mut self) -> Self {
        self.intensity = TextIntensity::Normal;
        self
    }
}

impl From<Color> for TextStyle {
    fn from(color: Color) -> Self {
        Self::new().with_foreground(color)
    }
}

impl From<Option<Color>> for TextStyle {
    fn from(foreground: Option<Color>) -> Self {
        Self {
            foreground,
            ..Self::new()
        }
    }
}
