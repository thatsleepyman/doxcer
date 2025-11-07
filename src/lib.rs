////////////////////////////////////////////////////////
// AUTHOR   : Stefan B. J. Meeuwessen
// CREATION : 2025-11-07
// VERSION  : 0.1.1
////////////////////////////////////////////////////////


// Compiler Directives
#![allow(unused)]


// Internal Libraries
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

// External Libraries
use dotenvy::from_path;
use fernet::Fernet;


// ====================================================
//  Fernet Decryption
// ====================================================
pub fn decrypt_fernet(encrypted_value_b64: &str, fernet_key: &str) -> Result<String, String> {

    // Decrypts a Fernet-encrypted, base64-encoded payload into a UTF-8 `String`.
    //
    // # Description
    // Uses the `fernet` crate to decrypt a base64-encoded token with the provided key.
    // Returns the plaintext as UTF-8.
    //
    // # Parameters
    // * `encrypted_value_b64` – The encrypted string (base64-encoded).
    // * `fernet_key` – The Fernet encryption key (URL-safe base64).
    //
    // # Returns
    // * `Ok(String)` on successful decryption.
    // * `Err(String)` if the key/ciphertext is invalid or not UTF-8.

    let fernet = Fernet::new(fernet_key).ok_or_else(|| "Invalid Fernet key".to_string())?;
    let decrypted = fernet
        .decrypt(encrypted_value_b64)
        .map_err(|_| "Decryption failed".to_string())?;
    String::from_utf8(decrypted).map_err(|_| "Decrypted bytes were not valid UTF-8".to_string())
}


// ====================================================
//  Environment Loading
// ====================================================
pub fn load_env_robust<P: AsRef<Path>>(override_path: Option<P>) -> Result<PathBuf, String> {

    // Loads a `.env` file from multiple potential locations, in priority order.
    //
    // # Description
    // Searches for a valid `.env` file across:
    // 1. Explicit `override_path` argument (if provided)
    // 2. `DOXCER_ENV_PATH` environment variable
    // 3. Current working directory and its `config` subfolder
    // 4. Executable directory and its parent directories
    //
    // Returns the path that was successfully loaded, or an error if none found.

    let override_path = override_path.map(|p| p.as_ref().to_path_buf());
    let explicit_env = env::var("DOXCER_ENV_PATH").ok().map(PathBuf::from);

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    let exe_parent = exe_dir.parent().map(|p| p.to_path_buf());
    let exe_grandparent = exe_parent.as_ref().and_then(|p| p.parent().map(|p| p.to_path_buf()));

    let candidates_in = |root: &Path| -> [PathBuf; 2] {
        [root.join("config").join(".env"), root.join(".env")]
    };

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(p) = override_path { candidates.push(p); }
    if let Some(p) = explicit_env { candidates.push(p); }

    candidates.extend(candidates_in(&cwd));
    candidates.extend(candidates_in(&exe_dir));
    if let Some(p) = &exe_parent { candidates.extend(candidates_in(p)); }
    if let Some(p) = &exe_grandparent { candidates.extend(candidates_in(p)); }

    let tried = candidates.clone();

    if let Some(found) = candidates.into_iter().find(|p| p.exists()) {
        from_path(&found)
            .map_err(|e| format!("Failed to load .env at {}: {e}", found.display()))?;
        Ok(found)
    } else {
        let searched = tried
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n  - ");
        Err(format!("Could not find a .env file. Searched:\n  - {}", searched))
    }
}


// ====================================================
//  Environment Helpers
// ====================================================
pub fn env_plain(var: &str) -> Result<String, String> {

    // Fetches an environment variable as plaintext.
    //
    // # Description
    // Retrieves the environment variable value directly without decryption.
    // Fails if the variable is missing.
    //
    // # Parameters
    // * `var` – The name of the environment variable.
    //
    // # Returns
    // * `Ok(String)` containing the variable value.
    // * `Err(String)` if the variable is not found.

    env::var(var).map_err(|_| format!("Missing required env var: {var}"))
}


pub fn env_secret(name: &str, key_override: Option<&str>) -> Result<String, String> {

    // Retrieves an environment secret, supporting both plaintext and encrypted values.
    //
    // # Description
    // The function checks for the following variables:
    // - `{name}` → returned as plaintext if found.
    // - `{name}_ENC` → decrypted using Fernet with either:
    //   - the provided `key_override`, or
    //   - the `ENCRYPTION_PASSWORD` environment variable.
    //
    // # Parameters
    // * `name` – The base name of the environment variable.
    // * `key_override` – Optional Fernet key to override `ENCRYPTION_PASSWORD`.
    //
    // # Returns
    // * `Ok(String)` containing the secret.
    // * `Err(String)` if the variable is missing or decryption fails.

    if let Ok(v) = env::var(name) {
        return Ok(v);
    }

    let enc_name = format!("{name}_ENC");
    let enc = env::var(&enc_name)
        .map_err(|_| format!("Neither {name} nor {enc_name} found in environment"))?;

    let key = if let Some(k) = key_override {
        k.to_string()
    } else {
        env::var("ENCRYPTION_PASSWORD")
            .map_err(|_| "Missing ENCRYPTION_PASSWORD for Fernet decryption".to_string())?
    };

    decrypt_fernet(&enc, &key)
}


pub fn env_fernet_key() -> Result<String, String> {

    // Retrieves and validates the Fernet key from the environment.
    //
    // # Description
    // Ensures that `ENCRYPTION_PASSWORD` is set and structurally valid as a Fernet key.
    //
    // # Returns
    // * `Ok(String)` containing the valid Fernet key.
    // * `Err(String)` if the key is missing or invalid.

    let key = env::var("ENCRYPTION_PASSWORD")
        .map_err(|_| "Missing ENCRYPTION_PASSWORD".to_string())?;
    Fernet::new(&key).ok_or_else(|| "ENCRYPTION_PASSWORD is not a valid Fernet key".to_string())?;
    Ok(key)
}


pub fn env_path_opt(var: &str) -> Result<Option<PathBuf>, String> {

    // Resolves an optional path-like environment variable into a `PathBuf`.
    //
    // # Description
    // If the variable is set, returns its value as a `PathBuf`.
    // If not set, returns `Ok(None)`. Fails on invalid Unicode.
    //
    // # Parameters
    // * `var` – The name of the environment variable.
    //
    // # Returns
    // * `Ok(Some(PathBuf))` if the variable exists.
    // * `Ok(None)` if the variable is not present.
    // * `Err(String)` if the variable contains invalid Unicode.

    match env::var(var) {
        Ok(v) => Ok(Some(PathBuf::from(v))),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(format!("{var} contains non-unicode data")),
    }
}


pub fn is_dotenv_name<S: AsRef<OsStr>>(name: S) -> bool {

    // Checks whether a given file name equals `.env` (case-sensitive).
    //
    // # Parameters
    // * `name` – The file name to check.
    //
    // # Returns
    // * `true` if the file name is `.env`, otherwise `false`.

    name.as_ref() == ".env"
}