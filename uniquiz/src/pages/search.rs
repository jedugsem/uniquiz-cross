use iced::{
    widget::{button, column, scrollable, text, text_editor},
    Length, Task,
};
use iced_widget::container;
use quizlib::*;
use serde::{Deserialize, Serialize};

use crate::{comps::searchbar, per::Com, Controls, Element, Loaded, Message};

use super::stepbystep::PSafe;

#[derive(Debug, Clone)]
pub enum SearchM {
    Init,
    Selectt(usize),
    Search(String),
    Back,
}
impl From<SearchM> for Message {
    fn from(m: SearchM) -> Message {
        Message::Search(m)
    }
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Search {
    pub search_index: usize,
    pub search: String,
    pub card_search: bool,
    pub psafe: PSafe,
}

impl Controls {
    pub fn update_search(&mut self, m: SearchM, _page: u8) -> Task<Message> {
        match &mut self.state.loaded {
            Some(Loaded {
                db,
                card,
                search: Some(search),
                ..
            }) => match m {
                SearchM::Search(m) => {
                    search.search = m.trim().to_string();
                    Com::save(&self)
                }
                SearchM::Init => Com::none(),
                SearchM::Back => match search.card_search {
                    true => {
                        search.card_search = false;

                        Com::save(&self)
                    }
                    false => Com::perform(&self, async { 0 }, Message::Select),
                },
                SearchM::Selectt(index) => {
                    search.psafe.question = db
                        .questions
                        .iter()
                        .filter(|quest| {
                            if let Some(question) = &quest.question.text {
                                question
                                    .to_lowercase()
                                    .contains(&search.search.to_lowercase())
                            } else {
                                false
                            }
                        })
                        .cloned()
                        .collect();
                    search.psafe.current = index;
                    card.prepare_vector(&search.psafe.question[search.psafe.current], true);
                    search.card_search = true;
                    search.search_index = index;
                    Com::save(&self)
                }
            },

            _ => Com::none(),
        }
    }

    pub fn view_search(&self) -> Element<Message> {
        if let Some(Loaded {
            search: Some(search),
            ..
        }) = &self.state.loaded
        {
            match &self.state.loaded {
                Some(Loaded { db, card, .. }) => {
                    if !search.card_search {
                        let w: Element<Message> = if search.search.len() < 3 {
                            column!().into()
                        } else {
                            let search = &search.search;
                            let coo = db
                                .questions
                                .iter()
                                .filter(|quest| match quest {
                                    Question {
                                        question:
                                            Frage {
                                                text: Some(question),
                                                ..
                                            },
                                        awnsers: Awnsers::Multiple(_vect),
                                        ..
                                    } => {
                                        question.to_lowercase().contains(&search.to_lowercase())
                                        //|| vect.iter().filter(|stri| stri.0.contains(search)).count() > 0
                                    }
                                    Question { .. } => false,
                                })
                                .enumerate()
                                .map(|(i, quest)| {
                                    if let Some(question) = &quest.question.text {
                                        //let splits = question.split(search);
                                        button(text(question))
                                            .on_press(SearchM::Selectt(i).into())
                                            .into()
                                    } else {
                                        column!().into()
                                    }
                                });
                            scrollable(column(coo).padding(10).spacing(10)).into()
                        };
                        column!(
                            container(searchbar(&self.editor, Message::EditorAction))
                                .center_x(Length::Fill),
                            w
                        )
                        .padding(10)
                        .spacing(5)
                        .into()
                    } else {
                        card.view(
                            &search.psafe.question[search.psafe.current],
                            self.state.settings.feedback,
                        )
                    }
                    //fds
                }
                _ => text("").into(),
            }
        } else {
            text("not loaded").into()
        }
    }
}
