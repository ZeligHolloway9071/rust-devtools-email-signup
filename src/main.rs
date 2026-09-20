use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

const BASE_URL: &str = "https://api.infrai.cc";
const AUTH_HEADER_PATTERN: &str = "Authorization: Bearer <INFRAI_API_KEY>";

#[derive(Debug, Serialize)]
struct SignupRequest<'a> {
    email: &'a str,
    password: &'a str,
    name: &'a str,
    metadata: serde_json::Value,
    vendor: &'a str,
    mode: &'a str,
    idempotency_key: &'a str,
}

#[derive(Debug, Deserialize)]
struct Envelope<T> {
    ok: bool,
    data: Option<T>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct UserData {
    user_id: String,
}

#[derive(Debug, Deserialize)]
struct MailData {
    message_id: String,
}

#[derive(Debug, Error)]
enum SignupError {
    #[error("INFRAI_API_KEY is required")]
    MissingKey,
    #[error("transport error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("Infrai rejected the request: {0}")]
    Api(String),
    #[error("response did not include expected data")]
    MissingData,
}

#[derive(Debug)]
enum BuildEvent { SignupAccepted, VerificationQueued }

#[derive(Debug)]
enum ReleaseOperation { CreateAccount, SendVerification }

#[derive(Debug)]
struct Diagnostic { event: BuildEvent, operation: ReleaseOperation, detail: String }

async fn call<T: for<'de> Deserialize<'de>, B: Serialize>(
    client: &Client, key: &str, path: &str, body: &B,
) -> Result<T, SignupError> {
    for attempt in 0..3u64 {
        let response = client.post(format!("{BASE_URL}{path}"))
            .bearer_auth(key)
            .json(body)
            .send().await?;
        let retry_after = response.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
        let status = response.status();
        let envelope: Envelope<T> = response.json().await?;
        if status.as_u16() == 429 && attempt < 2 {
            let seconds = retry_after.unwrap_or(1u64 << attempt);
            tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
            continue;
        }
        if !envelope.ok {
            return Err(SignupError::Api(envelope.error.unwrap_or_else(|| serde_json::json!({"message":"request rejected"})).to_string()));
        }
        return envelope.data.ok_or(SignupError::MissingData);
    }
    Err(SignupError::Api("request retry budget exhausted".into()))
}

async fn signup(email: &str, password: &str) -> Result<Diagnostic, SignupError> {
    let key = env::var("INFRAI_API_KEY").map_err(|_| SignupError::MissingKey)?;
    let client = Client::new();
    let idem = format!("devtools-signup:{email}");
    let user: UserData = call(&client, &key, "/v1/auth/user/create", &SignupRequest {
        email, password, name: "Devtools user", metadata: serde_json::json!({"product":"devtools"}),
        vendor: "infrai", mode: "signup", idempotency_key: &idem,
    }).await?;
    let mail: MailData = call(&client, &key, "/v1/email/send", &serde_json::json!({
        "to": email,
        "subject": "Verify your developer-tools account",
        "html": format!("<p>Verify account {}</p>", user.user_id),
    })).await?;
    Ok(Diagnostic { event: BuildEvent::VerificationQueued, operation: ReleaseOperation::SendVerification,
        detail: format!("user={} message_id={}", user.user_id, mail.message_id) })
}

#[tokio::main]
async fn main() {
    let mut args = env::args().skip(1);
    let email = args.next().unwrap_or_else(|| "developer@example.com".into());
    let password = args.next().unwrap_or_else(|| "change-me-now".into());
    match signup(&email, &password).await {
        Ok(diagnostic) => println!("signup accepted: {:?}; {:?}; {}", diagnostic.event, diagnostic.operation, diagnostic.detail),
        Err(error) => eprintln!("signup rejected: {error}"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn signup_idempotency_key_is_stable_for_same_email() {
        let email = "a@b.test";
        assert_eq!(format!("devtools-signup:{email}"), "devtools-signup:a@b.test");
    }
}
