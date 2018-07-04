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
    let mut s = String::from(format!("<html>
    <head><title>Index of {index}</title></head>
    <body bgcolor='white'>
    <h1>Index of {index}</h1>
    <hr><a href='../'>../</a>
    <table>\n", index=req.path()));
    let mut paths: Vec<_> = std::fs::read_dir(&dir.path).unwrap()
                                              .filter(|r| dir.is_visible(r))
                                              .filter_map(|r| r.ok())
                                              .collect();
    paths.sort_by_key(|dir| dir.metadata().unwrap().file_type().is_dir());
    for entry in paths {
        let meta = entry.metadata()?;
        let file_url = utf8_percent_encode(&entry.file_name().to_string_lossy(), DEFAULT_ENCODE_SET).to_string();
        // " -- &quot;  & -- &amp;  ' -- &#x27;  < -- &lt;  > -- &gt;
        let file_name = escape_html_entity(&entry.file_name().to_string_lossy());

        let size = meta.len();
        if meta.file_type().is_dir() {
            s.push_str(&format!("<tr><td><a href='{file_url}/'>{file_name}/</a> <small><a href='{file_url}.tar'>(tar)</a></small></td></tr>\n", file_name=file_name, file_url=file_url));
        } else {
            s.push_str(&format!("<tr><td><a href='{file_url}'>{file_name}</a> (size: {size})</td></tr>\n",  file_name=file_name, file_url=file_url, size=size));
        }
    }
    s.push_str("</table><hr></body></html>");
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

fn handle_tar(req: &HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = req.match_info().query("tail")?;
    if !(path.is_dir()) {
        return Err(error::ErrorBadRequest("not a directory"));
    }

    Ok(HttpResponse::Ok().body(format!("fixme: {:?}\n", path)))
}


