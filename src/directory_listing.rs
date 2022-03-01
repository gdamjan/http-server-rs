use actix_files::Directory;
use actix_web::dev::ServiceResponse;
use actix_web::{HttpRequest, HttpResponse};
use percent_encoding::{utf8_percent_encode, CONTROLS}; // NON_ALPHANUMERIC
use std::fmt::Write;
use std::path::Path;
use v_htmlescape::escape as escape_html_entity;

macro_rules! encode_file_url {
    ($path:ident) => {
        utf8_percent_encode(&$path, CONTROLS)
    };
}

// " -- &quot;  & -- &amp;  ' -- &#x27;  < -- &lt;  > -- &gt;  / -- &#x2f;
macro_rules! encode_file_name {
    ($entry:ident) => {
        escape_html_entity(&$entry.file_name().to_string_lossy())
    };
}

pub fn directory_listing(
    dir: &Directory,
    req: &HttpRequest,
) -> Result<ServiceResponse, std::io::Error> {
    let index_of = req.path().trim_end_matches('/');
    let mut body = String::new();
    let base = Path::new(req.path());

    for entry in dir.path.read_dir()? {
        if dir.is_visible(&entry) {
            let entry = entry.unwrap();
            let p = match entry.path().strip_prefix(&dir.path) {
                Ok(p) if cfg!(windows) => base.join(p).to_string_lossy().replace("\\", "/"),
                Ok(p) => base.join(p).to_string_lossy().into_owned(),
                Err(_) => continue,
            };

            // if file is a directory, add '/' to the end of the name
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    let _ = write!(
                        body,
                        "<tr><td>📂 <a href='{}/'>{}/</a></td> <td><small>[<a href='{}.tar'>.tar</a>]</small></td></tr>",
                        encode_file_url!(p),
                        encode_file_name!(entry),
                        encode_file_url!(p),
                    );
                } else {
                    let _ = write!(
                        body,
                        "<tr><td>🗎 <a href='{}'>{}</a></td> <td>{}</td></tr>",
                        encode_file_url!(p),
                        encode_file_name!(entry),
                        metadata.len(),
                    );
                }
            } else {
                continue;
            }
        }
    }

    let header = format!(
        "<h1>Index of {}/</h1>\n\
         <small>[<a href='{}.tar'>.tar</a> of whole directory]</small>",
        index_of,
        if index_of.is_empty() { "_" } else { index_of }
    );

    let footer = format!(
        r#"<footer><a href="{}">{} {}</a></footer>"#,
        env!("CARGO_PKG_HOMEPAGE"),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let style = include_str!("style.css");

    let html = format!(
        "<!DOCTYPE html>\n\
         <html>\n\
         <head>\n\
         <title>Index of {}</title>\n\
         <style>\n{}</style></head>\n\
         <body>\n{}\n\
         <table>\n\
         <tr><td>📁 <a href='../'>../</a></td><td>Size</td></tr>\n\
         {}\
         </table>\n\
         {}\
         </body>\n</html>",
        index_of, style, header, body, footer
    );

    Ok(ServiceResponse::new(
        req.clone(),
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
    ))
}
