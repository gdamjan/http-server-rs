use actix_files::{Files, NamedFile};
use actix_web::{
    get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use futures::StreamExt;

use std::path::PathBuf;

pub async fn run(bind_addr: &str, root: &PathBuf) -> std::io::Result<()> {
    let root_ = root.clone();
    let data_root = web::Data::new(root_.clone());
    let s = HttpServer::new(move || {
        let static_files = Files::new("/", &root_)
            .show_files_listing()
            .redirect_to_slash_directory()
            .files_listing_renderer(crate::directory_listing::directory_listing);

        App::new()
            .app_data(data_root.clone())
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
async fn handle_tar(
    req: HttpRequest,
    root: web::Data<PathBuf>,
    tail: web::Path<String>,
) -> impl Responder {
    let rel = tail.trim_end_matches('/');
    //
    let candidate_tar = root.join(format!("{}.tar", rel));

    if candidate_tar.is_file() {
        match NamedFile::open_async(candidate_tar).await {
            Ok(named) => return named.into_response(&req),
            Err(e) => {
                log::error!("Failed to open existing tar file: {}", e);
                return HttpResponse::InternalServerError().body("Failed to open tar\n");
            }
        }
    }
     
    let fullpath = match root.join(&rel).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            log::warn!("canonicalize failed for {:?}: {}", rel, e);
            return HttpResponse::NotFound().body("Directory not found\n");
        }
    };

    if !fullpath.starts_with(root.as_path()) {
        log::warn!("requested path escapes root: {:?}", fullpath);
        return HttpResponse::Forbidden().body("Forbidden\n");
    }

    if !(fullpath.is_dir()) {
        return HttpResponse::NotFound().body("Directory not found\n");
    }

    let stream = crate::threaded_archiver::stream_tar_in_thread(fullpath).map(Ok::<_, Error>);
    let response = HttpResponse::Ok()
        .content_type("application/x-tar")
        .streaming(stream);

    response
}

const FAVICON_ICO: &[u8] = include_bytes!("favicon.png");

#[get("/favicon.ico")]
async fn favicon_ico() -> impl Responder {
    HttpResponse::Ok()
        .content_type("image/png")
        .append_header(("Cache-Control", "only-if-cached, max-age=86400"))
        .body(FAVICON_ICO)
}
