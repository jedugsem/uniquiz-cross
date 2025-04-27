use chrono::prelude::*;
use iced::{
    widget::{button, column, container, row, scrollable, text, Space},
    Alignment, Length, Task,
};
use quizlib::*;
use serde::{Deserialize, Serialize};
use substring::Substring;

use crate::{fl, per::Com, Controls, Element, Loaded, Message};

use std::time::Duration;

use tokio::time::sleep;
#[derive(Clone, Debug)]
pub enum TestM {
    Back,
    Review,
    Select(Vec<u16>, bool),
    Init,
    Tick,
    Selector(Vec<u16>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestState {
    pub time: u64,
    pub data: TSafe,
    pub current: usize,
    pub questions_selected: Vec<Question>,
    pub questions: Vec<Question>,
    pub nav: TestEnum,
}

impl Default for TestState {
    fn default() -> Self {
        TestState {
            data: TSafe::default(),
            current: 0,
            time: 120,
            nav: TestEnum::Questions,
            questions: vec![],
            questions_selected: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestEnum {
    Review,
    Select(Vec<u16>),
    Questions,
}

impl From<TestM> for Message {
    fn from(m: TestM) -> Message {
        Message::Test(m)
    }
}
impl Controls {
    pub fn update_test(&mut self, m: TestM) -> Task<Message> {
        match m {
            TestM::Back => match &mut self.state.loaded {
                Some(Loaded { test, .. }) => {
                    if let Some(teststate) = test {
                        match &mut teststate.nav {
                            TestEnum::Review => {
                                teststate.nav = TestEnum::Questions;
                                Com::save(&self)
                            }
                            TestEnum::Questions => {
                                *test = None;
                                Com::perform(&self, async { 0 }, Message::Select)
                            }
                            TestEnum::Select(vec) => {
                                if vec.len() > 0 {
                                    vec.pop();
                                    Com::save(&self)
                                } else {
                                    teststate.nav = TestEnum::Questions;
                                    Com::save(&self)
                                }
                            }
                        }
                    } else {
                        Com::none()
                    }
                }
                _ => Com::none(),
            },
            TestM::Init => {
                match &mut self.state.loaded {
                    Some(loaded) => match &mut loaded.test {
                        Some(teststate) => {
                            loaded
                                .card
                                .prepare_vector(&teststate.questions[teststate.current], true);
                        }
                        None => {
                            let mut state = TestState::default();
                            let quests = get_test(&loaded.db.questions);
                            state.questions = quests.clone();
                            state.questions_selected = quests.clone();

                            loaded.test = Some(state.clone());
                            loaded.card.prepare_vector(&quests[state.current], true);
                        }
                    },
                    None => {}
                }
                Com::perform(&self, async { TestM::Tick }, Message::Test)
            }
            TestM::Review => match &mut self.state.loaded {
                Some(Loaded {
                    test: Some(test), ..
                }) => {
                    test.nav = TestEnum::Review;
                    Com::save(&self)
                }
                _ => Com::none(),
            },
            TestM::Select(vec, choose) => match &mut self.state.loaded {
                Some(Loaded {
                    test: Some(test), ..
                }) => {
                    if choose {
                        test.questions_selected = test
                            .questions
                            .iter()
                            .filter(|s| s.id.starts_with(&vec))
                            .map(|c| c.clone())
                            .collect();
                        test.nav = TestEnum::Questions;
                    } else {
                        test.nav = TestEnum::Select(vec)
                    }

                    Com::save(&self)
                }
                _ => Com::none(),
            },
            TestM::Tick => {
                if let Some(Loaded {
                    test: Some(test), ..
                }) = &mut self.state.loaded
                {
                    if test.time <= 0 {
                        test.nav = TestEnum::Review;
                        Com::save(&self)
                    } else {
                        test.time -= 1;
                        let com = Com::perform(
                            &self,
                            async {
                                sleep(Duration::from_secs(60)).await;
                                TestM::Tick
                            },
                            Message::Test,
                        );
                        Task::batch([com, Com::save(&self)])
                    }
                } else {
                    Com::none()
                }
            }
            _ => Com::none(),
        }
    }
    pub fn view_test(&self) -> Element<Message> {
        match &self.state.loaded {
            Some(Loaded {
                db,
                card,
                test: Some(test),
                ..
            }) => match &test.nav {
                TestEnum::Review => {
                    let mut display = String::new();
                    for i in &test.data.sol {
                        display.push_str(&format!(
                            "{} {} {} {}\n",
                            fl!("question"),
                            i.index + 1,
                            fl!("kategory"),
                            format_kat(&i.id)
                        ));
                        display.push_str("");

                        for i in i.awm.iter() {
                            display.push_str(&format!(
                                "{}:{} ,{}:{}\n",
                                fl!("date"),
                                Utc.timestamp_opt(i.0 as i64, 0).unwrap(),
                                fl!("false"),
                                i.1.iter().filter(|a| !*a).count()
                            ));
                        }
                    }
                    column!(text(fl!("results")).size(20), text!("{}", &display)).into()
                }
                TestEnum::Questions => {
                    let tex = format!("{} {}", test.time, fl!("minutes"),);
                    column!(
                        column!(
                            row!(
                                button(text(fl!("kategories")))
                                    .on_press(TestM::Select(vec![], false).into()),
                                Space::with_width(Length::Fill),
                                button(text(fl!("evaluate"))).on_press(TestM::Review.into()),
                            ),
                            text(tex)
                        )
                        .spacing(5)
                        .padding(10),
                        container(card.view(&test.questions_selected[test.current], false))
                            .height(Length::Fill),
                    )
                    .width(500)
                    .align_x(Alignment::Center)
                    .spacing(5)
                    .into()
                }
                TestEnum::Select(kat2) => {
                    let coo: Vec<Element<Message>> = if kat2.len() > 0 {
                        test.questions
                            .iter()
                            .filter(|kat| kat.id.starts_with(&kat2))
                            .map(|kat| {
                                let mut vec = kat.id.clone();
                                vec.pop();
                                //println!("{:?}", vec);
                                while vec.last().unwrap() == &0 {
                                    vec.pop();
                                }
                                let mut tex = format!("{}", &kat.question.text.as_ref().unwrap());
                                if tex.len() > 45 {
                                    tex = tex.substring(0, 40).to_string();
                                    tex.push_str(" ...")
                                }
                                tex = format!("{} {} {}", fl!("question"), kat.index, tex);

                                let choose: Element<Message> = if !vec.is_empty() {
                                    button("C")
                                        .on_press(TestM::Select(vec.clone(), true).into())
                                        .into()
                                } else {
                                    text("").into()
                                };
                                //println!("{:?}", vec);
                                row!(
                                    button(text(tex))
                                        .on_press(TestM::Select(vec.clone(), true).into())
                                        .width(Length::Fill),
                                    choose,
                                )
                                .spacing(10)
                                .into()
                            })
                            .collect()
                    } else {
                        db.kategorys
                            .iter()
                            .filter(|kat| compare_kat(&kat.kat, &kat2))
                            .map(|kat| {
                                let mut vec = kat.kat.clone();
                                //println!("{:?}", vec);
                                while vec.last().unwrap() == &0 {
                                    vec.pop();
                                }
                                let tex =
                                    format!("{} {}", format_kat_kat(&kat.kat), kat.name.clone());
                                //println!("{:?}", vec);
                                row!(
                                    button(text(tex))
                                        .on_press(TestM::Select(vec.clone(), false).into())
                                        .width(Length::Fill),
                                    button("C").on_press(TestM::Select(vec.clone(), true).into()),
                                )
                                .spacing(10)
                                .into()
                            })
                            .collect()
                    };
                    column!(scrollable(column(coo).spacing(10).padding(10)).width(500))
                        .padding(10)
                        .into()
                }
            },
            _ => column!().into(),
        }
    }
}
