use actix_files::Files;
use actix_web::{get, middleware, web, App, HttpServer, HttpResponse, Responder};


// use crate::threaded_archiver;

use std::path::PathBuf;


pub async fn run(bind_addr: &str, root: &PathBuf) -> std::io::Result<()> {
    let root_ = root.clone();
    let s = HttpServer::new(move || {

        let static_files = Files::new("/", &root_)
                    .show_files_listing()
                    .redirect_to_slash_directory()
                    .files_listing_renderer(crate::listing::directory_listing);

        App::new()
            .app_data(root_.clone())
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
async fn handle_tar(_root: web::Data<PathBuf>, web::Path(_tail): web::Path<String>) -> impl Responder {
//     let relpath = PathBuf::from(tail.trim_end_matches('/'));
//     let fullpath = root.join(&relpath).canonicalize()?;

//     if !(fullpath.is_dir()) {
//         return Err(error::ErrorBadRequest("not a directory"));
//     }

//     let stream = threaded_archiver::stream_tar_in_thread(fullpath);
//     let resp = HttpResponse::Ok()
//         .content_type("application/x-tar")
//         .streaming(stream.map_err(|_e| error::ErrorBadRequest("stream error")));
//     Ok(resp)
    HttpResponse::Ok()
}

const FAVICON_ICO: &'static [u8] = include_bytes!("favicon.png");

#[get("/favicon.ico")]
async fn favicon_ico() -> impl Responder {
    HttpResponse::Ok()
        .content_type("image/png")
        .header("Cache-Control", "only-if-cached, max-age=86400")
        .body(FAVICON_ICO)
}
