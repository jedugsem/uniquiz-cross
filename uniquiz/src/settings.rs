use crate::{
    comps::searchbar,
    dir, fl,
    git::{self, GitRepo},
    per::Com,
    Controls, Element, Message,
};
use iced::{
    alignment::Horizontal::Right,
    widget::{column, container, row, text, toggler, Space},
    Alignment::Center,
    Length::{self, Fill},
    Task,
};
use iced_material::theme::{self, container::grey_rounded};
use iced_widget::{button, pick_list, text_editor};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

const WIDTH: u16 = 500;
#[derive(Clone, Debug)]
pub enum SettingsM {
    Page(Settings),
    PasteUrl,
    PasteSsh,
    NoProg,
    AddProg,
    Remove(GitRepo),
    Update(GitRepo),
    AddRepo,
    ThemeChange(Them),
    LangChange(Language),
    CheckboxFeedback(bool),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Settings {
    Prog,
    Manage,
    Repos,
    Normal,
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
            SettingsM::Update(git) => {
                _ = std::fs::remove_dir_all(dir().join(git.path.clone()));
                Com::none()
            }
            SettingsM::Remove(git) => {
                //
                _ = std::fs::remove_dir_all(dir().join(git.path.clone()));
                let position = self.state.mo.repos.iter().position(|x| x.path == git.path);
                if let Some(pos) = position {
                    self.state.mo.repos.swap_remove(pos);
                }
                let path = dir().join("progress");
                if path.exists() {
                    if let Ok(file) = &mut std::fs::File::create(path.join("modules.ron")) {
                        let _ = ron::Options::default().to_io_writer_pretty(
                            file,
                            &self.state.mo,
                            PrettyConfig::default(),
                        );
                    }
                }

                Com::none()
            }
            SettingsM::Page(sett) => {
                self.state.sett = sett;
                Com::none()
            }
            SettingsM::PasteUrl => {
                #[cfg(target_os = "android")]
                {
                    _ = self.proxy.send_event(crate::UserEvent::ClipboardRead(1));
                }

                Com::none()
            }
            SettingsM::PasteSsh => {
                #[cfg(target_os = "android")]
                {
                    _ = self.proxy.send_event(crate::UserEvent::ClipboardRead(2));
                }

                Com::none()
            }
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
            SettingsM::NoProg => {
                if dir().join("progress").exists() {
                    let _ = std::fs::remove_dir_all(dir().join("progress"));
                }
                let _ = std::fs::create_dir_all(dir().join("progress"));
                self.ssh_editor = text_editor::Content::new();
                self.prog_editor = text_editor::Content::new();
                self.state.settings.prog_git = None;
                self.state.sett = Settings::Normal;
                let settings = self.state.settings.clone();
                Com::perform(
                    &self,
                    async move {
                        crate::per::write_settings(settings);
                        Message::Boot.into()
                    },
                    |m| m,
                )
            }
            SettingsM::AddProg => {
                if dir().join("progress").exists() {
                    let _ = std::fs::remove_dir_all(dir().join("progress"));
                }
                let ssh = self.ssh_editor.text();
                let git_url = self.prog_editor.text();
                self.ssh_editor = text_editor::Content::new();
                self.prog_editor = text_editor::Content::new();
                self.state.settings.prog_git = Some(GitRepo {
                    repo: git_url,
                    ssh_priv: Some(ssh),
                    path: "progress".to_string(),
                });
                self.state.sett = Settings::Normal;
                let settings = self.state.settings.clone();
                let git = self.state.settings.prog_git.clone().unwrap();
                Com::perform(
                    &self,
                    async move {
                        _ = git::clone(git).await;
                        crate::per::write_settings(settings);
                        Message::Boot.into()
                    },
                    |m| m,
                )
            }
            SettingsM::AddRepo => {
                let _ = std::fs::create_dir_all(dir().join("modules"));
                let git_url = self.prog_editor.text();
                self.ssh_editor = text_editor::Content::new();
                self.prog_editor = text_editor::Content::new();
                let name = git_url.split("/").last().unwrap();

                if dir().join("modules").join(name).exists() {
                    let _ = std::fs::remove_dir_all(dir().join("modules").join(name));
                }

                let repo = GitRepo {
                    repo: git_url.clone(),
                    ssh_priv: None,
                    path: format!("modules/{}", name),
                };
                self.state.mo.repos.push(repo.clone());
                self.state.sett = Settings::Normal;
                let settings = self.state.settings.clone();
                let path = dir().join("progress");
                if path.exists() {
                    if let Ok(file) = &mut std::fs::File::create(path.join("modules.ron")) {
                        let _ = ron::Options::default().to_io_writer_pretty(
                            file,
                            &self.state.mo,
                            PrettyConfig::default(),
                        );
                    }
                }

                Com::perform(
                    &self,
                    async move {
                        crate::per::write_settings(settings);
                        Message::Boot.into()
                    },
                    |m| m,
                )
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
        match self.state.sett {
            Settings::Manage => {
                let vec: Vec<Element<Message>> = self
                    .state
                    .mo
                    .repos
                    .iter()
                    .map(|x| {
                        //
                        container(
                            row!(
                                text!(" {}", x.path)
                                    .width(Fill)
                                    .height(Fill)
                                    .align_x(Center)
                                    .align_y(Center),
                                button("Update")
                                    .height(40)
                                    .width(70)
                                    .on_press(SettingsM::Update(x.clone()).into()),
                                Space::with_width(5),
                                button("Re")
                                    .height(40)
                                    .width(40)
                                    .on_press(SettingsM::Remove(x.clone()).into())
                            )
                            .align_y(Center),
                        )
                        .padding(5)
                        .style(grey_rounded)
                        .width(Fill)
                        .height(50)
                        .into()
                    })
                    .collect();
                //
                column!(
                    row!(
                        column!(button("change prog repo")
                            .on_press(SettingsM::Page(Settings::Prog).into()))
                        .width(Fill)
                        .align_x(Center),
                        column!(
                            button("add module").on_press(SettingsM::Page(Settings::Repos).into())
                        )
                        .width(Fill)
                        .align_x(Center)
                    )
                    .align_y(Center)
                    .height(40),
                    column(vec).spacing(5).padding(5),
                )
                .into()
            }
            Settings::Prog => {
                //
                column!(
                    text("Add Prog Repo"),
                    text("Git Url:"),
                    searchbar(
                        &self.prog_editor,
                        |s| Message::EditorAction(s, 1),
                        Some(|| SettingsM::PasteUrl.into())
                    ),
                    text("Ssh Key:"),
                    searchbar(
                        &self.ssh_editor,
                        |s| Message::EditorAction(s, 2),
                        Some(|| SettingsM::PasteSsh.into())
                    ),
                    column!(
                        button("No Prog").on_press(SettingsM::NoProg.into()),
                        button("Add").on_press(SettingsM::AddProg.into())
                    )
                    .spacing(10)
                    .padding(10)
                    .width(Fill)
                    .align_x(Right),
                )
                .align_x(Center)
                .padding(15)
                .spacing(10)
                .into()
            }
            Settings::Repos => {
                //
                column!(
                    text("Add Module"),
                    text("Git Url:"),
                    searchbar(
                        &self.prog_editor,
                        |s| Message::EditorAction(s, 1),
                        Some(|| SettingsM::PasteUrl.into())
                    ),
                    // text("Ssh Key:"),
                    // searchbar(
                    //     &self.ssh_editor,
                    //     |s| Message::EditorAction(s, 2),
                    //     Some(|| SettingsM::PasteSsh.into())
                    // ),
                    column!(button("Add").on_press(SettingsM::AddRepo.into()))
                        .width(Fill)
                        .align_x(Right),
                )
                .align_x(Center)
                .padding(15)
                .spacing(10)
                .into()
            }

            //Settings::Repos => button("safe repo").on_press(Message::Boot).into(),
            Settings::Normal => {
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
                    row!(
                        text("manage repos"),
                        Space::with_width(Length::Fill),
                        button("manage").on_press(SettingsM::Page(Settings::Manage).into())
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
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PSettings {
    pub prog_git: Option<GitRepo>,
    // Repos
    pub lang: Option<Language>,
    pub feedback: bool,
    pub theme: Option<Them>,
}
impl Default for PSettings {
    fn default() -> Self {
        Self {
            prog_git: None,
            lang: Some(Language::De),
            feedback: true,
            theme: Some(Them::Default),
        }
    }
}
