//! Authentication system for nixiedb.aerioncloud.is-local.org

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

const API_BASE: &str = "http://nixiedb.aerioncloud.is-local.org/api";

const REG_KEY_PATH: &str = r"SOFTWARE\Classes\SystemSettings\Auth";
const REG_VALUE_TOKEN: &str = "SessionToken";
const REG_VALUE_EXPIRY: &str = "SessionExpiry";

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

// ── API types ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
    hwid: &'a str,
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

// ── Login flow ─────────────────────────────────────────────────────────────

/// Blocking — call from a background thread only.
pub fn attempt_login(username: String, password: String) {
    set_state(AuthState::Loading);

    let hwid = get_hwid();

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

    // POST /api/login — server burns First_Use → real HWID here if needed
    let resp = client
        .post(format!("{API_BASE}/login"))
        .json(&LoginRequest {
            username: &username,
            password: &password,
            hwid: &hwid,
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
                            Some(token) => verify_and_finalize(token, username, &client),
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

/// Save the token to registry with a Unix timestamp expiry.
/// Token expires in 23h client-side (1h before server-side expiry as a buffer).
fn save_token_to_registry(token: &str) {
    use std::process::Command;

    // Store token
    let _ = Command::new("reg")
        .args([
            "add",
            &format!("HKCU\\{REG_KEY_PATH}"),
            "/v",
            REG_VALUE_TOKEN,
            "/t",
            "REG_SZ",
            "/d",
            token,
            "/f",
        ])
        .output();

    // Store expiry as Unix timestamp string (now + 23 hours)
    let expiry = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + (23 * 60 * 60); // 23 hours

    let _ = Command::new("reg")
        .args([
            "add",
            &format!("HKCU\\{REG_KEY_PATH}"),
            "/v",
            REG_VALUE_EXPIRY,
            "/t",
            "REG_SZ",
            "/d",
            &expiry.to_string(),
            "/f",
        ])
        .output();
}

/// Load and return the saved token if it exists and hasn't expired client-side.
fn load_token_from_registry() -> Option<String> {
    use std::process::Command;

    // Read expiry first
    let expiry_out = Command::new("reg")
        .args([
            "query",
            &format!("HKCU\\{REG_KEY_PATH}"),
            "/v",
            REG_VALUE_EXPIRY,
        ])
        .output()
        .ok()?;

    let expiry_str = String::from_utf8_lossy(&expiry_out.stdout);
    let expiry_ts: u64 = expiry_str
        .lines()
        .find(|l| l.contains(REG_VALUE_EXPIRY))?
        .split_whitespace()
        .last()?
        .parse()
        .ok()?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Client-side expiry check before even hitting the network
    if now >= expiry_ts {
        clear_token_from_registry();
        return None;
    }

    // Read token
    let token_out = Command::new("reg")
        .args([
            "query",
            &format!("HKCU\\{REG_KEY_PATH}"),
            "/v",
            REG_VALUE_TOKEN,
        ])
        .output()
        .ok()?;

    let token_str = String::from_utf8_lossy(&token_out.stdout);
    let token = token_str
        .lines()
        .find(|l| l.contains(REG_VALUE_TOKEN))?
        .split_whitespace()
        .last()?
        .to_string();

    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

/// Delete the saved session from registry (on expiry, ban, or HWID mismatch).
pub fn clear_token_from_registry() {
    use std::process::Command;
    let _ = Command::new("reg")
        .args(["delete", &format!("HKCU\\{REG_KEY_PATH}"), "/f"])
        .output();
}

/// POST /api/verify to confirm the token is valid and pull the username back.
/// By this point the server has already committed the HWID update.
fn verify_and_finalize(token: String, username: String, client: &reqwest::blocking::Client) {
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
                        // Save to registry so next launch skips login
                        save_token_to_registry(&token);

                        *auth_token().lock().unwrap() = Some(token);
                        let verified_username = json.username.unwrap_or(username);
                        set_state(AuthState::Authenticated {
                            username: verified_username,
                        });
                    } else {
                        // Token invalid/expired/banned — wipe saved session
                        clear_token_from_registry();
                        set_state(AuthState::Failed(json.error.unwrap_or_else(|| {
                            format!("Verify failed (HTTP {})", status.as_u16())
                        })));
                    }
                }
            }
        }
    }
}

/// Try to resume a saved session from registry without showing the login form.
/// Call this once from main_thread before the render loop starts.
pub fn try_resume_session() {
    let token = match load_token_from_registry() {
        Some(t) => t,
        None => return, // nothing saved or expired, stay Idle → show login form
    };

    set_state(AuthState::Loading);

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => {
            set_state(AuthState::Idle);
            return;
        }
    };

    // Re-verify the saved token against the server
    // This catches: server-side expiry, bans, HWID resets
    verify_and_finalize(token, String::new(), &client);

    // If verify failed, drop back to Idle so the login form appears
    if matches!(get_state(), AuthState::Failed(_)) {
        set_state(AuthState::Idle);
    }
}
