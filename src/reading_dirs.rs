use crate::{db, http, utils};
use anyhow::{Context, Result};
use log::trace;
use rocket::serde::Serialize;
use rocket_db_pools::Connection;
use sea_orm::*;
use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use crate::db::Db;
use db::prelude::*;

static MOVIE_EXTENSIONS: &[&str] = &["mkv", "avi"];

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ResultItem {
    name: String,
    full_path: String,
    rel_path: String,
    id: String, // Just md5 of full_path
    link: String,

    progress: Option<ResultItemProgress>,
}

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ResultItemProgress {
    pub percentage: i64,
    pub timestamp: i64,
}

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ReadDirResult {
    pub dirs: Vec<ResultItem>,
    pub movies: Vec<ResultItem>,
}

fn get_dir_link(urlencoded_path: &str) -> String {
    let mut res = String::from("/browse");

    res += urlencoded_path;

    res
}

fn get_mpv_link(
    urlencoded_path: &str,
    host_header: &http::HostHeader,
    user_id: &http::UserId,
) -> String {
    let mut res = String::from("mpv://");
    res += host_header.to_string();
    res += "/files";

    res += urlencoded_path;

    res += "?user_id=";
    res += user_id.as_str();

    res
}

#[derive(Debug)]
enum FileTypes {
    File,
    Dir,
    Other,
}

#[derive(Debug)]
struct PathProperties {
    filename: String,
    file_type: FileTypes,
    full_path: String,
    rel_path: String,
    urlencoded_path: String,
    extension: Option<String>,
}

fn get_extension(entry_path: &Path) -> Option<String> {
    let ext = entry_path.extension()?.to_str()?;

    Some(String::from(ext))
}

fn get_path_properties(entry: &DirEntry, root_dir: &Path) -> Option<PathProperties> {
    let entry_pathbuf = entry.path();
    let entry_pathbuf_str = entry_pathbuf.to_str()?;
    let full_path = String::from(entry_pathbuf_str);

    let filename = String::from(entry_pathbuf.file_name()?.to_str()?);

    let file_type = entry.file_type().ok()?;

    let root_dir_with_trailing_slash = String::from(root_dir.to_str()?) + "/";
    let stripped_path = entry_pathbuf_str.strip_prefix(&root_dir_with_trailing_slash)?;

    let urlencoded_path = utils::get_urlencoded_path(&PathBuf::from(stripped_path))?;

    // Unlike everything else, not getting an extension is expected
    let extension = get_extension(&entry_pathbuf);

    let file_type: FileTypes = {
        if file_type.is_file() {
            FileTypes::File
        } else if file_type.is_dir() {
            FileTypes::Dir
        } else {
            FileTypes::Other
        }
    };

    Some(PathProperties {
        filename,
        file_type,
        full_path,
        rel_path: String::from(stripped_path),
        urlencoded_path,
        extension,
    })
}

async fn put_entry(
    entry: &DirEntry,
    root_dir: &Path,
    host_header: &http::HostHeader,
    result: &mut ReadDirResult,
    user_id: &http::UserId,
    conn: &DatabaseConnection,
) -> Result<()> {
    trace!("put_entry {:?}", entry);

    let path_properties = get_path_properties(entry, root_dir)
        .with_context(|| format!("gettint path properties of {:?} failed", entry))?;

    trace!("put_entry, path_properties {:?}", path_properties);

    let entry_hash = md5::compute(path_properties.full_path.as_bytes());
    let entry_hash = format!("{:x}", entry_hash);

    match path_properties.file_type {
        FileTypes::Dir => {
            let link = get_dir_link(&path_properties.urlencoded_path);
            result.dirs.push(ResultItem {
                name: path_properties.filename.clone(),
                full_path: path_properties.full_path.clone(),
                rel_path: path_properties.rel_path.clone(),
                id: entry_hash,
                link,
                progress: None,
            })
        }
        FileTypes::File => {
            if let Some(ext) = path_properties.extension {
                if MOVIE_EXTENSIONS.contains(&ext.as_str()) {
                    let link = get_mpv_link(&path_properties.urlencoded_path, host_header, user_id);

                    let progress =
                        get_item_progress(&path_properties.urlencoded_path, user_id, conn).await;

                    result.movies.push(ResultItem {
                        name: path_properties.filename.clone(),
                        full_path: path_properties.full_path.clone(),
                        rel_path: path_properties.rel_path.clone(),
                        id: entry_hash,
                        link,
                        progress,
                    });
                }
            }
        }
        _ => {}
    }

    Ok(())
}

async fn get_item_progress(
    urlencoded_path: &str,
    user_id: &http::UserId,
    conn: &DatabaseConnection,
) -> Option<ResultItemProgress> {
    let path = String::from(urlencoded_path) + "?" + user_id.to_string();
    match MovieServing::find_by_id(path.clone()).one(conn).await {
        Ok(Some(serve_model)) => Some(ResultItemProgress {
            percentage: serve_model.last_file_position * 100 / serve_model.file_length,
            timestamp: serve_model.last_timestamp,
        }),
        Ok(None) => {
            log::debug!("No progress found for {}", &path);
            None
        }
        Err(e) => {
            log::debug!("No progress found for {}, {:?}", &path, e);
            None
        }
    }
}

pub async fn read_dir(
    dir: &PathBuf,
    root_dir: &Path,
    host_header: &http::HostHeader,
    user_id: &http::UserId,
    conn: &Connection<Db>,
) -> Result<ReadDirResult> {
    let mut res = ReadDirResult {
        dirs: Vec::new(),
        movies: Vec::new(),
    };

    let read_dir_res =
        fs::read_dir(dir).with_context(|| format!("failed to read dir {:?}", &dir))?;

    for entry in read_dir_res {
        match entry {
            Ok(entry) => {
                put_entry(&entry, root_dir, host_header, &mut res, user_id, conn)
                    .await
                    .with_context(|| format!("failed to process entry {:?}", &entry))?;
            }
            Err(err) => {
                println!("Error while iterating over directory: {}", err);
            }
        }
    }

    let comparator = |a: &ResultItem, b: &ResultItem| a.name.to_string().cmp(&b.name.to_string());

    res.dirs.sort_by(comparator);
    res.movies.sort_by(comparator);

    Ok(res)
}
