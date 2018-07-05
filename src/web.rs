use actix_web::{error, fs, App, HttpRequest, HttpResponse, Result, http::Method};
use futures::Stream;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use htmlescape::encode_minimal as escape_html_entity;

use channel;

use std::path::PathBuf;
use std;

pub fn create_app() -> App {
    let s = fs::StaticFiles::new(".").show_files_listing().files_listing_renderer(handle_directory);
    App::new()
        .resource(r"/{tail:.*}.tar", |r| r.method(Method::GET).f(handle_tar))
        .handler("/", s)
}

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

    let stream = channel::run_tar_in_thread(path);
    let resp = HttpResponse::Ok()
        .content_type("application/x-tar")
        .streaming(stream.map_err(|_e| error::ErrorBadRequest("bad request")));
    Ok(resp)
}
