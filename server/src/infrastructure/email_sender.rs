use crate::{
    domain::services::email_verification::{EmailMessage, EmailSender},
    error::ApiError,
};
use async_trait::async_trait;
use resend_rs::{Resend, types::CreateEmailBaseOptions};

#[derive(Clone, Default)]
pub struct ConsoleEmailSender;

#[async_trait]
impl EmailSender for ConsoleEmailSender {
    async fn send(&self, message: EmailMessage) -> Result<(), ApiError> {
        println!(
            "\n================= EMAIL =================\n\
            From: {}\nTo: {}\nSubject: {}\n------------------------------------------\n\
            {}\n==========================================\n",
            message.from,
            message.to.join(", "),
            message.subject,
            message.text_body.as_deref().unwrap_or(""),
        );
        Ok(())
    }
}

#[derive(Clone)]
pub struct ResendEmailSender {
    client: Resend,
}

impl ResendEmailSender {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Resend::new(api_key),
        }
    }
}

#[async_trait]
impl EmailSender for ResendEmailSender {
    async fn send(&self, message: EmailMessage) -> Result<(), ApiError> {
        let to = message.to.iter().map(String::as_str).collect::<Vec<_>>();
        let mut options = CreateEmailBaseOptions::new(&message.from, to, &message.subject);

        if let Some(text) = message.text_body.clone() {
            options = options.with_text(&text);
        }
        if let Some(html) = message.html_body.clone() {
            options = options.with_html(&html);
        }

        self.client
            .emails
            .send(options)
            .await
            .map_err(|_| ApiError::InternalServerError)?; // Map to your ApiError

        Ok(())
    }
}
