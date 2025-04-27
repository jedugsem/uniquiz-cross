use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
#[derive(Serialize, Deserialize)]
pub struct Lang {
    name: String,
    words: HashMap<String, String>,
}
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        crate::localize::fl($message_id)
    }};
}
use serde::{Deserialize, Serialize};

use crate::settings::Language;
pub static LANG: LazyLock<Mutex<Language>> = LazyLock::new(|| Mutex::new(Language::System));
pub static LANGUAGES: LazyLock<Mutex<Vec<Lang>>> = LazyLock::new(|| Mutex::new(vec![]));

pub fn fl(base: &str) -> String {
    let lang = match LANG.lock().unwrap().to_owned() {
        Language::System => {
            let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"));
            if locale == "C" {
                "en".to_string()
            } else {
                locale[..2].to_string()
            }
        }
        Language::De => "de".to_string(),
        Language::En => "en".to_string(),
    };

    let mutex = LANGUAGES.lock().unwrap();
    let language: &Lang = (*mutex).iter().filter(|y| y.name == lang).last().unwrap();
    match language.words.get(base) {
        Some(s) => s.to_string(),
        None => {
            println!("{} missing in tranlation in {}", base, lang);
            "missing".to_string()
        }
    }
}
