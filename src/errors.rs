pub fn friendly_error(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    let hint = if lower.contains("(401") || lower.contains("unauthorized") {
        "authentication failed (401): the token is invalid, expired or revoked. \
Create a new one with the read_api scope and update GITLAB_TOKEN or config.toml"
    } else if lower.contains("(403") || lower.contains("forbidden") {
        "permission denied (403): the token lacks a required scope (read_api, or api for actions) \
or your role does not allow this"
    } else if lower.contains("(429") || lower.contains("rate limit") {
        "rate limited by GitLab (429): wait a moment, the next refresh will retry"
    } else if lower.contains("certificate")
        || lower.contains("unknownissuer")
        || lower.contains("invalid peer")
    {
        "TLS certificate not trusted: add your CA certificate to the system trust store. \
As a temporary last resort, --insecure skips verification and exposes your token to interception"
    } else if lower.contains("internal server error 5") {
        "GitLab reported a server error (5xx): try again in a moment"
    } else if lower.contains("could not parse json response")
        || lower.contains("internal server error 2")
    {
        "unexpected response: check the GitLab URL; if GitLab is served under a path prefix, \
include it (for example https://host/gitlab)"
    } else if lower.contains("moved permanently") {
        "the server redirected the request: use the https:// form of the URL, including any path prefix"
    } else if lower.contains("error sending request")
        || lower.contains("dns error")
        || lower.contains("connection refused")
        || lower.contains("trying to connect")
    {
        "cannot reach GitLab: check your network connection and the URL"
    } else {
        return raw.to_string();
    };
    format!("{hint} [{}]", crate::redaction::redact_secrets(raw))
}
