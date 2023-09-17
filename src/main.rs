mod db;
mod http;
mod reading_dirs;
mod tracked_file_stream;

#[macro_use]
extern crate rocket;

use anyhow::{anyhow, Result};
use clap::Parser;
use log::debug;
use std::format;
use std::fs;
use std::path::{Path, PathBuf};

use crate::tracked_file_stream::TrackedFileStream;
use rocket::{
    fairing,
    fairing::AdHoc,
    fs::FileServer,
    response::{content, Redirect},
    serde::json::Json,
    serde::Serialize,
    Build, Rocket, State,
};
use rocket_db_pools::{Connection, Database};
use rocket_dyn_templates::{context, Template};
use rocket_seek_stream::SeekStream;

use crate::reading_dirs::ReadDirResult;
use migration::MigratorTrait;

/// Web server which creates mpv:// links for movies in the directory
#[derive(Parser, Debug)]
#[clap(about, long_about = None)]
struct CliArgs {
    /// Root directory with the movies
    #[clap(long)]
    dir: String,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub struct Config {
    db_url: String,
}

struct GlobalState {
    root_dir: String,
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &db::Db::fetch(&rocket).unwrap().conn;
    match migration::Migrator::up(conn, None).await {
        Ok(_) => Ok(rocket),
        Err(e) => {
            log::error!("Migration failed {:?}", e);
            Err(rocket)
        }
    }
}

fn render_error_page(err: &anyhow::Error, description: &str) -> content::RawHtml<Template> {
    content::RawHtml(Template::render(
        "error",
        context! {err: &err.to_string(), description},
    ))
}

#[get("/")]
async fn index() -> Redirect {
    Redirect::to(uri!(browse(dir = "")))
}

async fn dir_request(
    dir: &PathBuf,
    root_dir: &str,
    host_header: &http::HostHeader,
    user_id: &http::UserId,
    database: &Connection<db::Db>,
) -> Result<ReadDirResult> {
    let conn = &*database;
    let result_path = Path::new(".").join(dir);
    let joined_path = Path::new(&root_dir).join(&result_path);

    match fs::canonicalize(&joined_path) {
        Ok(path) => {
            debug!("Reading directory {:?}", path);
            let root_dir_pathbuf = PathBuf::from(&root_dir);

            reading_dirs::read_dir(&path, &root_dir_pathbuf, &host_header, &user_id, conn).await
        }
        Err(err) => Err(anyhow!(err)),
    }
}

#[get("/browse/<dir..>")]
async fn browse(
    dir: PathBuf,
    state: &State<GlobalState>,
    host_header: http::HostHeader,
    user_id: http::UserId,
    database: Connection<db::Db>,
) -> content::RawHtml<Template> {
    debug!("New request for dir {:?}", dir.to_str());

    match dir_request(&dir, &state.root_dir, &host_header, &user_id, &database).await {
        Ok(result) => {
            let context = context! {result, current_path: dir.clone(), user_id};
            content::RawHtml(Template::render("index", context))
        }
        Err(err) => render_error_page(&err, "Error occurred"),
    }
}

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub enum ApiBrowseResult {
    Error(JsonError),
    Result(ReadDirResult),
}

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct JsonError {
    message: String,
}

#[get("/api/browse/<dir..>")]
async fn api_browse(
    dir: PathBuf,
    state: &State<GlobalState>,
    host_header: http::HostHeader,
    user_id: http::UserId,
    database: Connection<db::Db>,
) -> Json<ApiBrowseResult> {
    debug!("New API request for dir {:?}", dir.to_str());
    match dir_request(&dir, &state.root_dir, &host_header, &user_id, &database).await {
        Ok(result) => Json(ApiBrowseResult::Result(result)),
        Err(err) => Json(ApiBrowseResult::Error(JsonError {
            message: err.to_string(),
        })),
    }
}

#[get("/files/<path..>?<user_id>")]
async fn files<'a>(
    database: Connection<db::Db>,
    path: PathBuf,
    user_id: Option<String>,
    state: &State<GlobalState>,
) -> std::io::Result<SeekStream<'a>> {
    let result_path = Path::new(&state.root_dir).join(&path);

    let user_id = match user_id {
        None => String::from("MISSING_USER_ID"),
        Some(val) => val,
    };

    let tracked_file_stream =
        TrackedFileStream::from_path(&result_path, &path, &user_id, database)?;
    let len = tracked_file_stream.data.len;

    Ok(SeekStream::with_opts(
        tracked_file_stream,
        u64::try_from(len).unwrap(),
        None,
    ))
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let args = CliArgs::parse();
    let _rocket = rocket::build()
        .mount("/", routes![index, browse, api_browse, files])
        .mount("/public", FileServer::from("./public"))
        .manage(GlobalState { root_dir: args.dir })
        .attach(Template::fairing())
        .attach(db::Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .launch()
        .await?;

    Ok(())
}
