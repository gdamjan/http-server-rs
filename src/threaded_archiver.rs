use bytes;
use futures;
use futures::Sink;
use tar;

use std::io;
use std::path::PathBuf;
use std::thread;

/*
 * TODO:
 *   don't tar hidden files
 */

type Stream = futures::sync::mpsc::Receiver<bytes::Bytes>;
type Sender = futures::sync::mpsc::Sender<bytes::Bytes>;
type BlockingSender = futures::sink::Wait<Sender>;

pub fn stream_tar_in_thread(path: PathBuf) -> Stream {
    let (writer, stream) = StreamWriter::new(64);

    thread::spawn(move || {
        let mut a = tar::Builder::new(writer);
        let last_path_component = path.file_name().unwrap();
        a.mode(tar::HeaderMode::Deterministic);
        a.append_dir_all(last_path_component, &path)
            .unwrap_or_else(|e| println!("{}", e));
        a.finish().unwrap_or_else(|e| println!("{}", e));
    });
    stream
}

struct StreamWriter {
    tx: BlockingSender,
}

impl StreamWriter {
    fn new(size: usize) -> (Self, Stream) {
        let (tx, rx) = futures::sync::mpsc::channel(size);
        let tx = tx.wait();
        (StreamWriter { tx }, rx)
    }
}

impl io::Write for StreamWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tx
            .send(bytes::Bytes::from(buf))
            .map(|_| buf.len())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tx
            .flush()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
