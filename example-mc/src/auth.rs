//! Authentication system for nixiedb.aerioncloud.is-local.org

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

const API_BASE: &str = "https://nixiedb.aerioncloud.is-local.org/api";

const REG_KEY_PATH: &str = r"SOFTWARE\Classes\SystemSettings\Auth";
const REG_VALUE_TOKEN: &str = "SessionToken";
const REG_VALUE_EXPIRY: &str = "SessionExpiry";
const REG_VALUE_USERNAME: &str = "SessionUsername";
const REG_VALUE_PASSWORD: &str = "SessionPassword";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthState {
    Idle,
    Loading,
    Authenticated { username: String },
    Failed(String),
}

static AUTH_STATE: OnceCell<Mutex<AuthState>> = OnceCell::new();
static AUTH_TOKEN: OnceCell<Mutex<Option<String>>> = OnceCell::new();

fn auth_state() -> &'static Mutex<AuthState> {
    AUTH_STATE.get_or_init(|| Mutex::new(AuthState::Idle))
}

fn auth_token() -> &'static Mutex<Option<String>> {
    AUTH_TOKEN.get_or_init(|| Mutex::new(None))
}

pub fn get_state() -> AuthState {
    auth_state().lock().unwrap().clone()
}

pub fn is_authenticated() -> bool {
    matches!(get_state(), AuthState::Authenticated { .. })
}

pub fn get_username() -> Option<String> {
    match get_state() {
        AuthState::Authenticated { username } => Some(username),
        _ => None,
    }
}

fn set_state(state: AuthState) {
    *auth_state().lock().unwrap() = state;
}

// ── HWID ───────────────────────────────────────────────────────────────────

pub fn get_hwid() -> String {
    use std::process::Command;

    let output = Command::new("reg")
        .args([
            "query",
            r"HKLM\SOFTWARE\Microsoft\Cryptography",
            "/v",
            "MachineGuid",
        ])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            for line in stdout.lines() {
                if line.contains("MachineGuid") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(guid) = parts.last() {
                        return guid.to_string();
                    }
                }
            }
            "unknown-hwid".to_string()
        }
        Err(_) => "unknown-hwid".to_string(),
    }
}

// ── Hardware info ──────────────────────────────────────────────────────────

use wmi::{COMLibrary, WMIConnection};

#[derive(Deserialize)]
#[serde(rename = "Win32_VideoController")]
#[serde(rename_all = "PascalCase")]
struct VideoController {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename = "Win32_PhysicalMemory")]
#[serde(rename_all = "PascalCase")]
struct PhysicalMemory {
    capacity: u64,
}

#[derive(Deserialize)]
#[serde(rename = "Win32_Processor")]
#[serde(rename_all = "PascalCase")]
struct Processor {
    name: String,
}

pub fn get_gpu() -> String {
    let Ok(com) = COMLibrary::new() else {
        return "unknown-gpu".to_string();
    };
    let Ok(wmi) = WMIConnection::new(com) else {
        return "unknown-gpu".to_string();
    };
    let Ok(results): Result<Vec<VideoController>, _> = wmi.query() else {
        return "unknown-gpu".to_string();
    };
    results
        .into_iter()
        .next()
        .map(|v| v.name)
        .unwrap_or_else(|| "unknown-gpu".to_string())
}

pub fn get_ram() -> String {
    let Ok(com) = COMLibrary::new() else {
        return "unknown-ram".to_string();
    };
    let Ok(wmi) = WMIConnection::new(com) else {
        return "unknown-ram".to_string();
    };
    let Ok(results): Result<Vec<PhysicalMemory>, _> = wmi.query() else {
        return "unknown-ram".to_string();
    };
    let total_bytes: u64 = results.iter().map(|m| m.capacity).sum();
    if total_bytes == 0 {
        return "unknown-ram".to_string();
    }
    let gb = (total_bytes as f64 / 1_073_741_824.0).round() as u64;
    format!("{gb} GB")
}

pub fn get_cpu() -> String {
    let Ok(com) = COMLibrary::new() else {
        return "unknown-cpu".to_string();
    };
    let Ok(wmi) = WMIConnection::new(com) else {
        return "unknown-cpu".to_string();
    };
    let Ok(results): Result<Vec<Processor>, _> = wmi.query() else {
        return "unknown-cpu".to_string();
    };
    results
        .into_iter()
        .next()
        .map(|p| p.name)
        .unwrap_or_else(|| "unknown-cpu".to_string())
}

// ── API types ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
    hwid: &'a str,
    gpu: &'a str,
    ram: &'a str,
    cpu: &'a str,
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
    token: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize, Debug)]
struct VerifyResponse {
    valid: Option<bool>,
    username: Option<String>,
    error: Option<String>,
}

// ── Registry helpers ───────────────────────────────────────────────────────

fn reg_write(value: &str, data: &str) {
    use std::process::Command;
    let _ = Command::new("reg")
        .args([
            "add",
            &format!("HKCU\\{REG_KEY_PATH}"),
            "/v",
            value,
            "/t",
            "REG_SZ",
            "/d",
            data,
            "/f",
        ])
        .output();
}

