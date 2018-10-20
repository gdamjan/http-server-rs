extern crate actix_web;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate tar;
extern crate htmlescape;
extern crate percent_encoding;
#[macro_use] extern crate clap;
#[macro_use] extern crate log;

use std::alloc::System;

#[global_allocator]
static GLOBAL: System = System;

mod channel;
mod web;

use actix_web::server;
use actix_web::actix;
use clap::Arg;

fn main() -> Result<(), std::io::Error> {
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
                    .value_name("PORT")
                    .help("Specify alternate port")
                    .default_value("8000")
                    .index(1));
    let matches = app.get_matches();

    let chdir = matches.value_of("chdir").unwrap(); // these shouldn't panic ever, since all have default_value
    let addr = matches.value_of("addr").unwrap();
    let port = matches.value_of("port").unwrap();
    let bind_addr = format!("{}:{}", addr, port);

    std::env::set_var("RUST_LOG", std::env::var("RUST_LOG").unwrap_or("info".to_string()));
    env_logger::init();

    let root = std::path::PathBuf::from(chdir).canonicalize()?;
    std::env::set_current_dir(&root)?;


    let sys = actix::System::new("http_server_rs");

    info!("Serving files from {:?}", root);
    server::new(move || web::create_app(&root).unwrap())
        .bind(&bind_addr)?
        .start();

    let _ = sys.run();
    Ok(())
}
