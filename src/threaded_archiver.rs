use futures::prelude::*;

use std::io;
use std::path::Path;
use std::thread;

/*
 * TODO:
 *   don't tar hidden files
 */

type Stream = futures::channel::mpsc::Receiver<bytes::Bytes>;
type Sender = futures::channel::mpsc::Sender<bytes::Bytes>;

pub fn stream_tar_in_thread<P>(path: P) -> Stream
where
    P: AsRef<Path> + Send + 'static,
{
    let (writer, stream) = StreamWriter::new(64);

    thread::spawn(move || {
        let mut a = tar::Builder::new(writer);
        let last_path_component = path.as_ref().file_name().unwrap();
        a.mode(tar::HeaderMode::Deterministic);
        a.append_dir_all(last_path_component, &path)
            .unwrap_or_else(|e| println!("{}", e));
        a.finish().unwrap_or_else(|e| println!("{}", e));
    });
    stream
}

struct StreamWriter {
    tx: Sender,
}

impl StreamWriter {
    fn new(size: usize) -> (Self, Stream) {
        let (tx, rx) = futures::channel::mpsc::channel(size);
        (StreamWriter { tx }, rx)
    }
}

impl io::Write for StreamWriter {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let len = data.len();
        futures::executor::block_on(async move {
            let buf = bytes::Bytes::copy_from_slice(data);
            self.tx.send(buf).await.ok(); // maybe propagate any errors back
        });
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        futures::executor::block_on(async move { self.tx.flush().await.ok() });
        Ok(())
    }
}
