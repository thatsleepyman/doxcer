////////////////////////////////////////////////////////
// AUTHOR   : Stefan B. J. Meeuwessen
// COMPANY  : Hoppenbrouwers
// TEAM     : Team Data & BI
// CREATION : 2025-11-05
// VERSION  : 0.1.4
////////////////////////////////////////////////////////


// Compiler Directives
#![allow(unused)]


// Internal Libraries
use std::env;
use std::fs;
use std::process;
use std::path::{Path, PathBuf};

// External Libraries
use dotenvy::from_path;
use fernet::Fernet;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};


// ----------------------------
// Data Structures
// ----------------------------
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    input: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    output: Option<Vec<ChatOutput>>,
}

#[derive(Deserialize)]
struct ChatOutput {
    content: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ChatMessage {
    r#type: String,
    text: Option<String>,
}


// ----------------------------
// Constants
// ----------------------------
const URL: &str = "https://api.openai.com/v1/responses";


// ----------------------------
// Helper Functions
// ----------------------------
fn decrypt_value(encrypted_value: &str, encryption_key: &str) -> Option<String> {

    /// Decrypts an encrypted string using Fernet symmetric encryption.
    ///
    /// # Description
    /// This function takes a Fernet-encrypted, base64-encoded string and decrypts it using
    /// a provided encryption key. It ensures that the resulting value is valid UTF-8.
    ///
    /// # Parameters
    /// * `encrypted_value` – The encrypted string to be decrypted.
    /// * `encryption_key` – The Fernet encryption key, as defined in the environment.
    ///
    /// # Returns
    /// * `Option<String>` – Returns the decrypted UTF-8 string if successful; otherwise `None`.

    let fernet = Fernet::new(encryption_key)?;
    let decrypted_bytes = fernet.decrypt(encrypted_value).ok()?;
    String::from_utf8(decrypted_bytes).ok()
}

fn load_env_robust() {

    /// Loads the environment configuration from a `.env` file using a robust multi-path search.
    ///
    /// # Description
    /// The function searches for a valid `.env` file across multiple potential locations,
    /// including:
    /// - An explicit override path defined by `DOXCER_ENV_PATH`.
    /// - The current working directory and its `config` subdirectory.
    /// - The directory of the executing binary and its parent directories.
    ///
    /// The first valid `.env` file discovered will be loaded into the process environment.
    /// If no configuration file is found, the function panics with a detailed list of
    /// the paths it attempted to locate.
    ///
    /// # Panics
    /// Panics if no `.env` file can be found or if the selected file cannot be read.

    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let explicit = env::var("DOXCER_ENV_PATH").ok().map(PathBuf::from);

    let candidates: Vec<PathBuf> = [
        explicit.as_ref().cloned(),
        Some(cwd.join("config").join(".env")),
        Some(cwd.join(".env")),
        Some(exe_dir.join("config").join(".env")),
        Some(exe_dir.join(".env")),
        exe_dir.parent().map(|p| p.join("config").join(".env")),
        exe_dir.parent().map(|p| p.join(".env")),
        exe_dir.parent().and_then(|p| p.parent()).map(|p| p.join("config").join(".env")),
        exe_dir.parent().and_then(|p| p.parent()).map(|p| p.join(".env")),
    ]
    .into_iter()
    .flatten()
    .collect();

    if let Some(found) = candidates.iter().find(|p| p.exists()) {
        from_path(found)
            .unwrap_or_else(|e| panic!("Failed to load .env at {}: {e}", found.display()));
        eprintln!("Loaded .env from: {}", found.display());
    } else {
        let searched = candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n  - ");
        panic!("Could not find a .env file. Searched:\n  - {}", searched);
    }
}


// ----------------------------
// Runtime
// ----------------------------
fn main() {

    /// Entry point for the Doxcer notebook documentation generator.
    ///
    /// # Description
    /// This function orchestrates the end-to-end workflow of the Doxcer tool:
    /// 1. Loads environment configuration via [`load_env_robust`].
    /// 2. Retrieves and decrypts the OpenAI API key from the environment.
    /// 3. Parses the CLI argument specifying the target Fabric PySpark notebook.
    /// 4. Reads the notebook and the Markdown template (`prompt.md`).
    /// 5. Constructs a prompt for the OpenAI API and sends a documentation generation request.
    /// 6. Outputs the generated documentation to standard output.
    ///
    /// # Usage
    /// ```bash
    /// doxcer <path/to/notebook.py>
    /// ```
    ///
    /// # Panics
    /// The function will panic if:
    /// - The `.env` file or its variables cannot be loaded.
    /// - The notebook or template files cannot be read.
    /// - The OpenAI API request fails unexpectedly.

    load_env_robust();

    let encryption_key = env::var("ENCRYPTION_PASSWORD")
        .expect("Missing ENCRYPTION_PASSWORD in .env");

    let encrypted_api_key = env::var("OPENAI_API_KEY_ENC")
        .expect("Missing OPENAI_API_KEY_ENC in .env");
    let api_key = decrypt_value(&encrypted_api_key, &encryption_key)
        .expect("Failed to decrypt API key");

    let args: Vec<String> = env::args()
        .collect();
    if args.len() != 2 {
        eprintln!("Usage: doxcer <path/to/notebook.py>");
        process::exit(1);
    }

    let file_path = &args[1];
    let notebook_content = fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("Failed to read file {}", file_path));

    let template_path = "./templates/prompt.md";
    let template_content = fs::read_to_string(template_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", template_path));

    println!(
        "Loaded prompt template from: {}\n--- Preview ---\n{}\n--- End of Preview ---\n",
        template_path,
        &template_content.chars()
            .take(250)
            .collect::<String>()
    );

    let prompt = format!("{}\n\nHier is de Notebook.py:\n\n{}", template_content, notebook_content);

    let request = ChatRequest {
        model: "gpt-5-mini".to_string(),
        input: prompt,
    };

    let client = Client::new();
    let response = client
        .post(URL)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send();

    match response {
        Ok(res) => {

            if res.status().is_success() {

                let parsed: ChatResponse = res.json()
                    .unwrap_or(ChatResponse { output: None });
                
                if let Some(outputs) = parsed.output {
                    
                    for o in outputs {
                        for msg in o.content {
                            
                            if let Some(text) = msg.text {
                                println!("{}", text);
                            }
                        }
                    }
                } else {
                    println!("No output received from API.");
                }
            } else {
                eprintln!("API request failed: {}", res.text().unwrap_or_default());
            }
        }
        Err(e) => eprintln!("Request error: {}", e),
    }
}