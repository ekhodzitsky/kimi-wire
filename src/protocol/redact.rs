use serde_json::Value;
use std::borrow::Cow;
use std::sync::LazyLock;

const REDACTED_SECRET: &str = "[REDACTED]";

/// Best-effort value-shape redaction patterns.
///
/// Key-based redaction in [`is_sensitive_key`] is the primary defense.
/// The patterns below are defense in depth for values that leak through
/// nested or unstructured fields.
static SECRET_VALUE_PATTERNS: LazyLock<Vec<regex::Regex>> = LazyLock::new(|| {
    let patterns: &[&str] = &[
        // GitHub personal-access / OAuth / refresh tokens.
        r"\bgh[pousr]_[A-Za-z0-9]{20,}\b",
        // AWS access key id.
        r"\bAKIA[0-9A-Z]{16}\b",
        // Slack bot/user/app/refresh tokens.
        r"\bxox[abprs]-[A-Za-z0-9-]{10,}\b",
        // Stripe live/test secret keys.
        r"\bsk_(?:live|test)_[A-Za-z0-9]{16,}\b",
        // Generic Bearer-token-shaped fragments.
        r"(?i)\bBearer\s+[A-Za-z0-9._~+/=-]{20,}\b",
        // PEM private key block markers.
        r"-----BEGIN [A-Z ]*PRIVATE KEY-----",
    ];
    patterns
        .iter()
        .map(|&p| regex::Regex::new(p).expect("static regex is valid"))
        .collect()
});

/// Scrub known secret patterns from a string.
pub fn scrub_secret_patterns(input: &str) -> Cow<'_, str> {
    let mut current: Cow<'_, str> = Cow::Borrowed(input);
    for re in SECRET_VALUE_PATTERNS.iter() {
        match re.replace_all(current.as_ref(), REDACTED_SECRET) {
            Cow::Borrowed(_) => {}
            Cow::Owned(new) => current = Cow::Owned(new),
        }
    }
    current
}

/// Redact sensitive fields in JSON payloads while preserving structure.
pub fn redact_secrets(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut redacted = serde_json::Map::with_capacity(map.len());
            for (key, entry) in map {
                if is_sensitive_key(key) {
                    redacted.insert(key.clone(), Value::String(REDACTED_SECRET.to_string()));
                } else {
                    redacted.insert(key.clone(), redact_secrets(entry));
                }
            }
            Value::Object(redacted)
        }
        Value::Array(items) => Value::Array(items.iter().map(redact_secrets).collect()),
        Value::String(s) => match scrub_secret_patterns(s) {
            Cow::Borrowed(_) => value.clone(),
            Cow::Owned(scrubbed) => Value::String(scrubbed),
        },
        _ => value.clone(),
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "api_key" | "apikey" | "token" | "authorization" | "password" | "secret"
    ) || lower.ends_with("_token")
        || lower.ends_with("-token")
        || lower.ends_with("_secret")
        || lower.ends_with("-secret")
        || lower.contains("authorization")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_redact_recursive() {
        let raw = json!({
            "api_key": "abc123",
            "nested": {
                "token": "tok123",
                "headers": { "authorization": "Bearer abc" },
                "token_usage": 42
            },
            "items": [{"password": "pass1"}, {"safe": "value"}]
        });
        let redacted = redact_secrets(&raw);
        assert_eq!(redacted["api_key"], REDACTED_SECRET);
        assert_eq!(redacted["nested"]["token"], REDACTED_SECRET);
        assert_eq!(redacted["nested"]["headers"]["authorization"], REDACTED_SECRET);
        assert_eq!(redacted["nested"]["token_usage"], 42);
        assert_eq!(redacted["items"][0]["password"], REDACTED_SECRET);
        assert_eq!(redacted["items"][1]["safe"], "value");
    }

    #[test]
    fn test_redact_value_patterns() {
        let github_pat = ["ghp", "_", "abcdefghijklmnop1234567890abcdef0011"].concat();
        let aws_key = ["AKIA", "ABCDEFGHIJKLMNOP"].concat();
        let slack_token = ["xoxb", "-", "1234567890-abcdefghij1"].concat();
        let stripe_key = ["sk_live", "_", "abcdefghij1234567890ABCD"].concat();
        let bearer = ["Bearer", " ", "abcdef0123456789abcdef0123456789"].concat();
        let pem = "-----BEGIN RSA PRIVATE KEY-----".to_string();

        let raw = json!({
            "transcript": [
                format!("leaked github pat {github_pat} in env"),
                format!("old aws key was {aws_key} and is rotated"),
                format!("slack hook {slack_token} expired"),
                format!("stripe payload {stripe_key} used in tests"),
                format!("header value: Authorization: {bearer}"),
                format!("pem block {pem} payload"),
            ]
        });
        let redacted = redact_secrets(&raw);
        let transcript = redacted["transcript"].as_array().unwrap();
        assert_eq!(transcript[0], "leaked github pat [REDACTED] in env");
        assert_eq!(transcript[1], "old aws key was [REDACTED] and is rotated");
        assert_eq!(transcript[2], "slack hook [REDACTED] expired");
        assert_eq!(transcript[3], "stripe payload [REDACTED] used in tests");
        assert_eq!(transcript[4], "header value: Authorization: [REDACTED]");
        assert_eq!(transcript[5], "pem block [REDACTED] payload");
    }

    #[test]
    fn test_redact_preserves_benign_strings() {
        let raw = json!({
            "summary": "rotated the github token quarterly",
            "url": "https://docs.example.com/auth/api-key.html",
            "code": "let token = std::env::var(\"GITHUB_TOKEN\");"
        });
        let redacted = redact_secrets(&raw);
        assert_eq!(redacted["summary"], "rotated the github token quarterly");
        assert_eq!(redacted["url"], "https://docs.example.com/auth/api-key.html");
        assert_eq!(redacted["code"], "let token = std::env::var(\"GITHUB_TOKEN\");");
    }

    #[test]
    fn test_redact_idempotent() {
        let github_pat = ["ghp", "_", "abcdefghijklmnop1234567890abcdef0011"].concat();
        let once = redact_secrets(&json!({ "msg": github_pat }));
        let twice = redact_secrets(&once);
        assert_eq!(once, twice);
        assert_eq!(once["msg"], REDACTED_SECRET);
    }
}
