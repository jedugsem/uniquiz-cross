use crate::{comps::card::*, fl, per::Com, Controls, Element, Loaded, Message};
use iced::{widget::text, Task};
use quizlib::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone)]
pub enum StepM {
    Back,
    Init,
    LoadedQuestions(Vec<Question>),
    Select(Vec<u16>),
}
impl From<StepM> for Message {
    fn from(m: StepM) -> Message {
        Message::Step(m)
    }
}
impl Controls {
    pub fn update_step(&mut self, m: StepM) -> Task<Message> {
        if let Some(Loaded {
            db,
            card,
            stepby: Some(psafe),
            ..
        }) = &mut self.state.loaded
        {
            match m {
                StepM::Back => Com::perform(&self, async { 0 }, Message::Select),
                StepM::LoadedQuestions(_) => Com::none(),
                StepM::Init => {
                    println!("init");
                    psafe.question = Db::get_questions(db, &psafe.tsafe);
                    psafe.current = 0;
                    card.prepare_vector(&psafe.question[psafe.current], true);
                    Com::save(&self)
                }
                StepM::Select(vec) => {
                    println!("select");
                    psafe.question = db.get_kategory(&vec);
                    card.prepare_vector(&psafe.question[psafe.current], true);
                    Com::perform(&self, async { 1 }, Message::Select)
                }
            }
        } else {
            Com::none()
        }
    }

    pub fn view_step(&self) -> Element<Message> {
        if let Some(Loaded {
            card,
            stepby: Some(psafe),
            ..
        }) = &self.state.loaded
        {
            if psafe.current < psafe.question.len() {
                card.view(&psafe.question[psafe.current], self.state.settings.feedback)
            } else {
                text(fl!("no-questions")).into()
            }
        } else {
            if let Some(_loaded) = &self.state.loaded {
                text(fl!("loaded")).into()
            } else {
                text(fl!("database-not-loaded")).into()
            }
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PSafe {
    pub question_visible: bool,
    pub kat_2: Vec<u16>,

    pub tsafe: TSafe,
    pub kat: Vec<u16>,
    pub question: Vec<Question>,
    pub card: Questi,
    pub current: usize,
}
