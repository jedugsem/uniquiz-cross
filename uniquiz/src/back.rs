use crate::pages::katselect::SelectM;
use crate::pages::progress::ProgM;
use crate::pages::search::SearchM;
use crate::pages::stepbystep::StepM;
use crate::pages::test::TestM;
use crate::Message;

pub fn back_message(tab: u8) -> Message {
    match tab {
        0 => Message::Exit,
        1 => StepM::Back.into(),
        2 => ProgM::Back.into(),
        3 => SelectM::Back.into(),
        4 => TestM::Back.into(),
        5 => SearchM::Back.into(),
        _ => Message::Nothing,
    }
}
