use crate::cmd::session::{clear_user_jwt, load_user_jwt, save_user_jwt, session_file_path};
use chain_access::ports::privy::APP_ID_ENV_VAR;
use console::style;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

fn verify_user_jwt(jwt: &str) -> anyhow::Result<()> {
    let trimmed = jwt.trim();
    if trimmed.is_empty() {
        anyhow::bail!("--jwt cannot be empty");
    }

    let app_id = std::env::var(APP_ID_ENV_VAR)
        .map_err(|_| anyhow::anyhow!("missing env var: {APP_ID_ENV_VAR}"))?;

    // Decode the JWT payload (base64url, no signature verification) to check
    // that it is a well-formed Privy access token for the expected app.
    let parts: Vec<&str> = trimmed.splitn(3, '.').collect();
    if parts.len() != 3 {
        anyhow::bail!("JWT is not a valid three-part token");
    }

    let payload_b64 = parts[1];
    // base64url → standard base64 and pad
    let b64 = payload_b64.replace('-', "+").replace('_', "/");
    let padded = match b64.len() % 4 {
        2 => format!("{b64}=="),
        3 => format!("{b64}="),
        _ => b64,
    };
    let payload_bytes = base64_decode(&padded)
        .map_err(|e| anyhow::anyhow!("failed to decode JWT payload: {e}"))?;
    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes)
        .map_err(|e| anyhow::anyhow!("JWT payload is not valid JSON: {e}"))?;

    // Privy access tokens have "iss" = "privy.io" and "aud" = the app ID.
    let iss = payload.get("iss").and_then(|v| v.as_str()).unwrap_or_default();
    if iss != "privy.io" {
        anyhow::bail!("JWT issuer is '{iss}', expected 'privy.io'");
    }

    let aud = payload.get("aud").and_then(|v| v.as_str()).unwrap_or_default();
    if aud != app_id {
        anyhow::bail!("JWT audience is '{aud}', expected '{app_id}'");
    }

    // Check expiry if present
    if let Some(exp) = payload.get("exp").and_then(|v| v.as_i64()) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        if exp < now {
            anyhow::bail!("JWT has expired (exp={exp}, now={now})");
        }
    }

    Ok(())
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|e| e.to_string())
}

fn extract_jwt_from_body(body: &str) -> Option<String> {
    for pair in body.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next()?;
        let value = parts.next().unwrap_or_default();
        if key == "jwt" {
            let decoded = url_decode_form_component(value)?;
            return Some(decoded);
        }
    }
    None
}

fn url_decode_form_component(value: &str) -> Option<String> {
    let mut out = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' => {
                if i + 2 >= bytes.len() {
                    return None;
                }
                let hi = bytes[i + 1] as char;
                let lo = bytes[i + 2] as char;
                let hex = [hi, lo].iter().collect::<String>();
                let val = u8::from_str_radix(&hex, 16).ok()?;
                out.push(val);
                i += 3;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }

    String::from_utf8(out).ok()
}

/// Pre-built Vite+React login app (single-file HTML with all Privy deps bundled).
/// Built from stablectl/login-app/ via `npm run build`.
const LOGIN_APP_HTML: &str = include_str!("../../login-app/dist/index.html");

fn login_browser_html(app_id: &str) -> String {
    LOGIN_APP_HTML.replacen(
        "</head>",
        &format!(
            r#"<script>window.__STABLECTL_APP_ID__="{app_id}";</script></head>"#,
            app_id = app_id
        ),
        1,
    )
}

pub async fn run_login_browser(port: u16) -> anyhow::Result<()> {
    let app_id = std::env::var(APP_ID_ENV_VAR)
        .map_err(|_| anyhow::anyhow!("missing env var: {APP_ID_ENV_VAR}"))?;

    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)
        .map_err(|e| anyhow::anyhow!("failed to bind login listener on {addr}: {e}"))?;

    let url = format!("http://{addr}/");
    println!(
        "{} {}",
        style("Open this URL in your browser:").green(),
        style(&url).cyan()
    );

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&url).status();
    }

    let mut captured_jwt: Option<String> = None;

    while captured_jwt.is_none() {
        let (mut stream, _) = listener.accept().map_err(|e| {
            anyhow::anyhow!("login-browser timed out waiting for browser input: {e}")
        })?;
        stream
            .set_read_timeout(Some(Duration::from_secs(300)))
            .map_err(|e| anyhow::anyhow!("failed to set connection timeout: {e}"))?;

        let mut buf = [0u8; 16 * 1024];
        let size = stream
            .read(&mut buf)
            .map_err(|e| anyhow::anyhow!("failed to read browser request: {e}"))?;
        if size == 0 {
            continue;
        }

        let request = String::from_utf8_lossy(&buf[..size]);
        let mut lines = request.lines();
        let first_line = lines.next().unwrap_or_default();

        if first_line.starts_with("GET /") {
            let body = login_browser_html(&app_id);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
            continue;
        }

        if first_line.starts_with("POST /token") {
            let body = request.split("\r\n\r\n").nth(1).unwrap_or_default();
            if let Some(jwt) = extract_jwt_from_body(body) {
                captured_jwt = Some(jwt);
                let success = "<html><body><h2>Token received.</h2><p>Return to your terminal for validation result.</p></body></html>";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    success.len(),
                    success
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            } else {
                let fail = "<html><body><h2>Missing JWT.</h2><p>Go back and paste a JWT.</p></body></html>";
                let response = format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    fail.len(),
                    fail
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        }
    }

    let jwt = captured_jwt.unwrap_or_default();
    run_login(&jwt).await
}

pub async fn run_login(jwt: &str) -> anyhow::Result<()> {
    verify_user_jwt(jwt)?;
    let path = save_user_jwt(jwt)?;
    println!(
        "{} {}",
        style("Validated and saved user session at").green(),
        style(path.display()).cyan()
    );
    println!(
        "{}",
        style("You can now use --signer privy-user with --wallet-id <WALLET_ID>").dim()
    );
    Ok(())
}

pub async fn run_logout() -> anyhow::Result<()> {
    let removed = clear_user_jwt()?;
    if removed {
        println!("{}", style("Signer session cleared").green());
    } else {
        println!("{}", style("No signer session found").yellow());
    }
    Ok(())
}

pub async fn run_whoami() -> anyhow::Result<()> {
    let path = session_file_path()?;
    let jwt = load_user_jwt();

    match jwt {
        Ok(token) => {
            let preview = if token.len() > 16 {
                format!("{}...{}", &token[..8], &token[token.len() - 8..])
            } else {
                token
            };

            println!("{}", style("Signer session: privy-user").bold().green());
            println!("  {} {}", style("Path").dim(), style(path.display()).cyan());
            println!("  {} {}", style("JWT").dim(), style(preview).cyan());
            Ok(())
        }
        Err(_) => {
            println!("{}", style("No signer session found").yellow());
            println!(
                "{}",
                style("Run: stablectl signer login --jwt <REAL_PRIVY_USER_JWT>").dim()
            );
            Ok(())
        }
    }
}
