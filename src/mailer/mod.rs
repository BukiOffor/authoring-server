use crate::{
    error::ModuleError,
    mailer::{
        config::EmailConfig as Config,
        email::{Email, Receiptent},
    },
};

pub mod config;
pub mod email;

pub mod types;

pub async fn send_otp(
    name: String,
    otp: String,
    to: String,
    subject: &str,
) -> Result<(), ModuleError> {
    let to = Receiptent {
        name: name.clone(),
        email: to.clone(),
    };
    let config = Config::init()?;
    let email = Email::new(to.clone(), config);
    let data = serde_json::json!({
        "name": name,
        "otp": otp,
    });
    tracing::info!("Sending OTP to: {}", to.name);
    email
        .send_email("otp", subject, data)
        .await
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    Ok(())
}
