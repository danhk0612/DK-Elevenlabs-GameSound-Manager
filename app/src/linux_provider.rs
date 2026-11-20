//! Explicit, non-persistent Linux credential session and fixed-origin transport.
use crate::{
    provider::{Connection, Response},
    storage::{Error, Result},
};
use serde::Deserialize;
use serde_json::Value;
use std::{
    io::{IsTerminal, Read},
    sync::Mutex,
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

const INPUT_LIMIT: usize = 4096;
const BODY_LIMIT: u64 = 32 * 1024 * 1024;
struct Session {
    account: String,
    connection: Connection,
    expires: Instant,
}
static SESSION: Mutex<Option<Session>> = Mutex::new(None);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input<'a> {
    account_id: &'a str,
    key: &'a str,
    enabled: bool,
    request_limit: i64,
    credit_limit: i64,
    session_seconds: u64,
}
fn decode(bytes: &[u8]) -> Result<Session> {
    // Borrow the secret from a zeroized buffer: no Value tree/debug error includes it.
    let v: Input<'_> = serde_json::from_slice(bytes).map_err(|_| Error("invalid_session_input"))?;
    if !crate::catalog::valid_id(v.account_id)
        || v.key.is_empty()
        || v.key.len() > 256
        || !v.key.bytes().all(|b| b.is_ascii_graphic())
    {
        return Err(Error("invalid_session_input"));
    }
    if !(60..=86400).contains(&v.session_seconds)
        || !(0..=1_000_000).contains(&v.request_limit)
        || !(0..=1_000_000_000).contains(&v.credit_limit)
        || v.enabled && (v.request_limit == 0 || v.credit_limit == 0)
    {
        return Err(Error("invalid_session_policy"));
    }
    Ok(Session {
        account: v.account_id.to_string(),
        connection: Connection {
            key: Zeroizing::new(v.key.to_string()),
            enabled: v.enabled,
            request_limit: v.request_limit,
            credit_limit: v.credit_limit,
        },
        expires: Instant::now() + Duration::from_secs(v.session_seconds),
    })
}

pub(crate) fn bootstrap() -> Result<()> {
    crate::linux::root().map_err(Error)?;
    // A launcher/secret manager pipe only. No key filename, argv, env, terminal echo.
    if std::io::stdin().is_terminal() {
        return Err(Error("session_requires_pipe"));
    }
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstat(0, &mut stat) } != 0
        || !matches!(stat.st_mode & libc::S_IFMT, libc::S_IFIFO | libc::S_IFSOCK)
    {
        return Err(Error("session_requires_pipe"));
    }
    // Best effort memory hardening before accepting a secret; fail closed.
    let limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    if unsafe { libc::setrlimit(libc::RLIMIT_CORE, &limit) } != 0
        || unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0) } != 0
    {
        return Err(Error("session_memory_protection_failed"));
    }
    let mut bytes = Zeroizing::new(Vec::new());
    std::io::stdin()
        .take(INPUT_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error("session_input_io"))?;
    if bytes.len() > INPUT_LIMIT {
        return Err(Error("session_input_too_large"));
    }
    let session = decode(&bytes)?;
    *SESSION
        .lock()
        .map_err(|_| Error("credential_unavailable"))? = Some(session);
    Ok(())
}

pub(crate) fn clear_expired() {
    if let Ok(mut current) = SESSION.lock() {
        if current
            .as_ref()
            .is_some_and(|s| Instant::now() >= s.expires)
        {
            *current = None;
        }
    }
}

pub(crate) fn read(account: &str) -> Result<Connection> {
    let mut current = SESSION
        .lock()
        .map_err(|_| Error("credential_unavailable"))?;
    if current
        .as_ref()
        .is_some_and(|s| Instant::now() >= s.expires)
    {
        *current = None; // Drop/zeroize the stored key, never extend expiry on reads.
        return Err(Error("credential_session_expired"));
    }
    let s = current
        .as_ref()
        .filter(|s| s.account == account)
        .ok_or(Error("credential_unavailable"))?;
    Ok(Connection {
        key: Zeroizing::new(s.connection.key.to_string()),
        enabled: s.connection.enabled,
        request_limit: s.connection.request_limit,
        credit_limit: s.connection.credit_limit,
    })
}

fn client_builder() -> reqwest::blocking::ClientBuilder {
    let _ = rustls::crypto::ring::default_provider().install_default();
    reqwest::blocking::Client::builder()
        .tls_sslkeylogfile(false)
        .tls_backend_rustls()
        .https_only(true)
        .http1_only()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .no_proxy()
        .referer(false)
        .connection_verbose(false)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(120))
}
fn client() -> Result<reqwest::blocking::Client> {
    client_builder()
        .build()
        .map_err(|_| Error("provider_transport_error"))
}

