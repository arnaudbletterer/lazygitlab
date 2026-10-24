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
