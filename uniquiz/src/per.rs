use iced_winit::futures::MaybeSend;
use quizlib::*;
use ron::{de::from_reader, ser::PrettyConfig};
use std::path::PathBuf;

use crate::{dir, git, Repos};
use crate::{settings::PSettings, Controls};
use std::{fs::File, io::Read};

pub async fn update(repos: Repos) {
    for i in repos.repos {
        if !dir().join(i.path.clone()).exists() {
            _ = git::clone(i).await;
        }
    }
}

pub struct Com;
impl Com {
    pub fn save(_controls: &Controls) -> iced::Task<crate::Message> {
        // #[cfg(target_os = "android")]
        // {
        //     let uniquiz = controls.state.clone();
        //     //
        //     Com::perform(
        //         &controls,
        //         async move {
        //             let path = PathBuf::from("/storage/emulated/0/git")
        //                 .join("uniquiz")
        //                 .join("write_lock");
        //
        //             File::create(path.clone()).unwrap();
        //             write_uniquiz(uniquiz);
        //             remove_file(path).unwrap();
        //         },
        //         |_| crate::Message::Nothing,
        //     )
        // }
        // #[cfg(not(target_os = "android"))]
        // iced::Task::none()
        iced::Task::none()
    }

    pub fn none() -> iced::Task<crate::Message> {
        iced::Task::none()
    }
    pub fn perform<A: MaybeSend + 'static>(
        _con: &crate::Controls,
        //proxy: &crate::EventLoopProxy<crate::UserEvent>,
        future: impl std::future::Future<Output = A> + 'static + iced_winit::futures::MaybeSend,
        f: impl FnOnce(A) -> crate::Message + 'static + iced_winit::futures::MaybeSend,
    ) -> iced::Task<crate::Message> {
        #[cfg(target_os = "android")]
        {
            use futures::FutureExt;
            let proxy = _con.proxy.clone();
            std::thread::spawn(move || {
                let m = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(future.map(f));
                _ = proxy.send_event(crate::UserEvent::Task(m));
            });
            iced::Task::none()
        }
        #[cfg(not(target_os = "android"))]
        iced::Task::perform(future, f)
    }
}

// pub fn write_uniquiz(uniquiz: crate::Uniquiz) {
//     #[cfg(not(target_os = "android"))]
//     let path = dirs::data_local_dir().unwrap().join("uniquiz");
//     #[cfg(target_os = "android")]
//     let path = PathBuf::from("/storage/emulated/0/git").join("uniquiz");
//     let file = File::create(path.join("uniquiz")).unwrap();
//     let _ = bincode::serialize_into(file, &uniquiz);
//     //println!("Settings writtem");
// }
//
// pub fn load_uniquiz() -> Uniquiz {
//     #[cfg(not(target_os = "android"))]
//     let path = dirs::data_local_dir().unwrap().join("uniquiz");
//     #[cfg(target_os = "android")]
//     let path = PathBuf::from("/storage/emulated/0/git").join("uniquiz");
//
//     match (
//         std::fs::File::open(path.join("uniquiz")),
//         path.join("write_lock").exists(),
//     ) {
//         (Ok(file), false) => bincode::deserialize_from(file).unwrap_or(Uniquiz::default()),
//         _ => Uniquiz::default(),
//     }
// }
//
use crate::GitRepo;
pub fn load_progress(path1: PathBuf) -> Result<TSafe, String> {
    let last = path1.iter().last().unwrap();
    let path = dir().join("progress").join(last);

    match std::fs::File::open(path.join(".process")) {
        Ok(file) => {
            let mut buf_reader = std::io::BufReader::new(file);
            let mut content: Vec<u8> = Vec::new();
            buf_reader
                .read_to_end(&mut content)
                .expect("Coudn't read the File given in the Config");
            match ron::Options::default().from_bytes(&content) {
                Ok(tsafe) => Ok(tsafe),
                Err(_) => Err(String::from("deserializing stats failed")),
            }
        }
        _ => Err("".to_string()),
    }
}
pub fn load_settings() -> Result<crate::settings::PSettings, String> {
    match std::fs::File::open(dir().join("config.ron")) {
        Ok(file) => Ok(from_reader(file).unwrap_or(PSettings::default())),
        _ => Err("".to_string()),
    }
}
pub async fn write_progress(git: Option<GitRepo>, progress: &TSafe, path1: PathBuf) {
    //
    let last = path1.iter().last().unwrap();
    let path = dir().join("progress").join(last);
    if !path.exists() {
        _ = std::fs::create_dir_all(&path);
    }

    let file = File::create(path.join(".process")).unwrap();
    _ = ron::Options::default().to_io_writer_pretty(file, progress, PrettyConfig::default());
    if let Some(git) = git {
        _ = git::add(git.clone()).await;
        _ = git::commit(git.clone(), "progress").await;
        _ = git::push(git).await;
    }
}
pub fn write_settings(settings: crate::settings::PSettings) {
    let mut file = File::create(dir().join("config.ron")).unwrap();
    let _ =
        ron::Options::default().to_io_writer_pretty(&mut file, &settings, PrettyConfig::default());
    //println!("Settings writtem");
}

pub fn get_modules() -> Result<Vec<Modul>, String> {
    let path = vec![dir().join("modules")];
    match read_dirs(path) {
        Ok(t) => Ok(t),
        Err(err) => Result::Err(format!("{:?}", err)),
    }
}

pub fn load_db(path: &PathBuf) -> Result<Db, String> {
    match std::fs::File::open(path.join("db.ron")) {
        Ok(file) => match from_reader(file) {
            Ok(db) => Ok(db),
            Err(err) => Err(err.to_string()),
        },
        Err(err) => Err(err.to_string()),
    }
}

fn read_dirs(pathes: Vec<PathBuf>) -> Result<Vec<Modul>, std::io::Error> {
    let mut mods = vec![];
    for modules in pathes {
        if let Ok(modules) = modules.read_dir() {
            for modul in modules {
                let modul = modul.unwrap();
                let file = std::fs::File::open(modul.path().join("desc.ron"))?;
                mods.push(Modul {
                    path: modul.path(),
                    desc: from_reader(file).unwrap(),
                });
                println!("{:?}", modul.path());
            }
        }
    }
    Ok(mods)
}
