use actix_web::{error, fs, App, HttpRequest, HttpResponse, Responder, http::Method};
use futures::Stream;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use htmlescape::encode_minimal as escape_html_entity;

use channel;

use std::path::PathBuf;
use std;

pub fn create_app() -> App {
    let s = fs::StaticFiles::new(".").unwrap().show_files_listing().files_listing_renderer(handle_directory);
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
    let mut body = String::new();
    body.push_str(&format!("<h1>Index of {index}</h1><small>[<a href={index}.tar>.tar</a> of whole directory]</small><hr>
    <table>
    <tr><td>📁 <a href='../'>../</a></td><td>Size</td></tr>\n", index=req.path()));
    for entry in paths {
        let meta = entry.metadata()?;
        let file_url = utf8_percent_encode(&entry.file_name().to_string_lossy(), DEFAULT_ENCODE_SET).to_string();
        let file_name = escape_html_entity(&entry.file_name().to_string_lossy());
        let size = meta.len();

        body.push_str("<tr>");
        if meta.file_type().is_dir() {
            body.push_str(&format!("<td>📂 <a href=\"{file_url}/\">{file_name}/</a></td>", file_name=file_name, file_url=file_url));
            body.push_str(&format!("<td><small>[<a href=\"{file_url}.tar\">.tar</a>]</small></td>", file_url=file_url));
        } else {
            body.push_str(&format!("<td>🗎 <a href=\"{file_url}\">{file_name}</a></td>", file_name=file_name, file_url=file_url));
            body.push_str(&format!("<td>{size}</td>", size=size));
        }
        body.push_str("</tr>\n");
    }
    body.push_str("</table><hr>\n");

    let mut html = String::from(format!("<html>
    <head>
    <title>Index of {index}</title>
    <style>h1 {{margin-bottom: 0}} table {{width:100%}} table td:nth-child(2) {{text-align:right}}</style>
    </head>
    <body bgcolor='white'>\n", index=req.path()));
    html.push_str(body.as_str());
    html.push_str("</body></html>\n");

    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html))
}

fn handle_tar(req: &HttpRequest) ->  impl Responder {
    let path: PathBuf = req.match_info().query("tail")?;
    if !(path.is_dir()) {
        return Err(error::ErrorBadRequest("not a directory"));
    }

    let stream = channel::stream_tar_in_thread(path);
    let resp = HttpResponse::Ok()
        .content_type("application/x-tar")
        .streaming(stream.map_err(|_e| error::ErrorBadRequest("stream error")));
    Ok(resp)
}
