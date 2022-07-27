use crate::http;
use log::{trace, warn};
use rocket::serde::{Deserialize, Serialize};
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::{fs, io};
use urlencoding;

static MOVIE_EXTENSIONS: &'static [&'static str] = &["mkv", "avi"];

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Dir {
    name: String,
    full_path: String,
    link: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Movie {
    name: String,
    full_path: String,
    link: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ReadDirResult {
    pub dirs: Vec<Dir>,
    pub movies: Vec<Movie>,
}

fn get_urlencoded_path(path_from_root: &Path) -> Option<Vec<String>> {
    let mut res = Vec::new();

    for path_item in path_from_root.into_iter() {
        match path_item.to_str() {
            Some(item) => {
                res.push(String::from(urlencoding::encode(item)));
            }
            None => return None,
        }
    }

    Some(res)
}

fn get_dir_link(path_from_root: &Path) -> Option<String> {
    match get_urlencoded_path(&path_from_root) {
        Some(encoded_path) => {
            let mut res = String::from("/browse");

            for item in encoded_path {
                res += "/";
                res += &item;
            }

            Some(res)
        }
        None => None,
    }
}

fn get_mpv_link(path_from_root: &Path, host_header: &http::HostHeader) -> Option<String> {
    match get_urlencoded_path(&path_from_root) {
        Some(encoded_path) => {
            let mut res = String::from("mpv://");
            res += host_header.to_string();
            res += "/files";

            for item in encoded_path {
                res += "/";
                res += &item;
            }

            Some(res)
        }
        None => None,
    }
}

fn put_entry(
    entry: &DirEntry,
    root_dir: &PathBuf,
    host_header: &http::HostHeader,
    result: &mut ReadDirResult,
) {
    trace!("put_entry {:?}", entry);
    let entry_path = entry.path();

    let entry_path_str = match entry_path.to_str() {
        Some(path) => path,
        None => return,
    };
    trace!("put_entry, entry_path_str: {}", entry_path_str);

    let entry_filename = match entry_path.file_name() {
        Some(path) => match path.to_str() {
            Some(path) => path,
            None => return,
        },
        None => return,
    };
    trace!("put_entry, entry_filename: {}", entry_filename);

    let file_type = match entry.file_type() {
        Ok(file_type) => file_type,
        Err(err) => {
            warn!(
                "Failed to get file_type of entry {}: {:?}",
                entry_path_str, err
            );
            return;
        }
    };

    let path_from_root = match entry_path.strip_prefix(&root_dir) {
        Ok(path) => path,
        Err(err) => {
            warn!(
                "Failed to get path_from_root of entry {}: {:?}",
                entry_path_str, err
            );
            return;
        }
    };

    if file_type.is_dir() {
        match get_dir_link(&path_from_root) {
            Some(link) => {result.dirs.push(Dir {
                name: String::from(entry_filename),
                full_path: String::from(entry_path_str),
                link,
            })},
            None => {}
        }

    } else if file_type.is_file() {
        match entry.path().extension() {
            Some(ext) => {
                for &movie_ext in MOVIE_EXTENSIONS {
                    if ext.eq(movie_ext) {
                        match get_mpv_link(&path_from_root, &host_header) {
                            Some(link) => result.movies.push(Movie {
                                name: String::from(entry_filename),
                                full_path: String::from(entry_path_str),
                                link,
                            }),
                            None => {}
                        }
                    }
                }
            }
            None => {}
        };
    }
}

pub async fn read_dir(
    dir: &PathBuf,
    root_dir: &PathBuf,
    host_header: &http::HostHeader,
) -> io::Result<ReadDirResult> {
    let mut res = ReadDirResult {
        dirs: Vec::new(),
        movies: Vec::new(),
    };

    match fs::read_dir(&dir) {
        Ok(result) => {
            for entry in result {
                match entry {
                    Ok(entry) => put_entry(&entry, &root_dir, &host_header, &mut res),
                    Err(err) => println!("Error while iterating over directory: {}", err),
                }
            }
        }
        Err(err) => {
            return Err(err);
        }
    }

    Ok(res)
}
