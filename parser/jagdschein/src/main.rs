//use regex::Regex;
//use std::env;
use quizlib::*;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
//use serde::{Deserialize,Serialize};
use std::io::prelude::*;
use substring::Substring;
fn buildup_db(db: &mut Db, path: &PathBuf) {
    let text = fs::read_to_string(path).unwrap();

    let mut temp_text = "".to_string();
    let mut temp_quest = Question {
        awnsers : Awnsers::Multiple(vec![]),
        ..Default::default()
    };
    let mut temp_kat = Kategory {
        ..Default::default()
    };
    let mut kat_index = 1;
    let mut begun = false;
    let mut index = 1;
    let mut temp_a = "".to_string();
    let mut working_on_question = false;
    let mut working_on_awnsers = false;
    for (k, i) in text.lines().enumerate() {
        //println!("text lines:{k}");
        if i.starts_with("Seite 3") && !begun {
            begun = true;
        }
        if begun {
            if working_on_question {
                match i {
                    s if s.is_empty() => {}
                    s if s.starts_with("a)") => {
                        temp_quest.question.text = Some({
                            let trim = format!("{}.", index - 1).len();
                            temp_text
                                .substring(trim, temp_text.len())
                                .trim()
                                .to_string()
                        });
                        temp_quest.id = temp_kat.kat.clone();
                        temp_quest.id.push(kat_index - 1);
                        working_on_awnsers = true;
                        working_on_question = false;
                        temp_text = "".to_string();
                    }
                    _ => {
                        temp_text = format!("{} {}", temp_text, i);
                    }
                }
            }

            if i == format!("{}.", index) || index > 10 && i.starts_with(&format!("{}.", index)) {
                // Cleanup last Question
                // Make new Things
                index += 1;
                kat_index += 1;

                temp_text = i.to_string();

                working_on_question = true;
            } else if !i.is_empty()
                && i.substring(0, 1).parse::<u8>().is_ok()
                && i.substring(1 , 2) == "." 
                && !working_on_awnsers
            {
                // Found Kategory
                println!("{i} - {k}");
                //println!("{}", i[0..1].to_string());
                let (num, name) = i.split_once(" ").unwrap();
                // set Kategory from name
                temp_kat = Kategory {
                    name: name.to_string(),
                    ..Default::default()
                };
                kat_index = 1;
                // Kollekt Kategory number
                for i in num.split(".") {
                    //println!("|{i}|");
                    if !i.is_empty() {
                        temp_kat.kat.push(i.parse::<u16>().unwrap());
                    }
                }
                // Fill with zeros
                for _ in temp_kat.kat.len()..4 {
                    temp_kat.kat.push(0);
                }
                //println!("len{}", temp_kat.kat.len());
                //println!("{}", i);
                //println!("hey");
                db.kategorys.push(temp_kat.clone())
            }
            if working_on_awnsers {
                match i {
                    s if s.starts_with(&format!("{}.", index - 1)) || s.is_empty() => {
                        if let Awnsers::Multiple(vec) = &mut temp_quest.awnsers {
                            vec.push((temp_a.substring(3, temp_a.len()).to_string(), false))
                        }
                        temp_quest.index = db.questions.len();
                        db.questions.push(temp_quest.clone());
                        temp_quest = Question::default();
                        temp_quest.awnsers = Awnsers::Multiple(vec![]);

                        temp_a = "".to_string();
                        working_on_awnsers = false
                    }
                    s if s.starts_with("a)") => {
                        temp_a = i.to_string();
                    }
                    s if s.starts_with("b)")
                        || s.starts_with("c)")
                        || s.starts_with("d)")
                        || s.starts_with("e)")
                        || s.starts_with("f)") =>
                    {
                        if let Awnsers::Multiple(vec) = &mut temp_quest.awnsers {
                            vec.push((temp_a.substring(3, temp_a.len()).to_string(), false))
                        }
                        temp_a = i.to_string();
                    }
                    _ => temp_a = format!("{} {}", temp_a, i),
                }
            }
        }
    }
}

