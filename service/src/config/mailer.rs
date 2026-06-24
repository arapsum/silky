use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct MailerConfig {
    smtp: SmtpConfig,
}

impl MailerConfig {
    #[must_use]
    pub const fn smtp(&self) -> &SmtpConfig {
        &self.smtp
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SmtpConfig {
    host: String,
    port: u16,
    secure: bool,
    enable: bool,
    auth: Option<MailerAuthConfig>,
}

impl SmtpConfig {
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    #[must_use]
    pub const fn secure(&self) -> bool {
        self.secure
    }

    #[must_use]
    pub const fn enable(&self) -> bool {
        self.enable
    }

    #[must_use]
    pub const fn auth(&self) -> Option<&MailerAuthConfig> {
        self.auth.as_ref()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MailerAuthConfig {
    username: String,
    password: String,
}

impl MailerAuthConfig {
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }
}
