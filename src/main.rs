mod http;
mod reading_dirs;

#[macro_use]
extern crate rocket;

use anyhow::anyhow;
use clap::Parser;
use log::debug;
use std::format;
use std::fs;
use std::path::{Path, PathBuf};

use rocket::fs::FileServer;
use rocket::response::{content, Redirect};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use rocket_seek_stream::SeekStream;

/// Web server which creates mpv:// links for movies in the directory
#[derive(Parser, Debug)]
#[clap(about, long_about = None)]
struct CliArgs {
    /// Root directory with the movies
    #[clap(short, long)]
    dir: String,
}

struct GlobalState {
    root_dir: String,
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

#[get("/browse/<dir..>")]
async fn browse(
    dir: PathBuf,
    state: &State<GlobalState>,
    host_header: http::HostHeader,
) -> content::RawHtml<Template> {
    debug!("New request for dir {:?}", dir.to_str());

    match {
        let result_path = Path::new(".").join(dir);
        let joined_path = Path::new(&state.root_dir).join(&result_path);

        match fs::canonicalize(&joined_path) {
            Ok(path) => {
                debug!("Reading directory {:?}", path);
                let root_dir_pathbuf = PathBuf::from(&state.root_dir);
                let dir_result =
                    reading_dirs::read_dir(&path, &root_dir_pathbuf, &host_header).await;

                match dir_result {
                    Ok(result) => Ok(context! {result, current_path: result_path.clone()}),
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(anyhow!(err)),
        }
    } {
        Ok(context) => content::RawHtml(Template::render("index", context)),
        Err(err) => render_error_page(&err, "Error occurred"),
    }
}

#[get("/files/<path..>")]
fn files<'a>(path: PathBuf, state: &State<GlobalState>) -> std::io::Result<SeekStream<'a>> {
    let result_path = Path::new(&state.root_dir).join(&path);

    SeekStream::from_path(result_path)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let args = CliArgs::parse();

    let _rocket = rocket::build()
        .mount("/", routes![index, browse, files])
        .mount("/public", FileServer::from("./public"))
        .manage(GlobalState { root_dir: args.dir })
        .attach(Template::fairing())
        .launch()
        .await?;

    Ok(())
}