fn add_solutions(db: &mut Db, path: PathBuf,ind:usize) {
    let sol = fs::read_to_string(path).unwrap();
    let debug = false;
    let mut index2 = ind;
    let mut inner_index = 0;
    let mut begun3 = false;

    for (k, i) in sol.lines().enumerate() {
        if debug {
            //println!("solution lines:{k}  -  {index2}  -  {inner_index}")
        }
        match i {
            s if s.starts_with("_ ") => {
                begun3 = true;
                if let Awnsers::Multiple(awns) = &mut db.questions[index2].awnsers {
                    if inner_index >= awns.len() {
                        println!("not Ok");
                    } 

                println!("line: {k} number: {}\n",index2+1-ind);
                println!("\n{}",i);
                println!("{}",awns[inner_index].0);
                println!("true");

                    awns[inner_index].1 = true;
                    inner_index += 1;
                }
            }
            s if s.starts_with("O ") => {
                begun3 = true;
                
                if let Awnsers::Multiple(awns) = &mut db.questions[index2].awnsers {
                    if inner_index >= awns.len() {
                        println!("not Ok");
                    } 

                println!("line: {k} number: {}\n",index2+1-ind);
                println!("\n{}",i);
                println!("{}",awns[inner_index].0);
                println!("false");
                    awns[inner_index].1 = false;
                    inner_index += 1;
                }
            }
            s if s.is_empty() => {
                if begun3 {
                    if debug {
                        println!("{:?}", db.questions[index2].clone());
                        println!(
                            "{:?}",
                            if let Awnsers::Multiple(awn) = &db.questions[index2].awnsers {
                                awn.len()
                            } else {
                                0
                            }
                        );
                    }

                    begun3 = false;

                    if inner_index >= 2 {
                        index2 += 1;
                        inner_index = 0;
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    // Load Files
    let source_directory = "data/text/";
    let solution_directory = "data/sol/";
    let mut text_files: Vec<PathBuf> = fs::read_dir(source_directory)
        .expect("Source Directory not found ()")
        .fold(Vec::new(), |mut acc, x| {
            if x.as_ref().unwrap().path().extension().unwrap() == "txt" {
                acc.push(x.unwrap().path());
            }
            acc
        });
    let mut solutions: Vec<PathBuf> = fs::read_dir(solution_directory)
        .expect("Source Directory not found ()")
        .fold(Vec::new(), |mut acc, x| {
            if x.as_ref().unwrap().path().extension().unwrap() == "txt" {
                acc.push(x.unwrap().path());
            }
            acc
        });
    text_files.sort();
    solutions.sort();
    // Parse Db

    let mut db = Db {
        name: "Jagdschein".to_string(),
        questions: vec![],
        kategorys: vec![],
    };
    for (_num, (path, solut)) in text_files.iter().zip(solutions).enumerate() {
        let ind = db.questions.len();
        let kat_ind = db.kategorys.len();
        buildup_db(&mut db, path);
        println!("{:?}", db);
        println!("{_num}");
        println!("{}", db.questions.len()-ind);
        println!("{}", db.kategorys.len()-kat_ind);
        
        add_solutions(&mut db, solut,ind);
    }

    // check and write Db
    for i in db.questions.iter() {
        println!("{:?}\n", i);
    }
    println!("{}", db.questions.len());
    println!("{}", db.kategorys.len());
    println!("{:?}", db.questions[0]);
    println!("{:?}", db.questions[1]);
    println!("{:?}", db.questions[2]);
    let bin_data = bincode::serialize(&db).unwrap();
    
    let mut file = File::create(dirs::data_local_dir().unwrap().join("uniquiz/modules/jagdschein/db.bin")).unwrap();
    println!("{:?}", file.write_all(&bin_data));
}
