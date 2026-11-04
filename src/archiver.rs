use std::path::PathBuf;

use tokio_util::io::ReaderStream;

/*
 * TODO:
 *   don't tar hidden files
 */

pub fn stream_tar(path: PathBuf) -> ReaderStream<tokio::io::DuplexStream> {
    let (writer, reader) = tokio::io::duplex(64 * 1024);

    actix_web::rt::spawn(async move {
        let mut a = async_tar::Builder::new(writer);
        let last_path_component = path.file_name().unwrap();
        a.mode(async_tar::HeaderMode::Deterministic);
        a.append_dir_all(last_path_component, &path)
            .await
            .unwrap_or_else(|e| log::error!("{}", e));
        // always finish, the tokio Builder panics if dropped unfinished
        a.finish().await.unwrap_or_else(|e| log::error!("{}", e));
    });
    ReaderStream::new(reader)
}
