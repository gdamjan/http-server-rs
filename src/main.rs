mod threaded_archiver;
mod directory_listing;
mod web;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app = clap::App::new(clap::crate_name!())
        .author(clap::crate_authors!("\n"))
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .arg(
            clap::Arg::with_name("chdir")
                .short("w")
                .long("chdir")
                .value_name("DIRECTORY")
                .help("directory to serve")
                .default_value(".")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("addr")
                .short("b")
                .long("bind")
                .value_name("ADDRESS")
                .help("bind address")
                .default_value("0.0.0.0")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("port")
                .value_name("PORT")
                .help("Specify alternate port")
                .default_value("8000")
                .index(1),
        );
    let matches = app.get_matches();

    let chdir = matches.value_of("chdir").unwrap(); // these shouldn't panic ever, since all have default_value
    let addr = matches.value_of("addr").unwrap();
    let port = matches.value_of("port").unwrap();
    let bind_addr = format!("{}:{}", addr, port);

    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
    );
    env_logger::init();

    let root = std::path::PathBuf::from(chdir).canonicalize()?;
    std::env::set_current_dir(&root)?;

    web::run(&bind_addr, &root).await
}
