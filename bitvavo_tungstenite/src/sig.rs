use hmac::{Hmac, KeyInit, Mac};
use serde_json::to_string;
use sha2::Sha256;
use std::collections::HashMap;
use std::fmt::Write as _;

pub fn create_signature(
    timestamp: &str,
    method: &str,
    url: &str,
    body: HashMap<String, String>,
    api_secret: &str,
) -> String {
    // Concatenate timestamp, method, and URL
    let mut result = format!("{}{}{}", timestamp, method, "/v2");
    result.push_str(url);

    // If body is not empty, convert it to a JSON string and append it to result
    if !body.is_empty() {
        match to_string(&body) {
            Ok(body_string) => result.push_str(&body_string),
            Err(_) => {
                panic!("Unable to serialize body");
            }
        }
    }
    // Create HMAC-SHA256 using the provided API secret
    let mut mac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(result.as_bytes());

    // Generate the signature and convert it to a hexadecimal string
    let signature = mac.finalize().into_bytes();
    let mut sha = String::new();
    for byte in signature {
        write!(&mut sha, "{:02x}", byte).unwrap();
    }

    sha
}
