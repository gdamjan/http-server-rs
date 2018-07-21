extern crate actix_web;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate tar;
extern crate htmlescape;
extern crate percent_encoding;
#[macro_use]
extern crate clap;
#[macro_use] extern crate log;

mod channel;
mod web;

use actix_web::server;
use actix_web::actix;
use clap::Arg;

use std::io;

fn main() -> Result<(), io::Error> {
    let app = clap::App::new(crate_name!())
                .author(crate_authors!("\n"))
                .version(crate_version!())
                .about(crate_description!())
                .arg(Arg::with_name("chdir")
                    .long("chdir")
                    .value_name("DIRECTORY")
                    .help("Specify directory to server")
                    .default_value(".")
                    .takes_value(true))
                .arg(Arg::with_name("addr")
                    .long("bind")
                    .value_name("ADDRESS")
                    .help("Specify alternate bind address")
                    .default_value("0.0.0.0")
                    .takes_value(true))
                .arg(Arg::with_name("port")
                    .help("Specify alternate port")
                    .default_value("8000")
                    .index(1));
    let matches = app.get_matches();

    let chdir = matches.value_of("chdir").unwrap();
    let port = matches.value_of("port").unwrap();
    let addr = matches.value_of("addr").unwrap();
    let bind_addr = format!("{}:{}", addr, port);

    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let sys = actix::System::new("http_server_rs");

    let directory = String::from(chdir);
    server::new(move || web::create_app(&directory))
        .bind(&bind_addr)
        .expect(&format!("Can't listen on {} ", bind_addr))
        .start();

    info!("Serving files from {}", chdir);

    let _ = sys.run();
    Ok(())
}
