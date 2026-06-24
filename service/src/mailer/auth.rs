use serde_json::json;

use crate::{AppContext, error::MailerResult, mailer::MailerError, models::User};

use super::{Email, HandlebarsTemplate, MAILER_TEMPLATES, Mailer};

pub struct AuthMailer {
    renderer: HandlebarsTemplate,
}

impl Mailer for AuthMailer {}

impl AuthMailer {
    /// Initializes a new `AuthMailer` instance with the provided renderer.
    ///
    /// # Arguments
    ///
    /// * `renderer` - A reference to the `HandlebarsTemplate` used for rendering email templates.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The `MAILER_TEMPLATES` lazy lock fails to initialize.
    /// - The `HandlebarsTemplate` fails to clone.
    pub fn init() -> MailerResult<Self, &'static MailerError> {
        let renderer = MAILER_TEMPLATES.as_ref()?;

        Ok(Self {
            renderer: renderer.clone(),
        })
    }

    /// Function to send a welcoming email to new users.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The email was sent successfully.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The `MAILER_TEMPLATES` lazy lock fails to initialize.
    /// - The `HandlebarsTemplate` fails to clone.
    /// - The user's verification token hash is `None`.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The `MAILER_TEMPLATES` lazy lock fails to initialize.
    /// - The `HandlebarsTemplate` fails to clone.
    pub async fn send_welcome(ctx: &AppContext, user: &User) -> MailerResult<()> {
        let this = Self::init()?;

        if user.verification_token_hash().is_none() {
            return Err(MailerError::MissingVariable);
        }

        let rendered = this.renderer.render_template(
            "welcome",
            &json!({
                "name": user.name(),
                "url": format!("{}/verify/{}", ctx.config().server().url(), user.verification_token_hash().unwrap()),
                "subject": "Welcome"
            }),
        )?;

        let email = Email {
            to: user.email().to_string(),
            subject: "Welcome to Silk".to_string(),
            text: rendered,
            html: "welcome.hbs".to_string(),
            ..Default::default()
        };

        this.mail(&email, ctx).await?;

        Ok(())
    }

    /// Function to send a reset link to users
    /// who have forgotten their password.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The email was sent successfully.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The `MAILER_TEMPLATES` lazy lock fails to initialize.
    /// - The `HandlebarsTemplate` fails to clone.
    /// - The user's reset token hash is `None`.
    ///
    /// # Panics
    /// * This function will panic if the reset token is not set
    pub async fn forgot_password(ctx: &AppContext, user: &User) -> MailerResult<()> {
        let this = Self::init()?;

        if user.reset_token_hash().is_none() {
            return Err(MailerError::MissingVariable);
        }

        let rendered = this.renderer.render_template(
            "forgot",
            &json!({
                "name": user.name(),
                "url": format!("{}/reset-password/{}", ctx.config().server().url(), user.reset_token_hash().unwrap()),
                "subject": "Forgot Password?"
            }),
        )?;

        let email = Email {
            to: user.email().to_string(),
            subject: "Forgot Your Password?".to_string(),
            text: rendered,
            html: "forgot.hbs".to_string(),
            ..Default::default()
        };

        this.mail(&email, ctx).await?;

        Ok(())
    }
}
