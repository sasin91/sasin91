//! A development server for ./public: `cargo run -- serve [port]`.
//!
//! Standard library only, localhost only, GET and HEAD only. It resolves
//! paths the way the Caddy config in docs/deploy.md does (`try_files {path}
//! {path}/ {path}/index.html`), including the 308 from `/blog` to `/blog/`,
//! so what works here works live.
//!
//! Every request reads the file fresh and nothing holds `public/` open, so a
//! rebuild can delete and recreate it while the server keeps running.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};

pub const DEFAULT_PORT: u16 = 8000;

pub fn run(root: &Path, port: u16) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("binding 127.0.0.1:{port} (is something else using it?)"))?;
    println!(
        "serving {}/ at http://localhost:{port}/ (Ctrl+C to stop)",
        root.display()
    );

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let root = root.to_path_buf();
        std::thread::spawn(move || {
            if let Err(error) = handle(stream, &root) {
                eprintln!("serve: {error:#}");
            }
        });
    }
    Ok(())
}

fn handle(mut stream: TcpStream, root: &Path) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    // The headers are not needed, but must be read before replying.
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 || header.trim().is_empty() {
            break;
        }
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or("/");
    let response = respond(root, method, target);
    println!("{} {method} {target}", response.status);

    let mut head = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n",
        response.status,
        reason(response.status),
        response.length
    );
    for (name, value) in &response.headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    stream.write_all(&response.body)?;
    Ok(())
}

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(&'static str, String)>,
    /// The Content-Length, which for HEAD is the length of the body that a
    /// GET would have sent.
    pub length: usize,
    pub body: Vec<u8>,
}

fn text(status: u16, message: &str) -> Response {
    Response {
        status,
        headers: vec![("Content-Type", "text/plain; charset=utf-8".into())],
        length: message.len(),
        body: message.as_bytes().to_vec(),
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Internal Server Error",
    }
}

/// Everything about a request except the socket, so it can be tested.
pub fn respond(root: &Path, method: &str, target: &str) -> Response {
    if method != "GET" && method != "HEAD" {
        let mut response = text(405, "method not allowed\n");
        response.headers.push(("Allow", "GET, HEAD".into()));
        return response;
    }

    let path = target.split(['?', '#']).next().unwrap_or("/");
    let Some(decoded) = percent_decode(path) else {
        return text(400, "bad percent-encoding\n");
    };
    let Some(file) = resolve(root, &decoded) else {
        return text(404, "not found\n");
    };

    let file = if file.is_dir() {
        if !decoded.ends_with('/') {
            let mut response = text(308, "");
            response.headers.push(("Location", format!("{path}/")));
            return response;
        }
        file.join("index.html")
    } else {
        file
    };

    match std::fs::read(&file) {
        Ok(body) => Response {
            status: 200,
            headers: vec![("Content-Type", content_type(&file).into())],
            length: body.len(),
            body: if method == "HEAD" { Vec::new() } else { body },
        },
        Err(_) => text(404, "not found\n"),
    }
}

/// Maps a URL path onto `root`, refusing anything that would step outside
/// it. `None` means "no such file".
fn resolve(root: &Path, url_path: &str) -> Option<PathBuf> {
    let mut path = root.to_path_buf();
    for component in Path::new(url_path.trim_start_matches('/')).components() {
        match component {
            Component::Normal(part) => path.push(part),
            Component::CurDir => {}
            // `..`, a drive prefix or a second root: never serve it.
            _ => return None,
        }
    }
    path.exists().then_some(path)
}

fn percent_decode(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn content_type(file: &Path) -> &'static str {
    let extension = file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "webm" => "video/webm",
        "pdf" => "application/pdf",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway public/ with one page, one nested page and one asset.
    fn site(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("serve-{name}-{}", std::process::id()));
        std::fs::create_dir_all(root.join("blog/post")).unwrap();
        std::fs::write(root.join("index.html"), "<p>home</p>").unwrap();
        std::fs::write(root.join("blog/post/index.html"), "<p>post</p>").unwrap();
        std::fs::write(root.join("site.abc.css"), "body{}").unwrap();
        root
    }

    #[test]
    fn serves_a_file_with_its_content_type() {
        let root = site("file");
        let r = respond(&root, "GET", "/site.abc.css");
        assert_eq!(r.status, 200);
        assert_eq!(r.body, b"body{}");
        assert!(
            r.headers
                .contains(&("Content-Type", "text/css; charset=utf-8".into()))
        );
    }

    #[test]
    fn serves_a_directory_index_and_redirects_to_the_trailing_slash_like_caddy() {
        let root = site("dir");
        assert_eq!(respond(&root, "GET", "/").body, b"<p>home</p>");
        assert_eq!(respond(&root, "GET", "/blog/post/").body, b"<p>post</p>");

        let redirect = respond(&root, "GET", "/blog/post?x=1");
        assert_eq!(redirect.status, 308);
        assert!(
            redirect
                .headers
                .contains(&("Location", "/blog/post/".into()))
        );

        // A directory with no index.html is not a page.
        assert_eq!(respond(&root, "GET", "/blog/").status, 404);
    }

    #[test]
    fn ignores_the_query_and_fragment_and_decodes_the_path() {
        let root = site("query");
        assert_eq!(respond(&root, "GET", "/site.abc.css?v=2#x").status, 200);
        assert_eq!(respond(&root, "GET", "/site%2Eabc.css").status, 200);
        assert_eq!(respond(&root, "GET", "/bad%zz").status, 400);
    }

    #[test]
    fn never_serves_outside_the_root() {
        let root = site("escape");
        std::fs::write(root.join("../serve-secret.txt"), "secret").ok();
        for target in [
            "/../serve-secret.txt",
            "/%2e%2e/serve-secret.txt",
            "/blog/../../serve-secret.txt",
        ] {
            assert_eq!(respond(&root, "GET", target).status, 404, "{target}");
        }
    }

    #[test]
    fn head_sends_the_length_but_no_body() {
        let root = site("head");
        let r = respond(&root, "HEAD", "/site.abc.css");
        assert_eq!((r.status, r.length), (200, 6));
        assert!(r.body.is_empty());
    }

    #[test]
    fn missing_files_and_other_methods_are_refused() {
        let root = site("refuse");
        assert_eq!(respond(&root, "GET", "/nope.html").status, 404);
        let post = respond(&root, "POST", "/");
        assert_eq!(post.status, 405);
        assert!(post.headers.contains(&("Allow", "GET, HEAD".into())));
    }
}
