use std::sync::OnceLock;
use std::thread;

static SERVER_PORT: OnceLock<u16> = OnceLock::new();

pub fn get_port() -> u16 {
    *SERVER_PORT.get().unwrap_or(&9876)
}

pub fn start_youtube_server() {
    if SERVER_PORT.get().is_some() {
        return;
    }

    let port = 9876u16;
    SERVER_PORT.set(port).ok();

    thread::spawn(move || {
        let server = tiny_http::Server::http(format!("127.0.0.1:{port}"))
            .expect("[youtube_server] failed to bind");
        eprintln!("[youtube_server] listening on http://127.0.0.1:{port}");

        for request in server.incoming_requests() {
            let url = request.url().to_string();

            // Parse video id from /embed?v=VIDEO_ID
            let video_id = url
                .split('?')
                .nth(1)
                .and_then(|q| {
                    q.split('&')
                        .find(|p| p.starts_with("v="))
                        .map(|p| p.trim_start_matches("v=").to_string())
                })
                .unwrap_or_default();

            if video_id.is_empty() {
                let response =
                    tiny_http::Response::from_string("missing video id").with_status_code(400);
                let _ = request.respond(response);
                continue;
            }

            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  html, body {{ width: 100%; height: 100%; overflow: hidden; background: #000; }}
  iframe {{ width: 100%; height: 100%; border: none; display: block; }}
</style>
</head>
<body>
<iframe
  src="https://www.youtube-nocookie.com/embed/{video_id}?autoplay=1&enablejsapi=1&controls=0&disablekb=1&modestbranding=1&rel=0&iv_load_policy=3&playsinline=1&origin=http://127.0.0.1:{port}&vq=medium"
  allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
  allowfullscreen>
</iframe>
</body>
</html>"#
            );

            let response = tiny_http::Response::from_string(html).with_header(
                tiny_http::Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"text/html; charset=utf-8"[..],
                )
                .unwrap(),
            );
            let _ = request.respond(response);
        }
    });
}
