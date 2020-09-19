use actix_files::Directory;
use actix_web::{HttpRequest, HttpResponse};
use actix_web::dev::ServiceResponse;
use std::path::Path;
use percent_encoding::{utf8_percent_encode, CONTROLS}; // NON_ALPHANUMERIC
use v_htmlescape::escape as escape_html_entity;
use std::fmt::Write;

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
    let index_of = format!("Index of {}", req.path());
    let mut body = String::new();
    let base = Path::new(req.path());

    for entry in dir.path.read_dir()? {
        if dir.is_visible(&entry) {
            let entry = entry.unwrap();
            let p = match entry.path().strip_prefix(&dir.path) {
                Ok(p) if cfg!(windows) => {
                    base.join(p).to_string_lossy().replace("\\", "/")
                }
                Ok(p) => base.join(p).to_string_lossy().into_owned(),
                Err(_) => continue,
            };

            // if file is a directory, add '/' to the end of the name
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    let _ = write!(
                        body,
                        "<li><a href=\"{}\">{}/</a></li>",
                        encode_file_url!(p),
                        encode_file_name!(entry),
                    );
                } else {
                    let _ = write!(
                        body,
                        "<li><a href=\"{}\">{}</a></li>",
                        encode_file_url!(p),
                        encode_file_name!(entry),
                    );
                }
            } else {
                continue;
            }
        }
    }

    let html = format!(
        "<html>\
         <head><title>{}</title></head>\
         <body><h1>{}</h1>\
         <ul>\
         {}\
         </ul></body>\n</html>",
        index_of, index_of, body
    );
    Ok(ServiceResponse::new(
        req.clone(),
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
    ))
}

// fn handle_directory(
//     dir: &fs::Directory,
//     req: &HttpRequest,
// ) -> Result<ServiceResponse, std::io::Error> {
//     let rd = std::fs::read_dir(&dir.path)?;

//     fn optimistic_is_dir(entry: &std::fs::DirEntry) -> bool {
//         // consider it non directory if metadata reading fails, better than an unwrap() panic
//         entry
//             .metadata()
//             .map(|m| m.file_type().is_dir())
//             .unwrap_or(false)
//     }
//     let mut paths: Vec<_> = rd
//         .filter_map(|entry| {
//             if dir.is_visible(&entry) {
//                 entry.ok()
//             } else {
//                 None
//             }
//         })
//         .collect();
//     paths.sort_by_key(|entry| (!optimistic_is_dir(entry), entry.file_name()));

//     let tar_url = req.path().trim_end_matches('/'); // this is already encoded

//     let mut body = String::new();
//     writeln!(body, "<h1>Index of {}</h1>", req.path()).unwrap(); // FIXME: decode from url, escape for html
//     writeln!(
//         body,
//         r#"<small>[<a href="{}.tar">.tar</a> of whole directory]</small>"#,
//         tar_url
//     )
//     .unwrap();
//     writeln!(body, "<table>").unwrap();
//     writeln!(
//         body,
//         "<tr><td>📁 <a href='../'>../</a></td><td>Size</td></tr>"
//     )
//     .unwrap();

//     for entry in paths {
//         let meta = entry.metadata()?;
//         let file_url =
//             utf8_percent_encode(&entry.file_name().to_string_lossy(), NON_ALPHANUMERIC).to_string();
//         let file_name = escape_html_entity(&entry.file_name().to_string_lossy()).to_string();
//         let size = meta.len();

//         write!(body, "<tr>").unwrap();
//         if meta.file_type().is_dir() {
//             writeln!(
//                 body,
//                 r#"<td>📂 <a href="{}/">{}/</a></td>"#,
//                 file_url, file_name
//             )
//             .unwrap();
//             write!(
//                 body,
//                 r#"    <td><small>[<a href="{}.tar">.tar</a>]</small></td>"#,
//                 file_url
//             )
//             .unwrap();
//         } else {
//             writeln!(
//                 body,
//                 r#"<td>🗎 <a href="{}">{}</a></td>"#,
//                 file_url, file_name
//             )
//             .unwrap();
//             write!(body, "    <td>{}</td>", size).unwrap();
//         }
//         writeln!(body, "</tr>").unwrap();
//     }
//     writeln!(body, "</table>").unwrap();
//     writeln!(
//         body,
//         r#"<footer><a href="{}">{} {}</a></footer>"#,
//         env!("CARGO_PKG_HOMEPAGE"),
//         env!("CARGO_PKG_NAME"),
//         env!("CARGO_PKG_VERSION")
//     )
//     .unwrap();

//     let mut html = String::new();
//     writeln!(html, "<!DOCTYPE html>").unwrap();
//     writeln!(html, "<html><head>").unwrap();
//     writeln!(html, "<title>Index of {}</title>", req.path()).unwrap();
//     writeln!(html, "<style>\n{}</style>", include_str!("style.css")).unwrap();
//     writeln!(html, "</head>").unwrap();
//     writeln!(html, "<body>\n{}</body>", body).unwrap();
//     writeln!(html, "</html>").unwrap();

//     let resp = HttpResponse::Ok()
//         .content_type("text/html; charset=utf-8")
//         .body(html);

//     Ok(ServiceResponse::new(req.clone(), resp))
// }
