use serde::{Deserialize, Serialize};

use crate::dir;
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitRepo {
    pub repo: String,
    pub ssh_priv: Option<String>,
    pub path: String,
}

pub async fn pull(git: GitRepo) -> Result<(), git2::Error> {
    let mut callbacks = git2::RemoteCallbacks::new();

    if let Some(creds) = &git.ssh_priv {
        callbacks.credentials(|_, _, _| git2::Cred::ssh_key_from_memory("git", None, creds, None));
    } else {
        callbacks.certificate_check(|_, _| true);
    }

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let repo = git2::Repository::open(dir().join(git.path))?;

    repo.find_remote("origin")?
        .fetch(&["main"], Some(&mut fo), None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
    let analysis = repo.merge_analysis(&[&fetch_commit])?;
    if analysis.0.is_fast_forward() {
        let refname = "refs/heads/main";
        let mut reference = repo.find_reference(refname)?;
        reference.set_target(fetch_commit.id(), "Fast-Forward")?;
        repo.set_head(refname)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    }
    Ok(())
}

pub async fn clone(git: GitRepo) -> Result<(), String> {
    println!("repo: {:?}", git);
    let mut callbacks = git2::RemoteCallbacks::new();
    if let Some(creds) = &git.ssh_priv {
        callbacks.credentials(|_, _, _| git2::Cred::ssh_key_from_memory("git", None, creds, None));
    } else {
        callbacks.certificate_check(|_, _| true);
    }

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);
    let repo = builder.clone(&git.repo, &dir().join(git.path));
    match repo {
        Ok(_repos) => Ok(()),
        Err(f) => {
            println!("{:?},{:?},{}", f.code(), f.class(), f.message());
            Err(format!("{:?},{:?},{}", f.code(), f.class(), f.message()))
        }
    }
}
pub async fn add(git: GitRepo) -> Result<String, git2::Error> {
    let repo = git2::Repository::open(dir().join(git.path))?;
    let mut index = repo.index()?;
    index.add_all(["."], git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok("".to_string())
}
pub async fn push(git: GitRepo) -> Result<String, git2::Error> {
    let mut callbacks = git2::RemoteCallbacks::new();
    if let Some(creds) = &git.ssh_priv {
        callbacks.credentials(|_, _, _| git2::Cred::ssh_key_from_memory("git", None, creds, None));
    } else {
        callbacks.certificate_check(|_, _| true);
    }

    let mut po = git2::PushOptions::new();
    po.remote_callbacks(callbacks);

    let repo = git2::Repository::open(dir().join(git.path))?;

    let mut remote = repo.find_remote("origin").unwrap();

    //let mut remote = repo.remote("origin", "https://github.com/abc/xyz").unwrap();
    remote.push(&["refs/heads/main"], Some(&mut po)).unwrap();
    Ok("".to_string())
}

pub async fn commit(git: GitRepo, message: &str) -> Result<String, git2::Error> {
    #[cfg(target_os = "android")]
    let author = git2::Signature::now("unirss", "unirss@android.com").unwrap();
    #[cfg(not(target_os = "android"))]
    let author = git2::Signature::now("unirss", "unirss@desktop.com").unwrap();

    let repo = git2::Repository::open(dir().join(git.path))?;
    let mut index = repo.index()?;
    //let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();
    let head = repo.head().unwrap();
    let parent_commit = repo.find_commit(head.target().unwrap()).unwrap();
    let _commit_oid = repo
        .commit(
            Some("HEAD"),
            &author,
            &author,
            message,
            &repo.find_tree(index.write_tree()?)?,
            &[&parent_commit],
        )
        .unwrap();
    // make the commit, since it's the initial commit, there's
    Ok("".to_string())
}
