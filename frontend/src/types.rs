use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    English,
    Chinese,
    Spanish,
    German,
    Japanese,
    French,
    Portuguese,
    Russian,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Chinese => "zh",
            Self::Spanish => "es",
            Self::German => "de",
            Self::Japanese => "ja",
            Self::French => "fr",
            Self::Portuguese => "pt",
            Self::Russian => "ru",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Chinese => "简体中文",
            Self::Spanish => "Español",
            Self::German => "Deutsch",
            Self::Japanese => "日本語",
            Self::French => "Français",
            Self::Portuguese => "Português",
            Self::Russian => "Русский",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "zh" => Self::Chinese,
            "es" => Self::Spanish,
            "de" => Self::German,
            "ja" => Self::Japanese,
            "fr" => Self::French,
            "pt" => Self::Portuguese,
            "ru" => Self::Russian,
            _ => Self::English,
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::English,
            Self::Chinese,
            Self::Spanish,
            Self::German,
            Self::Japanese,
            Self::French,
            Self::Portuguese,
            Self::Russian,
        ]
    }
}
