use crate::{fl, per::*, Controls, Element, Load, Loaded, Message};
use iced::{
    widget::{button, column, row, scrollable, text, Space},
    Alignment, Length, Padding,
};
use iced_winit::runtime::Task;
//use quizlib::*;
use quizlib::{Db, Modul, TSafe};

use super::stepbystep::PSafe;
// Modules to be loaded

#[derive(Debug, Clone)]
pub enum DbM {
    LoadedDb(Result<Db, String>, Modul),
    LoadedState(Result<TSafe, String>),
    Load(Modul),
    LoadedModules(Vec<Modul>),
}

impl From<DbM> for Message {
    fn from(m: DbM) -> Message {
        Message::Db(m)
    }
}
impl Controls {
    pub fn view_load(&self) -> Element<Message> {
        if let Load {
            modules: Some(modules),
            ..
        } = &self.state.modules
        {
            // webbrowser::open("http://github.com");
            let dbs: Vec<Element<Message>> = modules
                .iter()
                .map(|nns| {
                    button(column!(
                        row!(
                            text(nns.desc.name.clone()),
                            Space::with_width(Length::Fill),
                            text(format!("{}: {}", fl!("version"), nns.desc.version))
                        ),
                        row!(
                            text(format!("{}: {:?}", fl!("count"), nns.desc.count)),
                            Space::with_width(Length::Fill),
                            text(format!("{}: {} min", fl!("duration"), nns.desc.time))
                        ),
                    ))
                    .padding(Padding::from([5, 15]))
                    .width(500)
                    .on_press(DbM::Load(nns.clone()).into())
                    .into()
                })
                .collect();
            let error: Element<Message> = if let Some(err) = &self.state.modules.err {
                column!(text!("Error: {}", err))
                    .align_x(Alignment::Center)
                    .into()
            } else {
                column!().into()
            };
            column!(
                scrollable(
                    column!(text(fl!("databases")), column(dbs))
                        .spacing(20)
                        .padding(20)
                        .align_x(Alignment::Center),
                ),
                error,
            )
            .into()
        } else {
            text(fl!("no-db-found")).into()
        }
    }

    pub fn update_db(&mut self, m: DbM) -> Task<Message> {
        match m {
            DbM::LoadedModules(modules) => {
                self.state.modules = Load {
                    modules: Some(modules),
                    err: None,
                };
                Com::save(&self)
            }
            DbM::Load(modul) => Com::perform(
                &self,
                async move { DbM::LoadedDb(load_db(&modul.path), modul) },
                Message::Db,
            ),
            DbM::LoadedDb(db, modul) => {
                match db {
                    Ok(db) => {
                        self.state.loaded = Some(Loaded {
                            db,
                            module: modul.clone(),
                            ..Default::default()
                        });
                    }
                    Err(err) => {
                        self.state.modules.err = Some(err);
                    }
                };
                Com::perform(
                    &self,
                    async move { DbM::LoadedState(load_progress(modul.path)) },
                    Message::Db,
                )
            }
            DbM::LoadedState(state) => {
                match state {
                    Ok(state) => {
                        if let Some(loaded) = &mut self.state.loaded {
                            loaded.stepby = Some(PSafe {
                                tsafe: state,
                                ..Default::default()
                            });
                        }
                    }
                    _ => {
                        if let Some(loaded) = &mut self.state.loaded {
                            loaded.stepby = Some(PSafe {
                                tsafe: TSafe::default(),
                                ..Default::default()
                            });
                        }
                    }
                }
                Com::perform(&self, async {}, |_| Message::Select(1))
            }
        }
    }
}
// pub fn _init_modules() -> Command<Message> {
//     Command::perform(
//         async move { Message::Db(DbM::LoadedModules(get_modules().unwrap())) },
//         |x| x,
//     )
// }
