#![allow(unreachable_code)]

use std::sync::Arc;

#[cfg(not(target_os = "android"))]
pub use iced::Renderer;
use iced_graphics::text::cosmic_text::{fontdb::Source, Attrs};
use ron::de::from_bytes;
use serde::{Deserialize, Serialize};
pub mod back;
pub mod comps;
pub mod localize;
mod pages;
pub mod per;
pub mod settings;
use crate::back::back_message;
use comps::card::{CardM, Questi};
use iced::{
    widget::{column, container, responsive, row, text, text_editor, themer, Space},
    Alignment, Length, Theme,
};
use iced_material::{header::header, sidebar::sidebar, theme};
use iced_winit::runtime::Task;
use pages::{
    databases::DbM,
    katselect::{SelectM, SelectState},
    progress::ProgM,
    search::{Search, SearchM},
    stepbystep::{PSafe, StepM},
    test::{TestM, TestState},
};
use per::Com;
use quizlib::{Db, Modul};
use settings::{Language, PSettings, SettingsM, Them};
//use sys_locale::get_locale;
pub type Element<'a, Message> = iced::Element<'a, Message, theme::Theme, Renderer>;
// State Top Down
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uniquiz {
    // Pages - Optional
    pub modules: Load,
    // Window
    pub window: Window,
    // Sidebar
    pub loaded: Option<Loaded>,

    // Settings
    pub settings: PSettings,
    // Loading - Modules
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Window {
    pub keyboard: bool,
    pub title: String,
    pub settings_open: bool,
    pub sideselect: bool,
    pub sidebar: bool,
    pub tab: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Load {
    modules: Option<Vec<Modul>>,
    err: Option<String>,
}

// Loaded Database
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Loaded {
    // Modul + Db
    pub db: Db,
    pub module: Modul,

    // Card State
    pub card: Questi,

    // Pages
    pub prog: Option<usize>,
    pub stepby: Option<PSafe>,
    pub kat: Option<SelectState>,
    pub test: Option<TestState>,
    pub search: Option<Search>,
}
impl Default for Uniquiz {
    fn default() -> Self {
        let settings = per::load_settings().unwrap_or(PSettings::default());

        let mut lang = crate::localize::LANG.lock().unwrap();
        let mut languages = crate::localize::LANGUAGES.lock().unwrap();
        *languages = vec![
            from_bytes(include_bytes!("../i18n/de/uniquiz.ron")).unwrap(),
            from_bytes(include_bytes!("../i18n/en/uniquiz.ron")).unwrap(),
        ];

        if let Some(la) = settings.lang {
            *lang = la;
        }

        let modules = match per::get_modules() {
            Ok(ok) => Some(ok),
            Err(_err) => None,
        };

        Self {
            // Window
            // Sidebar
            loaded: None,

            window: Window::default(),
            // Settings
            settings,
            // Loading - Modules
            modules: Load { modules, err: None },
            // Pages - Optional
        }
    }
}
#[derive(Debug, Clone)]
pub enum Message {
    Boot,
    Clipboard(String, u8),
    Select(u8),
    Side,
    Back,
    Exit,
    Nothing,
    EditorAction(text_editor::Action),
    ToggleSettings,
    Settings(SettingsM),
    Card(CardM),
    Db(DbM),
    Step(StepM),
    Prog(ProgM),
    KatS(SelectM),
    Test(TestM),
    Search(SearchM),
}
const BREAKPOINT: f32 = 500.;
impl Clone for Controls {
    fn clone(&self) -> Self {
        Controls {
            editor: text_editor::Content::new(),
            theme: self.theme.clone(),
            state: self.state.clone(),
            #[cfg(target_os = "android")]
            background_color: self.background_color.clone(),
            #[cfg(target_os = "android")]
            proxy: self.proxy.clone(),
        }
    }
}
impl Controls {
    pub fn title(&self) -> String {
        "Uniquiz".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Boot => {
                //
                Com::none()
            }
            Message::Clipboard(m, n) => {
                match n {
                    0 => self
                        .editor
                        .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                            Arc::new(m),
                        ))),
                    _ => {}
                }
                Com::none()
            }

            Message::EditorAction(action) => match action {
                text_editor::Action::Click(_) => {
                    #[cfg(target_os = "android")]
                    let _ = self.proxy.send_event(crate::UserEvent::ShowKeyboard);
                    Com::none()
                }
                text_editor::Action::SelectWord => {
                    #[cfg(target_os = "android")]
                    let _ = self.proxy.send_event(crate::UserEvent::HideKeyboard);
                    Com::none()
                }
                text_editor::Action::Edit(_) => {
                    if let Some(Loaded {
                        search: Some(_search),
                        ..
                    }) = &mut self.state.loaded
                    {
                        self.editor.perform(action);
                        let text = self.editor.text();
                        Com::perform(&self, async move { text }, |x| SearchM::Search(x).into())
                    } else {
                        Com::save(&self)
                    }
                }

                other => {
                    if let Some(Loaded {
                        search: Some(_search),
                        ..
                    }) = &mut self.state.loaded
                    {
                        self.editor.perform(other);
                    }
                    Com::none()
                }
            },
            Message::Search(m) => self.update_search(m, self.state.window.tab),
            Message::Test(m) => self.update_test(m),
            Message::KatS(m) => self.update_select(m, self.state.window.tab),
            Message::Settings(m) => self.update_settings(m),
            Message::ToggleSettings => {
                let window = &mut self.state.window;
                match (window.sideselect, window.settings_open) {
                    (true, true) => {
                        window.settings_open = false;
                    }
                    (false, true) => {
                        window.sideselect = true;
                    }
                    _ => {
                        window.sideselect = true;
                        window.settings_open = !window.settings_open;
                    }
                }
                Com::save(&self)
            }
            Message::Back => {
                //
                let m = back_message(self.state.window.tab);
                Com::perform(&self, async move { m }, |x| x)
            }
            Message::Exit => {
                #[cfg(target_os = "android")]
                {
                    std::process::exit(0);
                    Com::none()
                }
                #[cfg(not(target_os = "android"))]
                iced::window::get_latest().and_then(|id| iced::window::close(id))
            }
            Message::Side => {
                let window = &mut self.state.window;
                if window.sideselect == true {
                    window.sideselect = false;
                    window.sidebar = true;
                } else {
                    window.sidebar = !window.sidebar;
                }
                //
                Com::none()
            }

            Message::Select(tab) => {
                let window = &mut self.state.window;
                if let Some(_loaded) = &mut self.state.loaded {
                    window.settings_open = false;
                    window.sideselect = true;
                    window.tab = tab;

                    match tab {
                        1 => Com::perform(&self, async { StepM::Init }, Message::Step),

                        3 => Com::perform(&self, async { SelectM::Init }, |y| y.into()),
                        4 => Com::perform(&self, async { TestM::Init }, |y| y.into()),
                        5 => {
                            _loaded.search = Some(Search::default());
                            Com::save(&self)
                        }
                        _ => Com::save(&self),
                    }
                } else {
                    if tab == 0 {
                        window.settings_open = false;
                        window.sideselect = true;
                        window.tab = 0;
                    }
                    Com::save(&self)
                }
            }
            Message::Nothing => Com::none(),
            Message::Card(m) => {
                if let Some(loaded) = &self.state.loaded {
                    if let Some(_ff) = &loaded.stepby {
                        self.update_card(m, self.state.window.tab)
                    } else {
                        Com::none()
                    }
                } else {
                    Com::none()
                }
                //
            }
            Message::Step(m) => self.update_step(m),
            Message::Prog(m) => self.update_prog(m),
            Message::Db(dbm) => self.update_db(dbm),
        }
    }

    pub fn view(&self) -> iced::Element<Message, Theme, Renderer> {
        let window = &self.state.window;
        let sidebar_widget: Element<Message> = responsive(move |size| {
            let content: Element<Message> = if window.settings_open {
                self.view_settings()
            } else {
                match window.tab {
                    0 => {
                        //
                        //
                        self.view_load()
                    }
                    1 => {
                        //
                        self.view_step()
                    }
                    2 => self.view_prog(),
                    3 => self.view_select(),
                    4 => self.view_test(),
                    5 => self.view_search(),
                    _ => text("failed").into(),
                }
            };
            let sidebar: Element<Message> = column!(sidebar(
                &[
                    &fl!("databases"),
                    &fl!("ongoing"),
                    &fl!("progress"),
                    &fl!("select"),
                    &fl!("test"),
                    &fl!("search"),
                ],
                Message::Select,
            ),)
            .align_x(Alignment::Center)
            .into();

            match (size, window.sidebar, window.sideselect) {
                (s, true, _) if s.width > BREAKPOINT => row!(
                    container(sidebar).width(Length::Fixed(300.)),
                    container(content).center_x(Length::Fill)
                )
                .into(),
                (s, _, true) if s.width <= BREAKPOINT => {
                    container(content).center_x(Length::Fill).into()
                }
                (_s, true, false) => container(sidebar).width(Length::Fill).into(),

                _ => container(content).center_x(Length::Fill).into(),
            }
        })
        .into();

        themer(
            self.theme.clone(),
            container(column![
                header(
                    Message::Side,
                    Message::Back,
                    Message::ToggleSettings,
                    Message::Exit,
                    "Uniquiz"
                ),
                sidebar_widget,
                Space::new(0, if cfg!(target_os = "android") { 17 } else { 0 })
            ])
            .style(theme::container::primary)
            .center(Length::Fill),
        )
        .into()
    }
}

