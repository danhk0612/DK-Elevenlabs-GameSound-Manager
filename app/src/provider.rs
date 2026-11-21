use crate::storage::{Error, Result};
use serde_json::{json, Value};
use zeroize::Zeroizing;

pub struct Connection {
    pub key: Zeroizing<String>,
    pub enabled: bool,
    pub request_limit: i64,
    pub credit_limit: i64,
}
pub struct Response {
    pub status: u32,
    pub body: Vec<u8>,
    pub credits: Option<i64>,
}
pub fn encoded(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

pub fn json_response(response: Response) -> Result<Value> {
    if response.status != 200 {
        return Err(Error(match response.status {
            401 | 403 => "provider_access_denied",
            429 => "provider_rate_limited",
            _ => "provider_http_error",
        }));
    }
    serde_json::from_slice(&response.body).map_err(|_| Error("invalid_provider_response"))
}

pub fn call(
    provider: &str,
    account: &str,
    method: &str,
    path: &str,
    body: &Value,
) -> Result<Response> {
    if provider == "mock" {
        return mock(method, path, body);
    }
    let connection = credentials(account)?;
    #[cfg(windows)]
    {
        native::http(&connection.key, method, path, body)
    }
    #[cfg(target_os = "linux")]
    {
        crate::linux_provider::http(&connection.key, method, path, body)
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = (connection, method, path, body);
        Err(Error("unsupported_platform"))
    }
}
pub fn credentials(account: &str) -> Result<Connection> {
    if !crate::catalog::valid_id(account) {
        return Err(Error("invalid_account_id"));
    }
    #[cfg(windows)]
    {
        native::read(account)
    }
    #[cfg(target_os = "linux")]
    {
        crate::linux_provider::read(account)
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        Err(Error("credential_manager_unavailable"))
    }
}
pub fn save(
    account: &str,
    key: String,
    enabled: bool,
    request_limit: i64,
    credit_limit: i64,
) -> Result<()> {
    let key = Zeroizing::new(key);
    if !crate::catalog::valid_id(account)
        || key.is_empty()
        || key.len() > 256
        || !key.bytes().all(|b| b.is_ascii_graphic())
    {
        return Err(Error("invalid_credential"));
    }
    if request_limit < 0
        || credit_limit < 0
        || request_limit > 1_000_000
        || credit_limit > 1_000_000_000
        || enabled && (request_limit == 0 || credit_limit == 0)
    {
        return Err(Error("invalid_budget"));
    }
    #[cfg(windows)]
    {
        native::save(account, &key, enabled, request_limit, credit_limit)
    }
    #[cfg(not(windows))]
    {
        Err(Error("credential_manager_unavailable"))
    }
}
pub fn disconnect(account: &str) -> Result<()> {
    if !crate::catalog::valid_id(account) {
        return Err(Error("invalid_account_id"));
    }
    #[cfg(windows)]
    {
        native::delete(account)
    }
    #[cfg(not(windows))]
    {
        Err(Error("credential_manager_unavailable"))
    }
}

#[cfg(test)]
thread_local! {
    static MOCK_FAULT: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
    static MOCK_POSTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
pub(crate) fn mock_fault(fault: u8) {
    MOCK_FAULT.set(fault);
    MOCK_POSTS.set(0);
}
#[cfg(test)]
pub(crate) fn mock_posts() -> usize {
    MOCK_POSTS.get()
}
fn mock(method: &str, path: &str, body: &Value) -> Result<Response> {
    #[cfg(test)]
    if method == "POST" {
        MOCK_POSTS.set(MOCK_POSTS.get() + 1);
        match MOCK_FAULT.get() {
            1 => return Err(Error("provider_transport_error")),
            2 => {
                return Ok(Response {
                    status: 429,
                    body: Vec::new(),
                    credits: None,
                })
            }
            _ => (),
        }
    }
    let value = if path == "/v1/user" {
        json!({"workspace_id":"mock-workspace","subscription":{"tier":"mock","character_count":0,"character_limit":0}})
    } else if path.starts_with("/v2/voices") {
        json!({"voices":[{"voice_id":"mock-voice","name":"Synthetic fixture","category":"mock","preview_url":null}],"has_more":false,"next_page_token":null})
    } else if path == "/v1/models" {
        json!([{"model_id":"mock-model","name":"Synthetic fixture","can_do_text_to_speech":true}])
    } else if path.ends_with("/settings") {
        json!({"stability":0.5,"similarity_boost":0.75,"style":0,"use_speaker_boost":true,"speed":1})
    } else if method == "GET" && path.starts_with("/v1/voices/") {
        json!({"voice_id":"mock-voice","name":"Synthetic fixture","category":"mock","preview_url":null})
    } else if method == "POST" {
        let ms = body["duration_seconds"]
            .as_f64()
            .map(|n| n * 1000.0)
            .or_else(|| body["music_length_ms"].as_f64())
            .unwrap_or(250.0) as usize;
        let samples = (ms * 8).clamp(1, 240_000);
        let mut audio = b"RIFF".to_vec();
        audio.extend(((samples * 2 + 36) as u32).to_le_bytes());
        audio.extend(b"WAVEfmt ");
        audio.extend(16u32.to_le_bytes());
        audio.extend(1u16.to_le_bytes());
        audio.extend(1u16.to_le_bytes());
        audio.extend(8000u32.to_le_bytes());
        audio.extend(16000u32.to_le_bytes());
        audio.extend(2u16.to_le_bytes());
        audio.extend(16u16.to_le_bytes());
        audio.extend(b"data");
        audio.extend(((samples * 2) as u32).to_le_bytes());
        for n in 0..samples {
            let v = if n % 32 < 16 { 500i16 } else { -500i16 };
            audio.extend(v.to_le_bytes());
        }
        return Ok(Response {
            status: 200,
            body: audio,
            credits: Some(1),
        });
    } else {
        return Err(Error("unsupported_mock_endpoint"));
    };
    Ok(Response {
        status: 200,
        body: serde_json::to_vec(&value).unwrap(),
        credits: None,
    })
}

#[cfg(windows)]
mod native {
    use super::*;
    use std::{
        ffi::c_void,
        ptr,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{Networking::WinHttp::*, Security::Credentials::*};
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }
    fn target(account: &str) -> Vec<u16> {
        wide(&format!(
            "DK-Elevenlabs-GameSound-Manager/{account}/connection-v1"
        ))
    }
    struct Credential(*mut CREDENTIALW);
    impl Drop for Credential {
        fn drop(&mut self) {
            unsafe {
                if !self.0.is_null() {
                    let c = &mut *self.0;
                    if !c.CredentialBlob.is_null() {
                        std::slice::from_raw_parts_mut(
                            c.CredentialBlob,
                            c.CredentialBlobSize as usize,
                        )
                        .fill(0);
                    }
                    CredFree(self.0.cast());
                }
            }
        }
    }
    pub fn read(account: &str) -> Result<Connection> {
        unsafe {
            let mut raw = ptr::null_mut();
            if CredReadW(target(account).as_ptr(), CRED_TYPE_GENERIC, 0, &mut raw) == 0 {
                return Err(Error("credential_unavailable"));
            }
            let credential = Credential(raw);
            let c = &*credential.0;
            if c.CredentialBlobSize > 2560 || c.CredentialBlob.is_null() {
                return Err(Error("invalid_credential"));
            }
            let bytes = Zeroizing::new(
                std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize)
                    .to_vec(),
            );
            let mut v: Value =
                serde_json::from_slice(&bytes).map_err(|_| Error("invalid_credential"))?;
            let key = match v["key"].take() {
                Value::String(key) => Zeroizing::new(key),
                _ => return Err(Error("invalid_credential")),
            };
            if key.is_empty() || !key.bytes().all(|b| b.is_ascii_graphic()) {
                return Err(Error("invalid_credential"));
            }
            Ok(Connection {
                key,
                enabled: v["enabled"] == true,
                request_limit: v["request_limit"].as_i64().unwrap_or(0),
                credit_limit: v["credit_limit"].as_i64().unwrap_or(0),
            })
        }
    }
    pub fn save(
        account: &str,
        key: &str,
        enabled: bool,
        request_limit: i64,
        credit_limit: i64,
    ) -> Result<()> {
        let target = target(account);
        let username = wide("DK GameSound");
        let mut blob=Zeroizing::new(serde_json::to_vec(&json!({"key":key,"enabled":enabled,"request_limit":request_limit,"credit_limit":credit_limit})).unwrap());
        let c = CREDENTIALW {
            Type: CRED_TYPE_GENERIC,
            TargetName: target.as_ptr() as *mut _,
            CredentialBlobSize: blob.len() as u32,
            CredentialBlob: blob.as_mut_ptr(),
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            UserName: username.as_ptr() as *mut _,
            ..unsafe { std::mem::zeroed() }
        };
        if unsafe { CredWriteW(&c, 0) } == 0 {
            return Err(Error("credential_write_failed"));
        }
        Ok(())
    }
    pub fn delete(account: &str) -> Result<()> {
        if unsafe { CredDeleteW(target(account).as_ptr(), CRED_TYPE_GENERIC, 0) } == 0
            && unsafe { windows_sys::Win32::Foundation::GetLastError() } != 1168
        {
            return Err(Error("credential_delete_failed"));
        }
        Ok(())
    }
    struct Http(*mut c_void);
    impl Drop for Http {
        fn drop(&mut self) {
            unsafe {
                WinHttpCloseHandle(self.0);
            }
        }
    }
    fn handle(raw: *mut c_void) -> Result<Http> {
        if raw.is_null() {
            Err(Error("provider_transport_error"))
        } else {
            Ok(Http(raw))
        }
    }
    pub fn http(key: &str, method: &str, path: &str, body: &Value) -> Result<Response> {
        if !path.starts_with("/v1/") && !path.starts_with("/v2/") {
            return Err(Error("invalid_provider_path"));
        }
        let agent = wide("DK-GameSound/0.3");
        let host = wide("api.elevenlabs.io");
        let verb = wide(method);
        let path = wide(path);
        let headers = Zeroizing::new(wide(&format!(
            "xi-api-key: {key}\r\nContent-Type: application/json\r\n"
        )));
        let bytes = if method == "POST" {
            serde_json::to_vec(body).map_err(|_| Error("invalid_request"))?
        } else {
            Vec::new()
        };
        unsafe {
            let session = handle(WinHttpOpen(
                agent.as_ptr(),
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                ptr::null(),
                ptr::null(),
                0,
            ))?;
            if WinHttpSetTimeouts(session.0, 10000, 10000, 30000, 60000) == 0 {
                return Err(Error("provider_transport_error"));
            }
            let connection = handle(WinHttpConnect(session.0, host.as_ptr(), 443, 0))?;
            let request = handle(WinHttpOpenRequest(
                connection.0,
                verb.as_ptr(),
                path.as_ptr(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                WINHTTP_FLAG_SECURE,
            ))?;
            let redirects: u32 = WINHTTP_OPTION_REDIRECT_POLICY_NEVER;
            if WinHttpSetOption(
                request.0,
                WINHTTP_OPTION_REDIRECT_POLICY,
                (&redirects as *const u32).cast(),
                4,
            ) == 0
            {
                return Err(Error("provider_transport_error"));
            }
            if WinHttpSendRequest(
                request.0,
                headers.as_ptr(),
                (headers.len() - 1) as u32,
                bytes.as_ptr().cast(),
                bytes.len() as u32,
                bytes.len() as u32,
                0,
            ) == 0
                || WinHttpReceiveResponse(request.0, ptr::null_mut()) == 0
            {
                return Err(Error("provider_transport_error"));
            }
            let mut status = 0u32;
            let mut size = 4;
            if WinHttpQueryHeaders(
                request.0,
                WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                ptr::null(),
                (&mut status as *mut u32).cast(),
                &mut size,
                ptr::null_mut(),
            ) == 0
            {
                return Err(Error("provider_transport_error"));
            }
            let name = wide("character-cost");
            let mut credit = [0u16; 64];
            let mut length = 128;
            let credits = if WinHttpQueryHeaders(
                request.0,
                WINHTTP_QUERY_CUSTOM,
                name.as_ptr(),
                credit.as_mut_ptr().cast(),
                &mut length,
                ptr::null_mut(),
            ) != 0
            {
                String::from_utf16_lossy(
                    &credit[..credit.iter().position(|v| *v == 0).unwrap_or(64)],
                )
                .parse::<i64>()
                .ok()
                .filter(|n| *n >= 0)
            } else {
                None
            };
            // Never read/return provider error bodies, which may echo sensitive data.
            if status != 200 {
                return Ok(Response {
                    status,
                    body: Vec::new(),
                    credits,
                });
            }
            let mut data = Vec::new();
            let mut chunk = [0u8; 16384];
            let deadline = Instant::now() + Duration::from_secs(120);
            loop {
                let mut n = 0;
                if Instant::now() > deadline
                    || WinHttpReadData(
                        request.0,
                        chunk.as_mut_ptr().cast(),
                        chunk.len() as u32,
                        &mut n,
                    ) == 0
                {
                    return Err(Error("provider_transport_error"));
                }
                if n == 0 {
                    break;
                }
                if data.len() + n as usize > 32 * 1024 * 1024 {
                    return Err(Error("provider_response_too_large"));
                }
                data.extend_from_slice(&chunk[..n as usize]);
            }
            Ok(Response {
                status,
                body: data,
                credits,
            })
        }
    }
}