fn reg_read(value: &str) -> Option<String> {
    use std::process::Command;
    let out = Command::new("reg")
        .args(["query", &format!("HKCU\\{REG_KEY_PATH}"), "/v", value])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let result = stdout
        .lines()
        .find(|l| l.contains(value))?
        .split_whitespace()
        .last()?
        .to_string();
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Persist token, credentials, and expiry so the next launch can resume
/// without showing the login form — and can re-POST to /api/login to keep
/// HWID + hardware in sync on the server.
fn save_session_to_registry(token: &str, username: &str, password: &str) {
    reg_write(REG_VALUE_TOKEN, token);
    reg_write(REG_VALUE_USERNAME, username);
    reg_write(REG_VALUE_PASSWORD, password);

    // 23 h client-side expiry — 1 h buffer before the server's 24 h JWT expiry
    let expiry = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + (23 * 60 * 60);
    reg_write(REG_VALUE_EXPIRY, &expiry.to_string());
}

/// Returns `(token, username, password)` if a non-expired session exists.
fn load_session_from_registry() -> Option<(String, String, String)> {
    let expiry_ts: u64 = reg_read(REG_VALUE_EXPIRY)?.parse().ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now >= expiry_ts {
        clear_session_from_registry();
        return None;
    }

    let username = reg_read(REG_VALUE_USERNAME)?;
    let password = reg_read(REG_VALUE_PASSWORD)?;
    let token = reg_read(REG_VALUE_TOKEN)?;

    Some((token, username, password))
}

/// Wipe the entire saved session (expiry, ban, HWID mismatch, or logout).
pub fn clear_session_from_registry() {
    use std::process::Command;
    let _ = Command::new("reg")
        .args(["delete", &format!("HKCU\\{REG_KEY_PATH}"), "/f"])
        .output();
}

// Backwards-compat alias for any existing call sites.
pub use clear_session_from_registry as clear_token_from_registry;

// ── Login flow ─────────────────────────────────────────────────────────────

/// Blocking — call from a background thread only.
pub fn attempt_login(username: String, password: String) {
    set_state(AuthState::Loading);
    login_with_credentials(&username, &password);
}

/// Core login logic shared by `attempt_login` and `try_resume_session`.
///
/// Always hits POST /api/login so the server receives the current HWID and
/// hardware info on every launch — this is what burns "First_Use" and keeps
/// hardware records up to date even on resumed sessions.
fn login_with_credentials(username: &str, password: &str) {
    let hwid = get_hwid();
    let gpu = get_gpu();
    let ram = get_ram();
    let cpu = get_cpu();

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            set_state(AuthState::Failed(format!("HTTP client error: {e}")));
            return;
        }
    };

    let resp = client
        .post(format!("{API_BASE}/login"))
        .json(&LoginRequest {
            username,
            password,
            hwid: &hwid,
            gpu: &gpu,
            ram: &ram,
            cpu: &cpu,
        })
        .send();

    match resp {
        Err(e) => {
            set_state(AuthState::Failed(format!("Network error: {e}")));
        }
        Ok(r) => {
            let status = r.status();
            match r.json::<LoginResponse>() {
                Err(e) => {
                    set_state(AuthState::Failed(format!("Parse error: {e}")));
                }
                Ok(json) => {
                    if status.is_success() {
                        match json.token {
                            Some(token) => {
                                verify_and_finalize(token, username.to_string(), password, &client)
                            }
                            None => set_state(AuthState::Failed(
                                json.error.unwrap_or_else(|| "No token in response".into()),
                            )),
                        }
                    } else {
                        // 401 = bad credentials, 403 = banned or HWID mismatch
                        set_state(AuthState::Failed(json.error.unwrap_or_else(|| {
                            format!("Login failed (HTTP {})", status.as_u16())
                        })));
                    }
                }
            }
        }
    }
}

/// POST /api/verify, then persist the full session and set Authenticated state.
fn verify_and_finalize(
    token: String,
    username: String,
    password: &str,
    client: &reqwest::blocking::Client,
) {
    #[derive(Serialize)]
    struct VerifyReq<'a> {
        token: &'a str,
    }

    let resp = client
        .post(format!("{API_BASE}/verify"))
        .json(&VerifyReq { token: &token })
        .send();

    match resp {
        Err(e) => {
            set_state(AuthState::Failed(format!("Verify network error: {e}")));
        }
        Ok(r) => {
            let status = r.status();
            match r.json::<VerifyResponse>() {
                Err(e) => {
                    set_state(AuthState::Failed(format!("Verify parse error: {e}")));
                }
                Ok(json) => {
                    if status.is_success() && json.valid == Some(true) {
                        // Save token + credentials so next launch can re-login
                        save_session_to_registry(&token, &username, password);

                        *auth_token().lock().unwrap() = Some(token);
                        let verified_username = json.username.unwrap_or(username);
                        set_state(AuthState::Authenticated {
                            username: verified_username,
                        });
                    } else {
                        // Token invalid / expired / banned — wipe saved session
                        clear_session_from_registry();
                        set_state(AuthState::Failed(json.error.unwrap_or_else(|| {
                            format!("Verify failed (HTTP {})", status.as_u16())
                        })));
                    }
                }
            }
        }
    }
}

/// Try to resume a saved session without showing the login form.
///
/// Unlike the old approach (which only called /api/verify with the cached
/// token), this always re-POSTs to /api/login with the saved credentials so
/// the server receives the current HWID and hardware on every launch.
/// Call this once from the main thread before the render loop starts.
pub fn try_resume_session() {
    let (_token, username, password) = match load_session_from_registry() {
        Some(s) => s,
        None => return, // nothing saved or expired → stay Idle → show login form
    };

    set_state(AuthState::Loading);
    login_with_credentials(&username, &password);

    // If login failed for any reason, drop back to Idle so the login form appears
    if matches!(get_state(), AuthState::Failed(_)) {
        set_state(AuthState::Idle);
    }
}
