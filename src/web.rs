use actix_web::{App, middleware, error, HttpServer, HttpRequest, HttpResponse, Responder, web};
use actix_web::dev::ServiceResponse;
use actix_files as fs;
use futures::Stream;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use htmlescape::encode_minimal as escape_html_entity;

use crate::channel;

use std::fmt::Write;
use std::path::PathBuf;


pub fn run(bind_addr: &str, root: &PathBuf) -> std::io::Result<()> {
    let root = root.clone();
    HttpServer::new(move || {
        log::info!("Serving files from {:?}", &root);

        let static_files = fs::Files::new("/", &root)
            .show_files_listing()
            .files_listing_renderer(handle_directory);

        App::new()
            .data(root.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource(r"/{tail:.*}.tar").to(handle_tar))
            .service(web::resource(r"/favicon.ico").to(favicon_ico))
            .service(static_files)
    })
    .bind(bind_addr)?
    .workers(1)
    .run()
}


fn handle_directory<'a, 'b>(
    dir: &'a fs::Directory,
    req: &'b HttpRequest,
) -> Result<ServiceResponse, std::io::Error> {

    let rd = std::fs::read_dir(&dir.path)?;

    fn optimistic_is_dir(entry: &std::fs::DirEntry) -> bool {
        // consider it non directory if metadata reading fails, better than an unwrap() panic
        entry.metadata().map(|m| m.file_type().is_dir()).unwrap_or(false)
    }
    let mut paths : Vec<_> = rd.filter_map(|entry| if dir.is_visible(&entry) { entry.ok() } else {None}).collect();
    paths.sort_by_key(|entry| (!optimistic_is_dir(entry), entry.file_name()));


    let dir_tar_path = String::from(req.path().trim_end_matches('/')) + ".tar";
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

    let resp = HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html);

    Ok(ServiceResponse::new(req.clone(), resp))
}

fn handle_tar(req: HttpRequest) -> impl Responder {
    let root = req.app_data::<PathBuf>().unwrap();
    let tail = req.match_info().query("tail");
    let relpath = PathBuf::from(tail.trim_end_matches('/'));
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

fn favicon_ico() -> impl Responder {
    HttpResponse::Ok()
        .content_type("image/png")
        .header("Cache-Control", "only-if-cached, max-age=86400")
        .body(bytes::Bytes::from_static(include_bytes!("favicon.png")))
}
