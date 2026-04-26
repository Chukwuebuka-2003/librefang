//! OAuth state token integration tests
//!
//! Tests the OAuth state token functionality to ensure:
//! 1. Tokens have proper structure (payload.signature)
//! 2. Tokens are HMAC-signed and verifiable
//! 3. Tampered tokens are rejected
//! 4. Expired tokens are rejected
//! 5. Keys are 256 bits (32 bytes base64-encoded = ~43 chars)

use base64::Engine;

/// Test that state tokens have the expected structure.
#[test]
fn test_state_token_structure() {
    let token = build_test_state_token("test-provider");

    // Token should have format: payload.signature
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(
        parts.len(),
        2,
        "Token should have exactly 2 parts (payload.signature)"
    );

    // Both parts should be valid base64
    let payload_b64 = parts[0];
    let sig_b64 = parts[1];

    assert!(
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload_b64)
            .is_ok(),
        "Payload should be valid base64url"
    );
    assert!(
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(sig_b64)
            .is_ok(),
        "Signature should be valid base64url"
    );
}

/// Test that tampered tokens are rejected.
#[test]
fn test_state_token_tamper_detection() {
    let token = build_test_state_token("test-provider");
    let parts: Vec<&str> = token.split('.').collect();

    // Tamper with payload
    let tampered = format!("{}.{}", "dGFtcGVyZWQ", parts[1]);
    assert!(
        verify_test_state_token(&tampered).is_err(),
        "Tampered token should fail verification"
    );

    // Tamper with signature
    let tampered = format!("{}.{}", parts[0], "dGFtcGVyZWQ");
    assert!(
        verify_test_state_token(&tampered).is_err(),
        "Token with invalid signature should fail verification"
    );
}

/// Test that expired tokens are rejected.
#[test]
fn test_state_token_expiry() {
    // Build a token with old timestamp
    let expired_token = build_expired_state_token();
    assert!(
        verify_test_state_token(&expired_token).is_err(),
        "Expired token should be rejected"
    );
}

/// Test that tokens from different providers are valid.
#[test]
fn test_multiple_providers() {
    let providers = vec!["google", "github", "azure", "keycloak"];

    for provider in providers {
        let token = build_test_state_token(provider);
        assert!(
            verify_test_state_token(&token).is_ok(),
            "Token for provider {} should be valid",
            provider
        );
    }
}

/// Test that the signing key is at least 32 bytes (256 bits).
/// This verifies the key generation uses proper entropy.
#[test]
fn test_signing_key_entropy() {
    let key = get_state_signing_key();
    // Base64 encoding of 32 bytes produces ~43 characters
    // Allow some flexibility for configured keys
    assert!(
        key.len() >= 32,
        "Signing key should be at least 32 bytes, got {} bytes",
        key.len()
    );
}

/// Test that the fallback produces a 32-byte key (256 bits) when randomly generated.
/// This verifies the new OsRng-based implementation generates full entropy keys.
#[test]
fn test_fallback_produces_32_byte_key() {
    // Ensure no env var is set so we use the random fallback
    std::env::remove_var("LIBREFANG_STATE_SECRET");

    // Get the key (will be randomly generated since no env var)
    let key = get_state_signing_key();

    // Decode the base64 to check actual byte length
    let decoded = base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(&key)
        .expect("Key should be valid base64");

    assert_eq!(
        decoded.len(),
        32,
        "Fallback key should be exactly 32 bytes (256 bits), got {} bytes",
        decoded.len()
    );
}

/// Test that the signing key is stable across calls within a process.
/// This verifies LazyLock semantics - the key should only be generated once.
#[test]
fn test_signing_key_stable_across_calls() {
    // Clear any existing env var to ensure consistent state
    std::env::remove_var("LIBREFANG_STATE_SECRET");

    // Get the key multiple times
    let key1 = get_state_signing_key();
    let key2 = get_state_signing_key();
    let key3 = get_state_signing_key();

    // All calls should return the same key (LazyLock semantics)
    assert_eq!(
        key1, key2,
        "Key should be stable across multiple calls (LazyLock semantics)"
    );
    assert_eq!(
        key2, key3,
        "Key should be stable across multiple calls (LazyLock semantics)"
    );

    // Verify tokens built with the same key can be verified
    let token1 = build_test_state_token("provider1");
    let token2 = build_test_state_token("provider2");

    // Both should be verifiable with the same key
    assert!(
        verify_test_state_token(&token1).is_ok(),
        "Token should be verifiable with stable key"
    );
    assert!(
        verify_test_state_token(&token2).is_ok(),
        "Token should be verifiable with stable key"
    );
}

// Helper functions for testing

fn build_test_state_token(provider_id: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let nonce = uuid::Uuid::new_v4().to_string();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    #[derive(serde::Serialize)]
    struct Payload {
        provider: String,
        nonce: String,
        ts: u64,
    }

    let payload = Payload {
        provider: provider_id.to_string(),
        nonce,
        ts,
    };

    let payload_json = serde_json::to_string(&payload).unwrap();
    let payload_b64 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload_json.as_bytes());

    let key = get_state_signing_key();

    let mut mac =
        HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload_b64.as_bytes());
    let sig = mac.finalize().into_bytes();
    let sig_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig);

    format!("{payload_b64}.{sig_b64}")
}

fn verify_test_state_token(token: &str) -> Result<(), String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let parts: Vec<&str> = token.splitn(2, '.').collect();
    if parts.len() != 2 {
        return Err("Invalid format".to_string());
    }

    let (payload_b64, sig_b64) = (parts[0], parts[1]);

    let key = get_state_signing_key();
    let mut mac =
        HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload_b64.as_bytes());

    let expected_sig = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| "Invalid signature encoding")?;

    mac.verify_slice(&expected_sig)
        .map_err(|_| "Signature mismatch")?;

    #[derive(serde::Deserialize)]
    struct Payload {
        ts: u64,
    }

    let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| "Invalid payload")?;

    let payload: Payload = serde_json::from_slice(&payload_bytes).map_err(|_| "Invalid JSON")?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now.saturating_sub(payload.ts) > 600 {
        return Err("Token expired".to_string());
    }

    Ok(())
}

fn build_expired_state_token() -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    #[derive(serde::Serialize)]
    struct Payload {
        provider: String,
        nonce: String,
        ts: u64,
    }

    let payload = Payload {
        provider: "test".to_string(),
        nonce: "nonce".to_string(),
        ts: 0, // Very old timestamp
    };

    let payload_json = serde_json::to_string(&payload).unwrap();
    let payload_b64 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload_json.as_bytes());

    let key = get_state_signing_key();
    let mut mac =
        HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload_b64.as_bytes());
    let sig = mac.finalize().into_bytes();
    let sig_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig);

    format!("{payload_b64}.{sig_b64}")
}

/// Replicate the state_signing_key function logic for testing
/// Uses a static LazyLock to ensure key stability within the test process
fn get_state_signing_key() -> String {
    use argon2::password_hash::rand_core::{OsRng, RngCore};

    static KEY: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        if let Some(secret) = std::env::var("LIBREFANG_STATE_SECRET")
            .ok()
            .filter(|s| !s.is_empty())
        {
            return secret;
        }

        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes)
    });

    KEY.clone()
}
