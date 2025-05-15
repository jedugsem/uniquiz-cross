use std::path::PathBuf;

use crate::comps::togg;
use crate::pages::search::Search;
use crate::{fl, pages::katselect::SelectState, per::Com, Controls, Element, Loaded, Message};
use iced::Alignment::Center;
use iced::Length::Fill;
use iced::{
    widget::{button, column, container, row, scrollable, text, Space},
    Alignment, Length, Task,
};
use iced_material::{icon, theme};
use quizlib::*;
use serde::{Deserialize, Serialize};
// Card Messages
#[derive(Debug, Clone)]
pub enum CardM {
    Next(bool),
    Toogle(usize, bool),
}
impl From<CardM> for Message {
    fn from(m: CardM) -> Message {
        Message::Card(m)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Questi {
    pub quest: bool,
    pub is_checked: Vec<bool>,
}
impl Default for Questi {
    fn default() -> Self {
        Questi {
            quest: true,
            is_checked: vec![],
        }
    }
}
impl Questi {
    pub fn view(&self, question: &Question, errors: bool) -> Element<Message> {
        let quests: Vec<Element<Message>> = match &question.awnsers {
            Awnsers::Multiple(textvec) => {
                textvec
                    .iter()
                    .enumerate()
                    .map(|(i, awnser)| {
                        if errors && !self.quest {
                            let color = if awnser.1 == self.is_checked[i] {
                                theme::svg::success
                            } else {
                                theme::svg::error
                            };
                            //✓
                            let err = if awnser.1 {
                                column!(icon!("check").width(20).height(20).style(color))
                            } else {
                                column!()
                            }
                            .width(20);
                            let check = togg(
                                row!(err, text!("{}", &awnser.0)).spacing(10).padding(5),
                                self.is_checked[i],
                                |_| Message::Nothing,
                            )
                            .width(Length::Fill);

                            row!(check).spacing(10).align_y(Alignment::Center).into()
                        } else {
                            if self.is_checked.len() > i {
                                togg(
                                    column!(text!("{}", &awnser.0)).padding(5),
                                    self.is_checked[i],
                                    move |is| CardM::Toogle(i, is).into(),
                                )
                                .width(Length::Fill)
                                .into()
                            } else {
                                text(fl!("card-error")).into()
                            }
                        }
                    })
                    .collect()
            }
            _ => {
                vec![]
            }
        };
        let quest: Element<Message> = match &question.question {
            Frage {
                text: Option::Some(tex),
                extras: _,
            } => column!(text!("{}", tex), column(quests).spacing(10))
                .spacing(15)
                .width(Length::Fill)
                .align_x(Alignment::Start)
                .into(),
            _ => column!().into(),
        };

        column!(
            row![
                text!("{} {}", fl!("question"), question.index + 1).size(20.),
                Space::with_width(Length::Fill),
                text!("{} {}", fl!("kategory"), format_kat(&question.id)).size(20.),
            ]
            .padding(10),
            container(scrollable(
                column!(
                    quest,
                    button(text(fl!("next"))).on_press(CardM::Next(errors).into()),
                )
                .align_x(Center)
                .width(Fill)
                .padding(15)
                .spacing(10)
            ))
            .padding(10),
            Space::new(0, 20),
        )
        .height(600)
        .width(500)
        .spacing(10)
        .align_x(Alignment::Center)
        .into()
    }
    pub fn prepare_vector(&mut self, quest: &Question, is_quest: bool) {
        if let Awnsers::Multiple(vec) = &quest.awnsers {
            if is_quest {
                self.is_checked = vec![false; vec.len()];
            } else {
                self.is_checked = vec.iter().map(|x| x.1).collect();
            }
        }
    }
}
impl Controls {
    pub fn update_card(&mut self, m: CardM, tab: u8) -> Task<Message> {
        match &mut self.state.loaded {
            Some(Loaded {
                module,
                card,
                test,
                kat,
                stepby,
                search,
                ..
            }) => {
                match m {
                    CardM::Toogle(i, bool) => {
                        card.is_checked[i] = bool;
                        Com::save(&self)
                    }
                    CardM::Next(errors) => {
                        let _data = format!("hey {}", tab);

                        match tab {
                            4 => match test {
                                Some(test) => {
                                    test.data.add_solution(
                                        &test.questions_selected[test.current],
                                        &card.is_checked,
                                    );
                                    if test.current < (test.questions_selected.len() - 1) {
                                        test.current += 1;
                                    }

                                    println!(
                                        "test cur{},len{}",
                                        test.current,
                                        test.questions_selected.len()
                                    );
                                    card.prepare_vector(
                                        &test.questions_selected[test.current],
                                        true,
                                    );
                                    Com::save(&self)
                                }
                                _ => Com::none(),
                            },
                            5 => match search {
                                Some(Search { psafe, .. }) => match (errors, card.quest) {
                                    (true, true) => {
                                        card.quest = false;
                                        Com::save(&self)
                                    }
                                    (_, _) => {
                                        card.quest = true;
                                        if psafe.current < (psafe.question.len() - 1) {
                                            psafe.current += 1;
                                        } else {
                                            psafe.question_visible = false;
                                        }
                                        println!(
                                            "kat cur{},len{}",
                                            psafe.current,
                                            psafe.question.len()
                                        );
                                        card.prepare_vector(&psafe.question[psafe.current], true);
                                        Com::save(&self)
                                    }
                                },
                                _ => Com::none(),
                            },

                            3 => match kat {
                                Some(SelectState { psafe, .. }) => match (errors, card.quest) {
                                    (true, true) => {
                                        card.quest = false;
                                        Com::save(&self)
                                    }
                                    (_, _) => {
                                        card.quest = true;
                                        if psafe.current < (psafe.question.len() - 1) {
                                            psafe.current += 1;
                                        } else {
                                            psafe.question_visible = false;
                                        }

                                        println!(
                                            "kat cur{},len{}",
                                            psafe.current,
                                            psafe.question.len()
                                        );
                                        card.prepare_vector(&psafe.question[psafe.current], true);
                                        Com::save(&self)
                                    }
                                },
                                _ => Com::none(),
                            },
                            1 => match stepby {
                                Some(psafe) => match (errors, card.quest) {
                                    (true, true) => {
                                        card.quest = false;
                                        Com::save(&self)
                                    }
                                    (_, _) => {
                                        card.quest = true;
                                        psafe.tsafe.add_solution(
                                            &psafe.question[psafe.current],
                                            &card.is_checked,
                                        );
                                        if psafe.current < (psafe.question.len() - 1) {
                                            psafe.current += 1;
                                        }
                                        //println!("step cur{},len{}", psafe.current, psafe.question.len());

                                        card.prepare_vector(&psafe.question[psafe.current], true);

                                        let progress = psafe.tsafe.clone();
                                        let path = module.path.clone();
                                        self.write_progress(progress, path)
                                    }
                                },
                                _ => Com::none(),
                            },
                            _ => Com::none(),
                        }
                    }
                }
            }
            _ => Com::none(),
        }
    }
    fn write_progress(&self, prog: TSafe, path: PathBuf) -> Task<Message> {
        let git = self.state.settings.prog_git.clone();
        Com::perform(
            &self,
            async move {
                crate::per::write_progress(git, &prog, path).await;
            },
            |_| Message::Nothing,
        )
    }
}
