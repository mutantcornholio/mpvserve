mod http;
mod reading_dirs;

#[macro_use]
extern crate rocket;

use clap::Parser;
use log::debug;
use std::format;
use std::fs;
use std::path::{Path, PathBuf};

use rocket::fs::FileServer;
use rocket::response::{content, Redirect};
use rocket::State;
use rocket_dyn_templates::{context, Template};

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

fn render_error_page(err: &dyn std::error::Error, description: &str) -> content::RawHtml<Template> {
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

    let result_path = Path::new(".").join(dir);
    let joined_path = Path::new(&state.root_dir).join(&result_path);
    let full_path = match fs::canonicalize(&joined_path) {
        Ok(path) => path,
        Err(err) => {
            return render_error_page(
                &err,
                &format!("Error while getting full_path of {:?}", &joined_path),
            );
        }
    };

    debug!("Reading directory {:?}", full_path);
    let dir_result =
        reading_dirs::read_dir(&full_path, &PathBuf::from(&state.root_dir), &host_header).await;

    match dir_result {
        Ok(result) => content::RawHtml(Template::render(
            "index",
            context! {result, current_path: result_path.to_str()},
        )),
        Err(err) => render_error_page(
            &err,
            &format!("Error while reading directory {:?}", &full_path),
        ),
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let args = CliArgs::parse();

    let _rocket = rocket::build()
        .mount("/", routes![index, browse])
        .mount("/files", FileServer::from(&args.dir))
        .mount("/public", FileServer::from("./public"))
        .manage(GlobalState { root_dir: args.dir })
        .attach(Template::fairing())
        .launch()
        .await?;

    Ok(())
}
