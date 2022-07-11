#[derive(Clone, Debug, Default)]
pub(crate) enum Color {
    Colored(colored::Color),
    #[default]
    None,
    Raw(String),
}

impl Color {
    pub fn colorize(&self, s: &str) -> String {
        use colored::Colorize;

        if !colored::control::SHOULD_COLORIZE.should_colorize() {
            return s.to_string();
        }

        match self {
            Self::Colored(color) => s.color(*color).to_string(),
            Self::None => s.to_string(),
            Self::Raw(color) => format!("{color}{s}\x1B[0m"),
        }
    }
}

impl From<&str> for Color {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or_default()
    }
}

impl std::str::FromStr for Color {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let color = if let Ok(color) = s.parse() {
            Self::Colored(color)
        } else {
            Self::Raw(s.replace("\\\\033", "\x1B"))
        };

        Ok(color)
    }
}

impl ToString for Color {
    fn to_string(&self) -> String {
        match self {
            Self::Colored(color) => color.to_bg_str().to_string(),
            Self::None => String::new(),
            Self::Raw(color) => color.clone(),
        }
    }
}