fn path_valid(method: &str, path: &str) -> bool {
    // Defense in depth: no origin override, fragment, control chars or userinfo.
    matches!(method, "GET" | "POST")
        && (path.starts_with("/v1/") || path.starts_with("/v2/"))
        && path
            .bytes()
            .all(|b| b.is_ascii_graphic() && !b"\\#@".contains(&b))
        && !path.contains("..")
}
pub(crate) fn http(key: &str, method: &str, path: &str, body: &Value) -> Result<Response> {
    if !path_valid(method, path) {
        return Err(Error("invalid_provider_path"));
    }
    send(
        client()?,
        &format!("https://api.elevenlabs.io{path}"),
        key,
        method,
        body,
    )
}
fn send(
    client: reqwest::blocking::Client,
    url: &str,
    key: &str,
    method: &str,
    body: &Value,
) -> Result<Response> {
    let method = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|_| Error("invalid_provider_path"))?;
    let mut header =
        reqwest::header::HeaderValue::from_str(key).map_err(|_| Error("invalid_credential"))?;
    header.set_sensitive(true);
    let mut request = client
        .request(method.clone(), url)
        .header("xi-api-key", header)
        .header("Content-Type", "application/json");
    if method == reqwest::Method::POST {
        request = request.body(serde_json::to_vec(body).map_err(|_| Error("invalid_request"))?);
    }
    // A single send. Never return reqwest errors (which can include URLs/headers).
    let response = request
        .send()
        .map_err(|_| Error("provider_transport_error"))?;
    decode_response(response)
}
fn decode_response(response: reqwest::blocking::Response) -> Result<Response> {
    let status = response.status().as_u16() as u32;
    let credits = response
        .headers()
        .get("character-cost")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .filter(|n| *n >= 0);
    // Never consume an error body: it may echo secrets/prompt details.
    if status != 200 {
        return Ok(Response {
            status,
            body: Vec::new(),
            credits,
        });
    }
    if response.content_length().is_some_and(|n| n > BODY_LIMIT) {
        return Err(Error("provider_response_too_large"));
    }
    let mut body = Vec::new();
    response
        .take(BODY_LIMIT + 1)
        .read_to_end(&mut body)
        .map_err(|_| Error("provider_transport_error"))?;
    if body.len() as u64 > BODY_LIMIT {
        return Err(Error("provider_response_too_large"));
    }
    Ok(Response {
        status,
        body,
        credits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn session_input_is_explicit_bounded_and_never_echoed() {
        let input = br#"{"account_id":"session-test","key":"fake-only","enabled":true,"request_limit":2,"credit_limit":40,"session_seconds":60}"#;
        assert!(decode(input).is_ok());
        for bytes in [br#"{"key":"fake-only"}"#.as_slice(), b"fake-only", br#"{"account_id":"test","key":"fake-only","enabled":true,"request_limit":0,"credit_limit":40,"session_seconds":60}"#] {
            assert!(decode(bytes).is_err());
        }
        let invalid = String::from_utf8(input.to_vec())
            .unwrap()
            .replace("60}", "0}");
        assert_eq!(
            decode(invalid.as_bytes()).err().unwrap().0,
            "invalid_session_policy"
        );
        let duplicate = String::from_utf8(input.to_vec())
            .unwrap()
            .replace("\"enabled\":true", "\"enabled\":true,\"enabled\":false");
        assert_eq!(
            decode(duplicate.as_bytes()).err().unwrap().0,
            "invalid_session_input"
        );
        let unknown = String::from_utf8(input.to_vec())
            .unwrap()
            .replace("60}", "60,\"extra\":true}");
        assert!(decode(unknown.as_bytes()).is_err());
    }
    #[test]
    fn expired_or_other_account_cannot_reuse_the_session() {
        let mut session = decode(br#"{"account_id":"session-test","key":"fake-only","enabled":false,"request_limit":0,"credit_limit":0,"session_seconds":60}"#).unwrap();
        session.expires = Instant::now() - Duration::from_secs(1);
        *SESSION.lock().unwrap() = Some(session);
        assert_eq!(
            read("session-test").err().unwrap().0,
            "credential_session_expired"
        );
        assert_eq!(read("other").err().unwrap().0, "credential_unavailable");
    }
    #[test]
    fn fixed_origin_path_validation_rejects_injection() {
        for p in [
            "https://bad/v1/user",
            "//bad/v1/user",
            "/v1/user\r\nX: bad",
            "/v1/../user",
            "/v1/user#fragment",
            "/v1/user@bad",
            "/v1/\\bad",
        ] {
            assert!(!path_valid("GET", p));
        }
        assert!(path_valid(
            "POST",
            "/v1/sound-generation?output_format=mp3_44100_128"
        ));
        assert!(!path_valid("DELETE", "/v1/user"));
    }
    fn local_server(
        reply: Vec<u8>,
        trust: bool,
        drop_reply: bool,
    ) -> (
        String,
        reqwest::blocking::Client,
        std::thread::JoinHandle<usize>,
    ) {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        let generated = rcgen::generate_simple_self_signed(vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
        ])
        .unwrap();
        let certificate = generated.cert.der().clone();
        let key =
            rustls::pki_types::PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
        let _ = rustls::crypto::ring::default_provider().install_default();
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![certificate.clone()], key.into())
            .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let client = if trust {
            client_builder()
                .tls_certs_only([reqwest::Certificate::from_der(certificate.as_ref()).unwrap()])
        } else {
            client_builder()
        }
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
        let task = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(4);
            let mut count = 0;
            while Instant::now() < deadline {
                let (stream, _) = match listener.accept() {
                    Ok(v) => v,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(_) => break,
                };
                count += 1;
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let connection =
                    rustls::ServerConnection::new(std::sync::Arc::new(config.clone())).unwrap();
                let mut tls = rustls::StreamOwned::new(connection, stream);
                let mut bytes = [0; 4096];
                if let Ok(n) = tls.read(&mut bytes) {
                    let request = String::from_utf8_lossy(&bytes[..n]).to_ascii_lowercase();
                    assert!(request.contains("xi-api-key: fake-only"));
                    if !drop_reply {
                        let _ = tls.write_all(&reply);
                        let _ = tls.flush();
                    }
                }
                // A retry would connect to the listener again. Poll briefly to count it.
                let until = Instant::now() + Duration::from_millis(200);
                while Instant::now() < until {
                    if let Ok((_extra, _)) = listener.accept() {
                        count += 1;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                break;
            }
            count
        });
        (format!("https://{address}/v1/test"), client, task)
    }
    #[test]
    fn tls_transport_bounds_costs_errors_redirects_and_never_retries() {
        let (url, client, task) = local_server(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\ncharacter-cost: 7\r\nConnection: close\r\n\r\nabc".to_vec(), true, false);
        let r = send(
            client,
            &url,
            "fake-only",
            "POST",
            &serde_json::json!({"text":"synthetic"}),
        )
        .unwrap();
        assert_eq!(r.status, 200);
        assert_eq!(r.body, b"abc");
        assert_eq!(r.credits, Some(7));
        assert_eq!(task.join().unwrap(), 1);
        for status in [401, 429, 302] {
            let reply = format!("HTTP/1.1 {status} test\r\nContent-Length: 9\r\nLocation: https://example.invalid/\r\nConnection: close\r\n\r\nfake-only").into_bytes();
            let (url, client, task) = local_server(reply, true, false);
            let r = send(client, &url, "fake-only", "GET", &Value::Null).unwrap();
            assert_eq!(r.status, status);
            assert!(r.body.is_empty());
            assert!(r.credits.is_none());
            assert_eq!(task.join().unwrap(), 1);
        }
        let (url, client, task) = local_server(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                BODY_LIMIT + 1
            )
            .into_bytes(),
            true,
            false,
        );
        assert_eq!(
            send(client, &url, "fake-only", "GET", &Value::Null)
                .err()
                .unwrap()
                .0,
            "provider_response_too_large"
        );
        assert_eq!(task.join().unwrap(), 1);
        let (url, client, task) = local_server(Vec::new(), true, true);
        assert_eq!(
            send(client, &url, "fake-only", "POST", &Value::Null)
                .err()
                .unwrap()
                .0,
            "provider_transport_error"
        );
        assert_eq!(task.join().unwrap(), 1);
        let (url, client, task) = local_server(Vec::new(), false, false);
        assert_eq!(
            send(client, &url, "fake-only", "GET", &Value::Null)
                .err()
                .unwrap()
                .0,
            "provider_transport_error"
        );
        assert_eq!(task.join().unwrap(), 1);
        // HTTPS-only is enforced before any credential-bearing request can use plaintext.
        assert!(send(
            super::client().unwrap(),
            "http://127.0.0.1:1/v1/test",
            "fake-only",
            "GET",
            &Value::Null
        )
        .is_err());
    }
}