#[cfg(target_os = "android")]
mod android {
    use crate::Message;
    pub use iced::Color;
    pub use iced_wgpu::Renderer;
    pub use iced_winit::winit::event_loop::EventLoopProxy;
    #[derive(Debug)]
    pub enum UserEvent {
        ClipboardRead(u8),
        ClipboardWrite(String),

        ShowKeyboard,
        Task(Message),
        HideKeyboard,
        Back,
    }
}
#[cfg(target_os = "android")]
pub use android::*;
pub struct Controls {
    pub editor: text_editor::Content<crate::Renderer>,
    pub theme: theme::Theme,
    pub state: Uniquiz,
    #[cfg(target_os = "android")]
    background_color: Color,
    #[cfg(target_os = "android")]
    proxy: EventLoopProxy<UserEvent>,
}

#[cfg(not(target_os = "android"))]
impl Default for Controls {
    fn default() -> Self {
        let uniquiz = Uniquiz::default();
        let theme = if let Some(them) = uniquiz.settings.theme {
            match them {
                Them::Dark => theme::Theme::dark(),
                Them::Light => theme::Theme::light(),
                Them::Default => theme::Theme::default(),
            }
        } else {
            theme::Theme::default()
        };
        Self {
            editor: text_editor::Content::new(),
            theme,
            state: Uniquiz::default(),
        }
    }
}
#[cfg(target_os = "android")]
impl Controls {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Controls {
        let state = Uniquiz::default();
        let theme = match state.settings.theme {
            Some(Them::Dark) => theme::Theme::dark(),
            Some(Them::Light) => theme::Theme::light(),
            Some(Them::Default) => theme::Theme::default(),
            _ => theme::Theme::default(),
        };
        let editor = match &state.loaded {
            Some(Loaded {
                search: Some(search),
                ..
            }) => text_editor::Content::with_text(&search.search.clone()),
            _ => text_editor::Content::new(),
        };
        Controls {
            state,
            theme,
            editor,
            background_color: Color::default(),
            proxy,
        }
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }
}
