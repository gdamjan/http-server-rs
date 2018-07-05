extern crate actix_web;
extern crate actix;
extern crate bytes;
extern crate futures;
extern crate tar;
extern crate htmlescape;
extern crate percent_encoding;

use actix_web::{server, error, fs, App, HttpRequest, HttpResponse, Error, Result, http::Method};
use futures::Stream;

use std::env;
use std::path::PathBuf;
use std::thread;

// TODO cli args
fn main() -> Result<(), Error> {
    let bind_addr = env::var("HTTP_ADDR").unwrap_or(String::from("0.0.0.0:8000"));
    let sys = actix::System::new("static_index");

    server::new(|| {
        let s = fs::StaticFiles::new(".").show_files_listing().files_listing_renderer(handle_directory);
        App::new()
          .resource(r"/{tail:.*}.tar", |r| r.method(Method::GET).f(handle_tar))
          .handler("/", s)
    })
    .bind(&bind_addr)
    .expect(&format!("Can't listen on {} ", bind_addr))
    .start();

    println!("Started http server: {}", bind_addr);
    let _ = sys.run();
    Ok(())
}

use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use htmlescape::encode_minimal as escape_html_entity;

fn handle_directory<'a, 'b>(
    dir: &'a fs::Directory,
    req: &'b HttpRequest,
) -> std::io::Result<HttpResponse> {

    let mut paths: Vec<_> = std::fs::read_dir(&dir.path).unwrap()
                                              .filter(|r| dir.is_visible(r))
                                              .filter_map(|r| r.ok())
                                              .collect();
    paths.sort_by_key(|r| (!r.metadata().unwrap().file_type().is_dir(), r.file_name()));
    let mut t = String::from("<table>
    <tr><td>📁 <a href='../'>../</a></td><td>Size</td></tr>\n");
    for entry in paths {
        let meta = entry.metadata()?;
        let file_url = utf8_percent_encode(&entry.file_name().to_string_lossy(), DEFAULT_ENCODE_SET).to_string();
        let file_name = escape_html_entity(&entry.file_name().to_string_lossy());
        let size = meta.len();

        t.push_str("<tr>");
        if meta.file_type().is_dir() {
            t.push_str(&format!("<td>📂 <a href=\"{file_url}/\">{file_name}/</a></td>", file_name=file_name, file_url=file_url));
            t.push_str(&format!("<td><small>[<a href=\"{file_url}.tar\">.tar</a>]</small></td>\n", file_url=file_url));
        } else {
            t.push_str(&format!("<td>🗎 <a href=\"{file_url}\">{file_name}</a></td>", file_name=file_name, file_url=file_url));
            t.push_str(&format!("<td>{size}</td>", size=size));
        }
        t.push_str("</tr>\n");
    }
    t.push_str("</table>");
    let mut body = String::from(format!("<html>
    <head>
    <title>Index of {index}</title>
    <style>table {{width:100%}} table td:nth-child(2) {{text-align:right}}</style>
    </head>
    <body bgcolor='white'>
    <h1>Index of {index}</h1><hr>\n", index=req.path()));
    body.push_str(t.as_str());
    body.push_str("<hr></body></html>\n");
    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body))
}

fn handle_tar(req: &HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = req.match_info().query("tail")?;
    if !(path.is_dir()) {
        return Err(error::ErrorBadRequest("not a directory"));
    }
    let (writer, stream) = MpscWriter::new();

    thread::spawn(move || {
        let mut a = tar::Builder::new(writer);
        a.mode(tar::HeaderMode::Deterministic);
        a.append_dir_all(path.clone(), path);
        a.finish();
    });

    let resp = HttpResponse::Ok()
        .content_type("application/x-tar")
        .streaming(stream.map_err(|_e| error::ErrorBadRequest("bad request")));
    Ok(resp)
}


// TODO: see about a bounded channel, that work with tokio
struct MpscWriter {
    tx: futures::sync::mpsc::UnboundedSender<bytes::Bytes>
}

impl MpscWriter {
    fn new() -> (Self, futures::sync::mpsc::UnboundedReceiver<bytes::Bytes>) {
        let (tx, rx) = futures::sync::mpsc::unbounded();
        (MpscWriter{tx:tx}, rx)
    }
}

impl std::io::Write for MpscWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.tx.unbounded_send(bytes::Bytes::from(buf));
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
