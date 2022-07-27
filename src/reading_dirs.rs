use crate::http;
use anyhow::{Context, Result};
use log::trace;
use rocket::serde::Serialize;
use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use urlencoding;

static MOVIE_EXTENSIONS: &'static [&'static str] = &["mkv", "avi"];

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ResultItem {
    name: String,
    full_path: String,
    link: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ReadDirResult {
    pub dirs: Vec<ResultItem>,
    pub movies: Vec<ResultItem>,
}

fn get_urlencoded_path(path_from_root: &Path) -> Option<Vec<String>> {
    let mut res = Vec::new();

    for path_item in path_from_root.into_iter() {
        let item = path_item.to_str()?;
        res.push(String::from(urlencoding::encode(item)));
    }

    Some(res)
}

fn get_dir_link(urlencoded_path_parts: &Vec<String>) -> String {
    let mut res = String::from("/browse");

    for item in urlencoded_path_parts {
        res += "/";
        res += &item;
    }

    res
}

fn get_mpv_link(urlencoded_path_parts: &Vec<String>, host_header: &http::HostHeader) -> String {
    let mut res = String::from("mpv://");
    res += host_header.to_string();
    res += "/files";

    for item in urlencoded_path_parts {
        res += "/";
        res += &item;
    }

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
    urlencoded_path_parts: Vec<String>,
    extension: Option<String>,
}

fn get_extension(entry_pathbuf: &PathBuf) -> Option<String> {
    let ext = entry_pathbuf.extension()?.to_str()?;

    Some(String::from(ext))
}

fn get_path_properties(entry: &DirEntry, root_dir: &PathBuf) -> Option<PathProperties> {
    let entry_pathbuf = entry.path();
    let entry_pathbuf_str = entry_pathbuf.to_str()?;
    let full_path = String::from(entry_pathbuf_str);

    let filename = String::from(entry_pathbuf.file_name()?.to_str()?);

    let file_type = entry.file_type().ok()?;

    let root_dir_with_trailing_slash = String::from(root_dir.to_str()?) + "/";
    let stripped_path = entry_pathbuf_str.strip_prefix(&root_dir_with_trailing_slash)?;

    let urlencoded_path_parts = get_urlencoded_path(&PathBuf::from(stripped_path))?;

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
        urlencoded_path_parts,
        extension,
    })
}

fn put_entry(
    entry: &DirEntry,
    root_dir: &PathBuf,
    host_header: &http::HostHeader,
    result: &mut ReadDirResult,
) -> Result<()> {
    trace!("put_entry {:?}", entry);

    let path_properties = get_path_properties(&entry, &root_dir)
        .with_context(|| format!("gettint path properties of {:?} failed", entry))?;

    trace!("put_entry, path_properties {:?}", path_properties);

    match path_properties.file_type {
        FileTypes::Dir => {
            let link = get_dir_link(&path_properties.urlencoded_path_parts);
            result.dirs.push(ResultItem {
                name: path_properties.filename.clone(),
                full_path: path_properties.full_path.clone(),
                link,
            })
        }
        FileTypes::File => match path_properties.extension {
            Some(ext) => {
                if MOVIE_EXTENSIONS.contains(&ext.as_str()) {
                    let link = get_mpv_link(&path_properties.urlencoded_path_parts, &host_header);

                    result.movies.push(ResultItem {
                        name: path_properties.filename.clone(),
                        full_path: path_properties.full_path.clone(),
                        link,
                    });
                }
            }
            _ => {}
        },
        _ => {}
    }

    Ok(())
}

pub async fn read_dir(
    dir: &PathBuf,
    root_dir: &PathBuf,
    host_header: &http::HostHeader,
) -> Result<ReadDirResult> {
    let mut res = ReadDirResult {
        dirs: Vec::new(),
        movies: Vec::new(),
    };

    let read_dir_res =
        fs::read_dir(&dir).with_context(|| format!("failed to read dir {:?}", &dir))?;

    for entry in read_dir_res {
        match entry {
            Ok(entry) => {
                put_entry(&entry, &root_dir, &host_header, &mut res)
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
