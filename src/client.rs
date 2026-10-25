use crate::config::Transport;
use gitlab::GitlabBuilder;

pub fn builder(host: String, token: String, transport: Transport) -> GitlabBuilder {
    let mut builder = GitlabBuilder::new(host, token);
    builder.oauth2_token();
    match transport {
        Transport::Https => {}
        Transport::PlainHttp => {
            builder.insecure();
        }
        Transport::HttpsSkipVerify => {
            builder.cert_insecure();
        }
    }
    builder
}

/// Builds the client without any network call: the token is attached by the
/// traced wrapper, so the caller can draw its first frame before the first request.
pub async fn build_deferred(
    host: String,
    token: &str,
    transport: Transport,
) -> Result<crate::trace::TracedClient, String> {
    let mut builder = GitlabBuilder::new_unauthenticated(host);
    match transport {
        Transport::Https => {}
        Transport::PlainHttp => {
            builder.insecure();
        }
        Transport::HttpsSkipVerify => {
            builder.cert_insecure();
        }
    }
    let inner = builder.build_async().await.map_err(|e| e.to_string())?;
    crate::trace::TracedClient::from(inner).with_bearer(token)
}
