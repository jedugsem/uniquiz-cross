use std::{fs::File, io::Read, path::PathBuf};

use quizlib;
use ron::ser::PrettyConfig;
fn main() {
    let file = std::fs::File::open(PathBuf::from(
        "/home/me/programming/iced/uniquizes/uniquiz-wandroid/files/modules/jagdschein/db.bin",
    ))
    .unwrap();
    let mut buf_reader = std::io::BufReader::new(file);
    let mut content: Vec<u8> = Vec::new();
    buf_reader
        .read_to_end(&mut content)
        .expect("Coudn't read the File given in the Config");
    let db: quizlib::Db = bincode::deserialize(&content).unwrap();

    let mut file = File::create(
        "/home/me/programming/iced/uniquizes/uniquiz-wandroid/files/modules/jagdschein/db.ron",
    )
    .unwrap();
    ron::Options::default().to_io_writer_pretty(&mut file, &db, PrettyConfig::default());
    println!("Hello, world!");
}
