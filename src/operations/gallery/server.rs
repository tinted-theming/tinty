//! Minimal localhost HTTP server backing `tinty gallery --serve`.
//!
//! The static gallery (`tinty gallery` / `tinty gallery --dump`) is a
//! self-contained site with no server. This module powers the opt-in *live*
//! variant: it serves the same embedded assets from memory and exposes a tiny
//! JSON API that runs real Tinty operations on this machine — reading the
//! currently applied scheme and applying a new one on request.
//!
//! It is intentionally dependency-free (just `std` + the `serde_json` already
//! used elsewhere). The only clients are this gallery's own JavaScript and the
//! browser fetching assets, so the request surface is small and controlled.

use crate::operations::apply::apply;
use crate::operations::current::get_current_scheme_slug;
use anyhow::{anyhow, Context as _, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// Pre-rendered gallery assets handed to the server. Strings are owned because
/// they have already had their placeholders substituted (scheme data, the live
/// flag); the binary assets stay as borrowed `'static` slices.
pub struct Assets {
    pub index_html: String,
    pub gallery_js: String,
    pub gallery_css: &'static str,
    pub logo: &'static [u8],
    pub favicon: &'static [u8],
    pub font_dm_serif_400: &'static [u8],
    pub font_dm_serif_400_italic: &'static [u8],
    pub font_ibm_plex_mono_400: &'static [u8],
    pub font_ibm_plex_mono_500: &'static [u8],
}

/// Shared, immutable-ish state handed to every connection thread.
struct ServerContext {
    assets: Assets,
    config_path: PathBuf,
    data_path: PathBuf,
    /// Serializes `apply` calls so two in-flight requests can't race on the
    /// artifacts directory.
    apply_lock: Mutex<()>,
}

#[derive(Deserialize)]
struct ApplyRequest {
    scheme: String,
}

struct ParsedRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

struct Response {
    status: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
}

impl Response {
    const fn html(body: String) -> Self {
        Self {
            status: "200 OK",
            content_type: "text/html; charset=utf-8",
            body: body.into_bytes(),
        }
    }

    fn css(body: &str) -> Self {
        Self {
            status: "200 OK",
            content_type: "text/css; charset=utf-8",
            body: body.as_bytes().to_vec(),
        }
    }

    const fn js(body: String) -> Self {
        Self {
            status: "200 OK",
            content_type: "text/javascript; charset=utf-8",
            body: body.into_bytes(),
        }
    }

    fn png(body: &[u8]) -> Self {
        Self {
            status: "200 OK",
            content_type: "image/png",
            body: body.to_vec(),
        }
    }

    fn font(body: &[u8]) -> Self {
        Self {
            status: "200 OK",
            content_type: "font/woff2",
            body: body.to_vec(),
        }
    }

    fn json(status: &'static str, value: &Value) -> Self {
        Self {
            status,
            content_type: "application/json; charset=utf-8",
            body: value.to_string().into_bytes(),
        }
    }

    fn not_found() -> Self {
        Self {
            status: "404 Not Found",
            content_type: "text/plain; charset=utf-8",
            body: b"Not found".to_vec(),
        }
    }

    fn method_not_allowed() -> Self {
        Self {
            status: "405 Method Not Allowed",
            content_type: "text/plain; charset=utf-8",
            body: b"Method not allowed".to_vec(),
        }
    }
}

/// Binds a localhost listener and serves the live gallery until interrupted.
pub fn serve(
    assets: Assets,
    config_path: PathBuf,
    data_path: PathBuf,
    port: Option<u16>,
    should_open: bool,
) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port.unwrap_or(0)))
        .context("Unable to start the gallery server")?;
    let local_addr = listener
        .local_addr()
        .context("Unable to read the gallery server address")?;
    let url = format!("http://127.0.0.1:{}/", local_addr.port());

    let context = Arc::new(ServerContext {
        assets,
        config_path,
        data_path,
        apply_lock: Mutex::new(()),
    });

    println!("Live gallery served at {url}");
    println!("Schemes you apply here are applied on this machine. Press Ctrl+C to stop.");

    if should_open {
        if let Err(err) = open_url(&url) {
            eprintln!("Unable to open the gallery in a browser: {err:#}");
        }
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let context = Arc::clone(&context);
                thread::spawn(move || {
                    if let Err(err) = handle_connection(&stream, &context) {
                        eprintln!("gallery request error: {err:#}");
                    }
                });
            }
            Err(err) => eprintln!("gallery connection error: {err}"),
        }
    }

    Ok(())
}

