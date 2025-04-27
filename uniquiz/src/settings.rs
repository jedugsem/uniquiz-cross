use crate::{fl, per::Com, Controls, Element, Message};
use iced::{
    widget::{column, container, row, text, toggler, Space},
    Length, Task,
};
use iced_material::theme;
use iced_widget::{button, pick_list};
use serde::{Deserialize, Serialize};

const WIDTH: u16 = 500;
#[derive(Clone, Debug)]
pub enum SettingsM {
    ThemeChange(Them),
    LangChange(Language),
    CheckboxFeedback(bool),
}

impl From<SettingsM> for Message {
    fn from(m: SettingsM) -> Message {
        Message::Settings(m)
    }
}
pub enum Lang {
    Custom(String),
    System(String),
}
impl Controls {
    pub fn update_settings(&mut self, m: SettingsM) -> Task<Message> {
        match m {
            SettingsM::LangChange(lang) => {
                let mut la = crate::localize::LANG.lock().unwrap();
                *la = lang;
                self.state.settings.lang = Some(lang);
                self.write_settings()
            }
            SettingsM::CheckboxFeedback(bool) => {
                self.state.settings.feedback = bool;
                self.write_settings()
            }
            SettingsM::ThemeChange(them) => {
                self.state.settings.theme = Some(them);
                match them {
                    Them::Dark => self.theme = theme::Theme::dark(),
                    Them::Light => self.theme = theme::Theme::light(),
                    Them::Default => self.theme = theme::Theme::default(),
                }
                self.write_settings()
            }
        }
    }
    fn write_settings(&self) -> Task<Message> {
        let settings = self.state.settings.clone();
        Com::perform(
            &self,
            async move {
                crate::per::write_settings(settings);
                Message::Nothing.into()
            },
            |m| m,
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    System,
    De,
    En,
}

impl Language {
    const ALL: [Language; 3] = [Self::System, Self::En, Self::De];
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::En => fl!("english"),
                Self::De => fl!("german"),
                Self::System => fl!("system"),
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum Them {
    Light,
    Dark,
    #[default]
    Default,
}

impl Them {
    const ALL: [Them; 3] = [Self::Dark, Self::Light, Self::Default];
}

impl std::fmt::Display for Them {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Light => fl!("light-theme"),
                Self::Dark => fl!("dark-theme"),
                Self::Default => fl!("default-theme"),
            }
        )
    }
}

impl Controls {
    pub fn view_settings(&self) -> Element<Message> {
        let vec: Vec<Element<Message>> = vec![
            row!(
                text(fl!("feedback")),
                Space::with_width(Length::Fill),
                toggler(self.state.settings.feedback)
                    .on_toggle(|boo| Message::Settings(SettingsM::CheckboxFeedback(boo)))
                    .size(20.)
                    .width(100)
            )
            .spacing(10)
            .into(),
            row!(
                text(fl!("theme")),
                Space::with_width(Length::Fill),
                pick_list(Them::ALL, self.state.settings.theme, |x| {
                    SettingsM::ThemeChange(x).into()
                }),
            )
            .into(),
            row!(
                text(fl!("language")),
                Space::with_width(Length::Fill),
                pick_list(Language::ALL, self.state.settings.lang, |x| {
                    SettingsM::LangChange(x).into()
                }),
            )
            .into(),
        ];
        container(
            column(vec)
                .spacing(15)
                .width(Length::Fixed(WIDTH as f32))
                .padding(20),
        )
        .center_x(Length::Fill)
        .into()
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PSettings {
    pub lang: Option<Language>,
    pub feedback: bool,
    pub theme: Option<Them>,
}
impl Default for PSettings {
    fn default() -> Self {
        Self {
            lang: Some(Language::De),
            feedback: true,
            theme: Some(Them::Default),
        }
    }
}
