use actix_web::{error, fs, App, HttpRequest, HttpResponse, Responder, middleware};
use actix_web::dev::FromParam;
use futures::Stream;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use htmlescape::encode_minimal as escape_html_entity;

use channel;

use std::fmt::Write;
use std::path::PathBuf;
use std;
use bytes;

pub fn create_app(directory: &PathBuf) -> Result<App<PathBuf>, error::Error> {
    let root = directory.to_path_buf(); // practically makes a copy, so it can be used as state
    let static_files = fs::StaticFiles::new(&root)?
                            .show_files_listing()
                            .files_listing_renderer(handle_directory);
    let app = App::with_state(root)
        .middleware(middleware::Logger::new(r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#))
        .resource(r"/{tail:.*}.tar", |r| r.get().f(handle_tar))
        .resource(r"/favicon.ico", |r| r.get().f(favicon_ico))
        .handler("/", static_files);
    Ok(app)
}

fn handle_directory<'a, 'b>(
    dir: &'a fs::Directory,
    req: &'b HttpRequest<PathBuf>,
) -> std::io::Result<HttpResponse> {

    let rd = std::fs::read_dir(&dir.path)?;

    fn optimistic_is_dir(entry: &std::fs::DirEntry) -> bool {
        // consider it non directory if metadata reading fails, better than an unwrap() panic
        entry.metadata().map(|m| m.file_type().is_dir()).unwrap_or(false)
    }
    let mut paths : Vec<_> = rd.filter_map(|entry| if dir.is_visible(&entry) { entry.ok() } else {None}).collect();
    paths.sort_by_key(|entry| (!optimistic_is_dir(entry), entry.file_name()));


    let dir_tar_path = String::from(req.path().trim_right_matches('/')) + ".tar";
    let tar_url = utf8_percent_encode(&dir_tar_path, DEFAULT_ENCODE_SET).to_string();

    let mut body = String::new();
    writeln!(body, "<h1>Index of {}</h1>", req.path()).unwrap();
    writeln!(body, r#"<small>[<a href="{}">.tar</a> of whole directory]</small>"#, tar_url).unwrap();
    writeln!(body, "<table>").unwrap();
    writeln!(body, "<tr><td>📁 <a href='../'>../</a></td><td>Size</td></tr>").unwrap();

    for entry in paths {
        let meta = entry.metadata()?;
        let file_url = utf8_percent_encode(&entry.file_name().to_string_lossy(), DEFAULT_ENCODE_SET).to_string();
        let file_name = escape_html_entity(&entry.file_name().to_string_lossy());
        let size = meta.len();

        write!(body, "<tr>").unwrap();
        if meta.file_type().is_dir() {
            writeln!(body, r#"<td>📂 <a href="{}/">{}/</a></td>"#, file_url, file_name).unwrap();
            write!(body, r#"    <td><small>[<a href="{}.tar">.tar</a>]</small></td>"#, file_url).unwrap();
        } else {
            writeln!(body, r#"<td>🗎 <a href="{}">{}</a></td>"#, file_url, file_name).unwrap();
            write!(body, "    <td>{}</td>", size).unwrap();
        }
        writeln!(body, "</tr>").unwrap();
    }
    writeln!(body, "</table>").unwrap();
    writeln!(body, r#"<footer><a href="{}">{} {}</a></footer>"#,
            env!("CARGO_PKG_HOMEPAGE"), env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).unwrap();

    let mut html = String::new();
    writeln!(html, "<!DOCTYPE html>").unwrap();
    writeln!(html, "<html><head>").unwrap();
    writeln!(html, "<title>Index of {}</title>", req.path()).unwrap();
    writeln!(html, "<style>\n{}</style>", include_str!("style.css")).unwrap();
    writeln!(html, "</head>").unwrap();
    writeln!(html, "<body>\n{}</body>", body).unwrap();
    writeln!(html, "</html>").unwrap();

    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html))
}

fn handle_tar(req: &HttpRequest<PathBuf>) -> impl Responder {
    let root = req.state();
    let tail: String = req.match_info().query("tail")?;
    let relpath = PathBuf::from_param(tail.trim_left_matches('/'))?;
    let fullpath = root.join(&relpath).canonicalize()?;

    if !(fullpath.is_dir()) {
        return Err(error::ErrorBadRequest("not a directory"));
    }

    let stream = channel::stream_tar_in_thread(fullpath);
    let resp = HttpResponse::Ok()
        .content_type("application/x-tar")
        .streaming(stream.map_err(|_e| error::ErrorBadRequest("stream error")));
    Ok(resp)
}

fn favicon_ico(_req: &HttpRequest<PathBuf>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("image/png")
        .header("Cache-Control", "only-if-cached, max-age=86400")
        .body(bytes::Bytes::from_static(include_bytes!("favicon.png")))
}
