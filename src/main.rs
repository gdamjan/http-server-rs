use std::path::PathBuf;

use clap::{arg, command, value_parser};

mod directory_listing;
mod threaded_archiver;
mod web;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = command!()
        .arg(
            arg!(
                -w --chdir <DIRECTORY> "directory to serve"
            )
            .value_parser(value_parser!(PathBuf))
            .default_value("."),
        )
        .arg(
            arg!(
                -b --bind <ADDRESS> "bind address"
            )
            .default_value("0.0.0.0"),
        )
        .arg(
            arg!([PORT] "Network port to use")
                .default_value("8000")
                .value_parser(value_parser!(u16)),
        );
    let matches = args.get_matches();

    let chdir = matches.get_one::<PathBuf>("chdir").unwrap(); // these shouldn't panic ever, since all have default_value
    let addr = matches.get_one::<String>("bind").unwrap();
    let port = matches.get_one::<u16>("PORT").unwrap();
    let bind_addr = format!("{}:{}", addr, port);

    env_logger::init();

    let root = chdir.canonicalize()?;
    std::env::set_current_dir(&root)?;

    web::run(&bind_addr, &root).await
}
