extern crate actix_web;
extern crate actix;
extern crate futures;
extern crate tar;

use actix_web::{server, error, fs, App, HttpRequest, HttpResponse, Error, Result, http::Method, http::StatusCode};

use std::env;
use std::io;
use std::path::PathBuf;


fn main() -> Result<(), Error> {
    let bind_addr = env::var("HTTP_ADDR").unwrap_or("127.0.0.1:8088".to_string());
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

fn handle_directory<'a, 'b>(
    dir: &'a fs::Directory,
    req: &'b HttpRequest,
) -> io::Result<HttpResponse> {
     Ok(HttpResponse::with_body(StatusCode::OK, "directory index here"))
}

fn handle_tar(req: &HttpRequest) -> Result<HttpResponse> {
    let path: PathBuf = req.match_info().query("tail")?;
    if !(path.is_dir()) {
        return Err(error::ErrorBadRequest("not a directory"));
    }

    Ok(HttpResponse::Ok().body(format!("fixme: {:?}\n", path)))
}