fn handle_connection(stream: &TcpStream, context: &ServerContext) -> Result<()> {
    let Some(request) = read_request(stream)? else {
        return Ok(());
    };
    let response = route(&request, context);
    write_response(stream, &response)
}

/// Parses the request line, headers, and (for bodied methods) the body. Returns
/// `Ok(None)` when the peer closed the connection without sending anything.
fn read_request(stream: &TcpStream) -> Result<Option<ParsedRequest>> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(None);
    }

    let mut tokens = request_line.split_whitespace();
    let method = tokens.next().unwrap_or_default().to_string();
    let raw_target = tokens.next().unwrap_or_default();
    // Drop any query string; routing only cares about the path.
    let path = raw_target
        .split('?')
        .next()
        .unwrap_or(raw_target)
        .to_string();

    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().unwrap_or(0);
            }
        }
    }

    let mut body = Vec::new();
    if content_length > 0 {
        let limit = u64::try_from(content_length).context("Request body is too large")?;
        reader.take(limit).read_to_end(&mut body)?;
    }

    Ok(Some(ParsedRequest { method, path, body }))
}

fn route(request: &ParsedRequest, context: &ServerContext) -> Response {
    let assets = &context.assets;
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/" | "/index.html") => Response::html(assets.index_html.clone()),
        ("GET", "/assets/gallery.css") => Response::css(assets.gallery_css),
        ("GET", "/assets/gallery.js") => Response::js(assets.gallery_js.clone()),
        ("GET", "/assets/tinted-theming-logo.png") => Response::png(assets.logo),
        ("GET", "/assets/favicon.png") => Response::png(assets.favicon),
        ("GET", "/assets/fonts/dm-serif-display-400.woff2") => {
            Response::font(assets.font_dm_serif_400)
        }
        ("GET", "/assets/fonts/dm-serif-display-400-italic.woff2") => {
            Response::font(assets.font_dm_serif_400_italic)
        }
        ("GET", "/assets/fonts/ibm-plex-mono-400.woff2") => {
            Response::font(assets.font_ibm_plex_mono_400)
        }
        ("GET", "/assets/fonts/ibm-plex-mono-500.woff2") => {
            Response::font(assets.font_ibm_plex_mono_500)
        }
        ("GET", "/api/current") => current_response(context),
        ("POST", "/api/apply") => apply_response(request, context),
        ("GET", _) => Response::not_found(),
        _ => Response::method_not_allowed(),
    }
}

/// `GET /api/current` — the currently applied scheme slug, or `null`.
fn current_response(context: &ServerContext) -> Response {
    let current = get_current_scheme_slug(&context.data_path);
    let trimmed = current.trim();
    let scheme = if trimmed.is_empty() {
        Value::Null
    } else {
        Value::String(trimmed.to_owned())
    };
    Response::json("200 OK", &json!({ "scheme": scheme }))
}

/// `POST /api/apply` — applies the requested scheme on this machine.
fn apply_response(request: &ParsedRequest, context: &ServerContext) -> Response {
    let payload: ApplyRequest = match serde_json::from_slice(&request.body) {
        Ok(payload) => payload,
        Err(err) => {
            return Response::json(
                "400 Bad Request",
                &json!({ "ok": false, "error": format!("Invalid request body: {err}") }),
            );
        }
    };

    let Ok(_guard) = context.apply_lock.lock() else {
        return Response::json(
            "500 Internal Server Error",
            &json!({ "ok": false, "error": "Apply lock was poisoned" }),
        );
    };

    match apply(
        &context.config_path,
        &context.data_path,
        &payload.scheme,
        true,
        None,
    ) {
        Ok(()) => {
            println!("Applied {}", payload.scheme);
            Response::json("200 OK", &json!({ "ok": true, "scheme": payload.scheme }))
        }
        Err(err) => Response::json(
            "400 Bad Request",
            &json!({ "ok": false, "error": format!("{err:#}") }),
        ),
    }
}

fn write_response(mut stream: &TcpStream, response: &Response) -> Result<()> {
    let header = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n",
        response.status,
        response.content_type,
        response.body.len(),
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(&response.body)?;
    stream.flush()?;
    Ok(())
}

fn open_url(url: &str) -> Result<()> {
    let status = opener_command(url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .with_context(|| format!("Unable to open {url}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Unable to open {url}"))
    }
}

#[cfg(target_os = "macos")]
fn opener_command(url: &str) -> Command {
    let mut command = Command::new("open");
    command.arg(url);
    command
}

#[cfg(target_os = "windows")]
fn opener_command(url: &str) -> Command {
    let mut command = Command::new("cmd");
    command.args(["/C", "start", ""]).arg(url);
    command
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
fn opener_command(url: &str) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(url);
    command
}
