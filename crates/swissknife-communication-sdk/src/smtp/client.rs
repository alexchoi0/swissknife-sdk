use crate::{Error, Result};
use async_trait::async_trait;
use base64::Engine;

#[cfg(feature = "email")]
use crate::email::{Email, EmailResponse, EmailSender};

pub struct SmtpClient {
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    use_tls: bool,
}

impl SmtpClient {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            username: None,
            password: None,
            use_tls: true,
        }
    }

    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        self
    }

    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }

    pub fn gmail(username: &str, app_password: &str) -> Self {
        Self::new("smtp.gmail.com", 587)
            .with_auth(username, app_password)
    }

    pub fn outlook(username: &str, password: &str) -> Self {
        Self::new("smtp.office365.com", 587)
            .with_auth(username, password)
    }

    pub fn sendgrid(api_key: &str) -> Self {
        Self::new("smtp.sendgrid.net", 587)
            .with_auth("apikey", api_key)
    }

    pub fn mailgun(username: &str, password: &str) -> Self {
        Self::new("smtp.mailgun.org", 587)
            .with_auth(username, password)
    }

    fn build_raw_email(&self, email: &SmtpEmail) -> String {
        let boundary = "boundary_swissknife_smtp";
        let to_header = email.to.join(", ");

        let mut raw = format!(
            "From: {}\r\nTo: {}\r\nSubject: {}\r\nMIME-Version: 1.0\r\n",
            email.from, to_header, email.subject
        );

        if !email.cc.is_empty() {
            raw.push_str(&format!("Cc: {}\r\n", email.cc.join(", ")));
        }

        if let Some(ref reply_to) = email.reply_to {
            raw.push_str(&format!("Reply-To: {}\r\n", reply_to));
        }

        if email.text.is_some() && email.html.is_some() {
            raw.push_str(&format!("Content-Type: multipart/alternative; boundary=\"{}\"\r\n\r\n", boundary));
            if let Some(ref t) = email.text {
                raw.push_str(&format!("--{}\r\nContent-Type: text/plain; charset=\"UTF-8\"\r\n\r\n{}\r\n", boundary, t));
            }
            if let Some(ref h) = email.html {
                raw.push_str(&format!("--{}\r\nContent-Type: text/html; charset=\"UTF-8\"\r\n\r\n{}\r\n", boundary, h));
            }
            raw.push_str(&format!("--{}--", boundary));
        } else if let Some(ref h) = email.html {
            raw.push_str("Content-Type: text/html; charset=\"UTF-8\"\r\n\r\n");
            raw.push_str(h);
        } else if let Some(ref t) = email.text {
            raw.push_str("Content-Type: text/plain; charset=\"UTF-8\"\r\n\r\n");
            raw.push_str(t);
        }

        raw
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    pub fn use_tls(&self) -> bool {
        self.use_tls
    }
}

#[derive(Debug, Clone, Default)]
pub struct SmtpEmail {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub text: Option<String>,
    pub html: Option<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub reply_to: Option<String>,
}

impl SmtpEmail {
    pub fn new(from: &str, to: &str, subject: &str) -> Self {
        Self {
            from: from.to_string(),
            to: vec![to.to_string()],
            subject: subject.to_string(),
            ..Default::default()
        }
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }

    pub fn html(mut self, html: &str) -> Self {
        self.html = Some(html.to_string());
        self
    }

    pub fn add_to(mut self, to: &str) -> Self {
        self.to.push(to.to_string());
        self
    }

    pub fn cc(mut self, cc: &str) -> Self {
        self.cc.push(cc.to_string());
        self
    }

    pub fn bcc(mut self, bcc: &str) -> Self {
        self.bcc.push(bcc.to_string());
        self
    }

    pub fn reply_to(mut self, reply_to: &str) -> Self {
        self.reply_to = Some(reply_to.to_string());
        self
    }
}

#[cfg(feature = "email")]
#[async_trait]
impl EmailSender for SmtpClient {
    async fn send_email(&self, email: &Email) -> Result<EmailResponse> {
        let from = if let Some(ref name) = email.from.name {
            format!("{} <{}>", name, email.from.email)
        } else {
            email.from.email.clone()
        };

        let to: Vec<String> = email.to.iter().map(|a| {
            if let Some(ref name) = a.name {
                format!("{} <{}>", name, a.email)
            } else {
                a.email.clone()
            }
        }).collect();

        let _smtp_email = SmtpEmail {
            from,
            to,
            subject: email.subject.clone(),
            text: email.text.clone(),
            html: email.html.clone(),
            ..Default::default()
        };

        Err(Error::InvalidRequest(
            "SMTP sending requires an async SMTP client. Use lettre crate or similar in your runtime.".to_string()
        ))
    }
}
