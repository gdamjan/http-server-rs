extern crate actix_web;
extern crate actix;
extern crate bytes;
extern crate futures;
extern crate tar;
extern crate htmlescape;
extern crate percent_encoding;

mod channel;
mod web;

use actix_web::server;

use std::env;
use std::io;

// TODO cli args
fn main() -> Result<(), io::Error> {
    let bind_addr = env::var("HTTP_ADDR").unwrap_or(String::from("0.0.0.0:8000"));
    let sys = actix::System::new("static_index");

    server::new(web::create_app)
    .bind(&bind_addr)
    .expect(&format!("Can't listen on {} ", bind_addr))
    .start();

    println!("Started http server: {}", bind_addr);
    let _ = sys.run();
    Ok(())
}
