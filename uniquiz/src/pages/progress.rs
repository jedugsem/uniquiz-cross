use crate::{fl, per::Com, Controls, Element, Loaded, Message};
use chrono::{TimeZone, Utc};
use iced::{
    alignment::Horizontal,
    widget::{column, row, text, Space},
    Alignment, Length, Padding, Task,
};
use iced_widget::{button, scrollable};
use quizlib::TSafe;
use quizlib::*;

#[derive(Debug, Clone)]
pub enum ProgM {
    Select(usize),
    Start,
    Back,
    Clear,
}
impl From<ProgM> for Message {
    fn from(m: ProgM) -> Message {
        Message::Prog(m)
    }
}
impl Controls {
    pub fn update_prog(&mut self, m: ProgM) -> Task<Message> {
        if let Some(Loaded { prog, .. }) = &mut self.state.loaded {
            match m {
                ProgM::Select(num) => {
                    *prog = Some(num);
                    Com::save(&self)
                }
                ProgM::Start => Com::perform(&self, async { 0 }, Message::Select),
                ProgM::Back => {
                    if prog.is_some() {
                        *prog = None;
                        Com::save(&self)
                    } else {
                        //self.state.loaded = None;
                        Com::perform(&self, async { 0 }, Message::Select)
                    }
                }
                ProgM::Clear => {
                    if let Some(Loaded {
                        module,
                        stepby: Some(psafe),
                        ..
                    }) = &mut self.state.loaded
                    {
                        let path = module.path.clone();
                        psafe.current = 0;
                        psafe.tsafe = TSafe::default();

                        let git = self.state.settings.prog_git.clone();
                        Com::perform(
                            &self,
                            async {
                                crate::per::write_progress(git, &TSafe::default(), path).await;
                            },
                            |_| Message::Nothing,
                        )
                    } else {
                        Com::none()
                    }
                }
            }
        } else {
            Com::none()
        }
    }
    pub fn view_prog(&self) -> Element<Message> {
        if let Some(Loaded {
            stepby: Some(psafe),
            prog,
            db,
            ..
        }) = &self.state.loaded
        {
            let (header, rows): (Element<Message>, Vec<Element<Message>>) = {
                match prog {
                    None => (
                        row!(
                            text(fl!("question"))
                                .width(Length::Fill)
                                .align_x(Horizontal::Center),
                            text(fl!("kategory"))
                                .width(Length::Fill)
                                .align_x(Horizontal::Center),
                            text(fl!("false"))
                                .width(Length::Fill)
                                .align_x(Horizontal::Center),
                            text(fl!("last-time"))
                                .width(Length::Fill)
                                .align_x(Horizontal::Center),
                        )
                        .into(),
                        psafe
                            .tsafe
                            .sol
                            .iter()
                            .enumerate()
                            .map(|(i, sol)| {
                                button(row!(
                                    text(sol.index)
                                        .width(Length::Shrink)
                                        .align_x(Horizontal::Center),
                                    text(format_kat(&sol.id))
                                        .width(Length::Fill)
                                        .align_x(Horizontal::Center),
                                    text(sol.awm.last().unwrap().1.iter().filter(|x| !*x).count())
                                        .width(Length::Fill)
                                        .align_x(Horizontal::Center),
                                    text(stamp_to_date(sol.awm.last().unwrap().0))
                                        .width(Length::Fill)
                                        .align_x(Horizontal::Center),
                                ))
                                .on_press(ProgM::Select(i).into())
                                .into()
                            })
                            .collect(),
                    ),
                    Some(num) => (
                        row!(
                            text(fl!("last-time"))
                                .width(Length::Fill)
                                .align_x(Horizontal::Center),
                            text(fl!("false"))
                                .width(Length::Fill)
                                .align_x(Horizontal::Center),
                        )
                        .into(),
                        psafe.tsafe.sol[*num]
                            .awm
                            .iter()
                            .map(|awn| {
                                button(row!(
                                    text(stamp_to_date(awn.0))
                                        .width(Length::Fill)
                                        .align_x(Horizontal::Center),
                                    text(awn.1.iter().filter(|x| !*x).count())
                                        .width(Length::Fill)
                                        .align_x(Horizontal::Center),
                                ))
                                .into()
                            })
                            .collect(),
                    ),
                }
            };

            let stats: Element<Message> = column!(
                row!(
                    text(fl!("tried")),
                    Space::with_width(Length::Fill),
                    text!(
                        "{:.1}%",
                        100. * psafe.tsafe.sol.len() as f32 / db.questions.len() as f32
                    )
                ),
                row!(
                    text(fl!("right")),
                    Space::with_width(Length::Fill),
                    text!(
                        "{:.1}%",
                        100.0
                            * psafe
                                .tsafe
                                .sol
                                .iter()
                                .filter(|k| k
                                    .awm
                                    .iter()
                                    .filter(|y| { y.1.iter().filter(|y| !*y).count() == 0 })
                                    .last()
                                    .is_some())
                                .count() as f32
                            / db.questions.len() as f32
                    )
                ),
                row!(
                    text(fl!("right-factor")),
                    Space::with_width(Length::Fill),
                    text!(
                        "{:.1}%",
                        100. * psafe
                            .tsafe
                            .sol
                            .iter()
                            .filter(|k| k
                                .awm
                                .iter()
                                .filter(|y| { y.1.iter().filter(|y| !*y).count() == 0 })
                                .last()
                                .is_some())
                            .count() as f32
                            / psafe.tsafe.sol.iter().count() as f32
                    ),
                ),
            )
            .spacing(10)
            .padding(20)
            .into();

            column!(
                stats,
                header,
                column!(scrollable(
                    column!(
                        column(rows).spacing(5).padding(Padding::from([0, 20])),
                        row!(
                            button(text(fl!("delete"))).on_press(ProgM::Clear.into()),
                            Space::with_width(Length::Fill),
                            button(text(fl!("start"))).on_press(Message::Prog(ProgM::Start)),
                        )
                        .padding(10),
                    )
                    .spacing(10)
                ),)
                .height(Length::Fill),
            )
            .padding(15)
            .height(800)
            .spacing(20)
            .align_x(Alignment::Center)
            .width(500)
            .into()
        } else {
            text(fl!("no-db-found")).into()
        }
    }
}
fn stamp_to_date(stamp: u64) -> String {
    Utc.timestamp_opt(stamp as i64, 0)
        .unwrap()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string()
}
