use futures;
use bytes;
use tar;

use std::thread;
use std::path::PathBuf;
use std::io;

pub fn run_tar_in_thread(path: PathBuf) -> futures::sync::mpsc::UnboundedReceiver<bytes::Bytes> {
    let (writer, stream) = MpscWriter::new();

    thread::spawn(move || {
        let mut a = tar::Builder::new(writer);
        a.mode(tar::HeaderMode::Deterministic);
        a.append_dir_all(path.clone(), path);
        a.finish();
    });
    stream
}


/*
 * TODO:
 *
 *   there are 2 features important about futures::sync::mpsc
 *       - it works with tokio (and so with actix), so the stream is async friendly
 *       - it can be sent across threads (more importantly, the tx part)
 *   cons:
 *       futures::sync::mpsc::unbounded() is unbounded, which means the tar thread will
 *       just push everything in memory as fast as it can (as cpu allows).
 *       a better implementation would use a bounded channel, so that the thread would block
 *       if the async core can't send data from the stream fast enough, and wouldn't fill up
 *       GBs of memory. Alas, there doesn't seem to be a bounded channel compatible
 *       with futures at this time (05-07-2018, but pending work on futures 0.3 might help).
 */
struct MpscWriter {
    tx: futures::sync::mpsc::UnboundedSender<bytes::Bytes>
}

impl MpscWriter {
    fn new() -> (Self, futures::sync::mpsc::UnboundedReceiver<bytes::Bytes>) {
        let (tx, rx) = futures::sync::mpsc::unbounded();
        (MpscWriter{tx:tx}, rx)
    }
}

impl io::Write for MpscWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tx.unbounded_send(bytes::Bytes::from(buf));
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
