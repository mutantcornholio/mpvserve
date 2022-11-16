use crate::{db, utils};
use rocket::futures::executor::block_on;
use rocket::tokio::fs::File;
use rocket::tokio::io::{AsyncRead, AsyncSeek, ReadBuf};
use rocket::tokio::runtime::Handle;
use rocket::tokio::sync::oneshot;
use rocket::tokio::task;
use rocket_db_pools::Connection;
use sea_orm::ActiveValue::Set;
use sea_orm::*;
use std::{
    io::SeekFrom,
    path::Path,
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
    time::{SystemTime, UNIX_EPOCH},
};

use db::prelude::*;
/*
How does mpvserve display current progress, even though progress is done by mpv on client machine?
It tracks last byte requested by mpv. Not 100% accurate, but should do.

However:
* there's no way in Rocket to say, how much data was transferred during the request
* there's no way to write Rocket fairing that's going to be executed at the end of request
....
soo... this monstrosity
*/

pub struct TrackedFileStream {
    tokio_file: File,
    pub data: TrackedFileStreamData,
    task_trigger: Option<oneshot::Sender<TrackedFileStreamData>>,
}

#[derive(Debug, Clone)]
pub struct TrackedFileStreamData {
    pub path: String,

    pub len: i64,
    pub last_pos: i64,
}
impl TrackedFileStream {
    pub fn from_path(
        abs_path: &PathBuf,
        rel_path: &Path,
        user_id: &str,
        database: Connection<db::Db>,
    ) -> std::io::Result<Self> {
        let urlencoded_path = utils::get_urlencoded_path(rel_path).unwrap();
        let result_path = urlencoded_path + "?" + user_id;
        let handle = Handle::current();
        let _ = handle.enter();
        let file = match block_on(File::open(abs_path)) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        let len = i64::try_from(block_on(file.metadata()).unwrap().len()).unwrap();

        let (task_trigger, trigger_waiter) = oneshot::channel::<TrackedFileStreamData>();

        task::spawn(async move {
            let data = match trigger_waiter.await {
                Ok(data) => data,
                _ => return,
            };

            let now_secs = i64::try_from(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            )
            .unwrap();

            let serving = db::movie_servings::ActiveModel {
                path: Set(data.path.clone()),
                last_timestamp: Set(now_secs),
                last_file_position: Set(data.last_pos),
                file_length: Set(data.len),
            };

            let conn = database.into_inner();

            let insert_error = match serving.insert(&conn).await {
                Ok(_) => return,
                Err(e) => e,
            };
            debug!("insert failed, trying update: {}", insert_error);

            match MovieServing::find_by_id(data.path.clone()).one(&conn).await {
                Ok(serv) => {
                    let serve_movdel: db::movie_servings::Model = serv.unwrap();
                    let mut active_serving: db::movie_servings::ActiveModel = serve_movdel.into();
                    active_serving.last_timestamp = Set(now_secs);
                    active_serving.last_file_position = Set(data.last_pos);

                    if let Err(e) = active_serving.update(&conn).await {
                        log::error!("update failed on insert conflict: {:?}", e);
                    }
                }
                Err(e) => {
                    log::error!("find_by_id failed on insert conflict: {:?}", e);
                }
            };
        });

        let data = TrackedFileStreamData {
            path: result_path,
            len,
            last_pos: 0,
        };

        Ok(Self {
            tokio_file: file,
            task_trigger: Some(task_trigger),
            data,
        })
    }
}

impl Drop for TrackedFileStream {
    fn drop(&mut self) {
        if let Some(task_trigger) = self.task_trigger.take() {
            task_trigger.send(self.data.clone()).unwrap();
        }
    }
}

impl AsyncRead for TrackedFileStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<std::io::Result<()>> {
        let poll = Pin::new(&mut self.tokio_file).poll_read(cx, buf);

        if poll.is_ready() {
            self.data.last_pos += buf.filled().len() as i64;
        }

        poll
    }
}

impl AsyncSeek for TrackedFileStream {
    fn start_seek(mut self: Pin<&mut Self>, position: SeekFrom) -> std::io::Result<()> {
        Pin::new(&mut self.tokio_file).start_seek(position)
    }

    fn poll_complete(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<std::io::Result<u64>> {
        let poll = Pin::new(&mut self.tokio_file).poll_complete(cx);

        if let Poll::Ready(Ok(new_pos)) = poll {
            self.data.last_pos = i64::try_from(new_pos).unwrap();
        }

        poll
    }
}
