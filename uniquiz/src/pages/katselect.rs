use iced::{
    widget::{button, scrollable, text},
    widget::{column, row},
    Length, Padding, Task,
};

use quizlib::*;
use serde::{Deserialize, Serialize};

use crate::{per::Com, Controls, Element, Loaded, Message, PSafe};
#[derive(Debug, Clone)]
pub enum SelectM {
    Init,
    Back,
    Select(Vec<u16>),
    Open(Vec<u16>),
}
impl From<SelectM> for Message {
    fn from(m: SelectM) -> Message {
        Message::KatS(m)
    }
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SelectState {
    pub psafe: PSafe,
    pub kat: Vec<u16>,
    pub question_visible: bool,
}
impl Controls {
    pub fn update_select(&mut self, m: SelectM, _page: u8) -> Task<Message> {
        match &mut self.state.loaded {
            Some(Loaded { db, card, kat, .. }) => {
                match m {
                    SelectM::Init => {
                        match kat {
                            Some(SelectState { psafe: _, .. }) => {
                                //card.prepare_vector(&psafe.question[psafe.current], true);
                            }
                            _ => {
                                *kat = Some(SelectState::default());
                            }
                        }
                        Com::save(&self)
                    }
                    SelectM::Back => match kat {
                        Some(SelectState {
                            question_visible: false,
                            kat,
                            ..
                        }) => {
                            if !kat.is_empty() {
                                kat.pop();
                                Com::save(&self)
                            } else {
                                Com::perform(&self, async { Message::Select(0) }, |y| y)
                            }
                        }
                        Some(SelectState {
                            question_visible, ..
                        }) if *question_visible == true => {
                            *question_visible = false;

                            Com::save(&self)
                        }
                        _ => Com::perform(&self, async { Message::Select(0) }, |m| m),
                    },
                    SelectM::Select(vec) => {
                        if db.kategorys[0].kat.len() >= (vec.len() + 1) {
                            let coo = db
                                .kategorys
                                .iter()
                                .filter(|kat| kat.kat.starts_with(&vec))
                                .collect::<Vec<&Kategory>>()
                                .len();
                            if coo == 1 {
                                //println!("last Kategory");
                                Com::perform(
                                    &self,
                                    async { Message::KatS(SelectM::Open(vec)) },
                                    |m| m,
                                )
                            } else {
                                if let Some(kat) = kat {
                                    kat.kat = vec;
                                }
                                Com::save(&self)
                            }
                        } else {
                            //println!("Error");
                            Com::perform(&self, async { Message::KatS(SelectM::Open(vec)) }, |m| m)
                        }
                    }
                    SelectM::Open(vec) => {
                        let mut psafe = PSafe::default();
                        psafe.question = db.get_kategory(&vec);
                        card.prepare_vector(&psafe.question[psafe.current], true);
                        let mut kategory = vec;
                        kategory.pop();
                        *kat = Some(SelectState {
                            question_visible: true,
                            psafe,
                            kat: kategory,
                        });
                        Com::save(&self)
                    }
                }
            }

            _ => Com::none(),
        }
    }

    pub fn view_select(&self) -> Element<Message> {
        match &self.state.loaded {
            Some(Loaded {
                db, kat: Some(kat), ..
            }) if !kat.question_visible => {
                let coo: Vec<Element<Message>> = db
                    .kategorys
                    .iter()
                    .filter(|kategory| compare_kat(&kategory.kat, &kat.kat))
                    .map(|kat| {
                        let mut vec = kat.kat.clone();
                        //println!("{:?}", vec);
                        while vec.last().unwrap() == &0 {
                            vec.pop();
                        }
                        let tex = format!("{} {}", format_kat_kat(&kat.kat), kat.name.clone());
                        //println!("{:?}", vec);
                        row!(
                            button(text(tex))
                                .on_press(SelectM::Select(vec.clone()).into())
                                .width(Length::Fill)
                                .padding(Padding::from([5, 15])),
                            button("C").on_press(SelectM::Open(vec.clone()).into()),
                        )
                        .width(500)
                        .spacing(10)
                        .into()
                    })
                    .collect();
                column!(scrollable(column(coo).padding(20).spacing(10)))
                    .padding(10)
                    .spacing(5)
                    .into()
            }
            Some(Loaded {
                card,
                kat: Some(select),
                ..
            }) if select.question_visible => card.view(
                &select.psafe.question[select.psafe.current],
                self.state.settings.feedback,
            ),
            _ => text("").into(),
        }
    }
}
