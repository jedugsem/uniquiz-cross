use rand::prelude::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default, Clone, PartialEq, Deserialize, Debug, Serialize, Eq)]
pub struct Frage {
    pub text: Option<String>,
    pub extras: FragenExtra,
}
#[derive(Default, Clone, PartialEq, Deserialize, Debug, Serialize, Eq)]
pub enum FragenExtra {
    #[default]
    None,
    Video(PathBuf),
    Image(PathBuf),
    Audio(PathBuf),
}
#[derive(Default, Clone, PartialEq, Deserialize, Debug, Serialize, Eq)]
pub enum Awnsers {
    #[default]
    None,
    Free(String),
    Multiple(Vec<(String, bool)>),
}
#[derive(Default, Clone, PartialEq, Deserialize, Debug, Serialize, Eq)]
pub struct Kategory {
    pub kat: Vec<u16>,
    pub name: String,
}

#[derive(Default, Clone, PartialEq, Deserialize, Debug, Serialize, Eq)]
pub struct Question {
    pub question: Frage,
    pub awnsers: Awnsers,
    pub id: Vec<u16>,
    pub index: usize,
}

#[derive(Default, Clone, PartialEq, Deserialize, Debug, Serialize, Eq)]
pub struct Db {
    pub name: String,
    pub questions: Vec<Question>,
    pub kategorys: Vec<Kategory>,
}
pub fn get_viedeo(_path: PathBuf) -> Vec<u8> {
    vec![]
}
pub fn get_audio(_path: PathBuf) -> Vec<u8> {
    vec![]
}
pub fn get_image(_path: PathBuf) -> Vec<u8> {
    vec![]
}
pub fn format_kat(vec: &Vec<u16>) -> String {
    let mut vec2 = vec.clone();
    vec2.pop();
    let mut st = vec2
        .into_iter()
        .filter(|t| if t > &0 { true } else { false })
        .map(|t| format!("{}.", t))
        .collect::<String>();
    st.push_str(&format!("{}", vec.last().unwrap()));
    st
}
pub fn format_kat_kat(vec: &Vec<u16>) -> String {
    vec.clone()
        .into_iter()
        .filter(|t| if t > &0 { true } else { false })
        .map(|t| format!("{}.", t))
        .collect::<String>()
}
pub fn compare_kat(kat: &Vec<u16>, test: &Vec<u16>) -> bool {
    if kat.len() >= (test.len() + 2) {
        kat.starts_with(test) && kat[test.len() + 1] == 0 && kat[test.len()] != 0
    } else if kat.len() >= (test.len() + 1) {
        kat.starts_with(test) && kat[test.len()] != 0
    } else {
        false
    }
}
pub fn compare_results(result: &Vec<bool>, sol: &Vec<(String, bool)>) -> Vec<bool> {
    result
        .iter()
        .zip(sol.iter())
        .map(|(res, (_, sol))| res == sol)
        .collect()
}

#[derive(Default, Clone, PartialEq, Deserialize, Debug, Serialize, Eq)]
pub struct Modul {
    pub desc: Modul2,
    pub path: PathBuf,
}

#[derive(Default, Clone, PartialEq, Deserialize, Debug, Serialize, Eq)]
pub struct Modul2 {
    pub name: String,
    pub version: String,
    pub count: u32,
    pub time: u32,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct TSafe {
    pub sol: Vec<Sol>,
    pub current: usize,
}
impl TSafe {
    pub fn add_solution(&mut self, question: &Question, is_checked: &Vec<bool>) {
        if let Some(index) = self.sol.iter().position(|sol| sol.index == question.index) {
            self.sol[index].awm.push((
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
                if let Awnsers::Multiple(vec) = &question.awnsers {
                    compare_results(is_checked, vec)
                } else {
                    vec![]
                },
            ));
        } else {
            self.sol.push(Sol {
                index: question.index.clone(),
                id: question.id.clone(),
                awm: vec![(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs(),
                    if let Awnsers::Multiple(vec) = &question.awnsers {
                        compare_results(is_checked, vec)
                    } else {
                        vec![]
                    },
                )],
            });
            self.current = question.index.clone() + 1;
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Sol {
    pub id: Vec<u16>,
    pub index: usize,
    pub awm: Vec<(u64, Vec<bool>)>,
}

fn _get_false_questions(safe: &TSafe, elf: &Db) -> Vec<Question> {
    let _temps: Vec<&Question> = elf.questions[..safe.current]
        .iter()
        .filter(move |x| {
            (safe
                .sol
                .iter()
                .find(|y| y.index == x.index)
                .unwrap()
                .awm
                .iter()
                .map(|c| c.1.clone())
                .flatten()
                .count())
                > 2
        })
        .choose_multiple(&mut rand::thread_rng(), 20);
    let temps: Vec<&Question> = elf.questions[..safe.current]
        .iter()
        .filter(move |x| x.id.is_empty())
        .choose_multiple(&mut rand::thread_rng(), 10);

    temps.iter().copied().map(|a| a.clone()).collect()
}
pub fn get_test(se: &Vec<Question>) -> Vec<Question> {
    let mut temp_vec: Vec<&Question> = vec![];
    let mut kats = se
        .iter()
        .map(|kat| kat.id.first().unwrap())
        .collect::<Vec<&u16>>();
    kats.dedup();
    for i in kats {
        let temps: Vec<&Question> = se
            .iter()
            .filter(move |x| x.id.first().unwrap() == i)
            .choose_multiple(&mut rand::thread_rng(), 20);
        temp_vec.extend(temps);
    }
    temp_vec.iter().copied().map(|m| m.clone()).collect()
}
impl Db {
    pub fn get_questions(&self, safe: &TSafe) -> Vec<Question> {
        let vec = self.questions[(safe.current)..].to_vec();
        //vec.extend(get_false_questions(&safe,self   ));
        //(
        vec //,self.questions.len()-safe.current)
    }

    pub fn get_kategory(&self, kat: &Vec<u16>) -> Vec<Question> {
        self.questions
            .iter()
            .filter(|ele| {
                if ele.id.starts_with(&kat) {
                    //println!("{:?}",ele);
                    true
                } else {
                    false
                }
            })
            .map(|ele| ele.clone())
            .collect()
    }
}
