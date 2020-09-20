use actix_web::{get, error, middleware, web, App, Error, HttpServer, HttpRequest, HttpResponse, Responder};
use actix_web::http::StatusCode;
use actix_files::{Files, NamedFile};
use futures::StreamExt;

use std::path::PathBuf;


pub async fn run(bind_addr: &str, root: &PathBuf) -> std::io::Result<()> {
    let root_ = root.clone();
    let s = HttpServer::new(move || {

        let static_files = Files::new("/", &root_)
            .show_files_listing()
            .redirect_to_slash_directory()
            .files_listing_renderer(crate::directory_listing::directory_listing);

        App::new()
            .data(root_.clone())
            .wrap(middleware::Logger::default())
            .service(favicon_ico)
            .service(handle_tar)
            .service(static_files)
    })
    .bind(bind_addr)?
    .run();

    log::info!("Serving files from {:?}", &root);
    s.await
}

#[get("/{tail:.*}.tar")]
async fn handle_tar(req: HttpRequest, root: web::Data<PathBuf>, web::Path(tail): web::Path<String>) -> impl Responder {
    let relpath = PathBuf::from(tail.trim_end_matches('/'));
    let fullpath = root.join(&relpath).canonicalize()
        .map_err(|err| error::InternalError::new(err, StatusCode::INTERNAL_SERVER_ERROR))?;

    if fullpath.is_file() {
        return NamedFile::open(fullpath)
            .map_err(|err| error::InternalError::new(err, StatusCode::INTERNAL_SERVER_ERROR))?
            .into_response(&req);
    }

    if !(fullpath.is_dir()) {
        return Ok(HttpResponse::NotFound().body("Directory not found"));
    }

    let stream = crate::threaded_archiver::stream_tar_in_thread(fullpath)
        .map(Ok::<_, Error>);
    let response = HttpResponse::Ok()
        .content_type("application/x-tar")
        .streaming(stream);

    Ok(response)
}

const FAVICON_ICO: &[u8] = include_bytes!("favicon.png");

#[get("/favicon.ico")]
async fn favicon_ico() -> impl Responder {
    HttpResponse::Ok()
        .content_type("image/png")
        .header("Cache-Control", "only-if-cached, max-age=86400")
        .body(FAVICON_ICO)
}
