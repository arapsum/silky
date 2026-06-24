mod auth;

use std::sync::LazyLock;

use handlebars::{DirectorySourceOptions, Handlebars};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, Transport,
    message::{MessageBuilder, MultiPart},
    transport::{smtp::authentication::Credentials, stub::StubTransport},
};
use serde::{Deserialize, Serialize};

use crate::{
    AppContext,
    config::SmtpConfig,
    error::{MailerError, MailerResult},
};

pub const DEFAULT_SENDER: &str = "System <noreply@silk.io>";

pub static MAILER_TEMPLATES: LazyLock<MailerResult<HandlebarsTemplate>> =
    LazyLock::new(|| -> MailerResult<HandlebarsTemplate> {
        let mut handlebars = HandlebarsTemplate::init()?;

        handlebars.add_template("styles", Some("partials"))?;
        handlebars.add_template("base", Some("layouts"))?;
        handlebars.add_template("welcome", None)?;
        handlebars.add_template("forgot", None)?;

        Ok(handlebars)
    });

pub struct HandlebarsTemplate {
    pub registry: Handlebars<'static>,
}

impl HandlebarsTemplate {
    /// Initializes a new [`HandlebarsTemplate`] instance.
    ///
    /// Creates a Handlebars registry with strict mode enabled and loads all
    /// templates found in the project's `templates` directory.
    ///
    /// Strict mode causes rendering to fail whenever a referenced variable is
    /// missing from the provided template context.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - The `templates` directory cannot be accessed.
    /// - A template file cannot be parsed or registered.
    pub fn init() -> MailerResult<Self> {
        let mut registry = Handlebars::new();
        registry.set_strict_mode(true);

        registry.register_templates_directory("templates", DirectorySourceOptions::default())?;

        Ok(Self { registry })
    }

    /// Registers a template file in the Handlebars registry.
    ///
    /// The template name is used as the registration key and may be provided
    /// with or without the `.hbs` extension. If a path is supplied, it is
    /// treated as a subdirectory within the `templates` directory.
    ///
    /// # Examples
    ///
    /// ```text
    /// add_template("welcome", None)
    /// // -> ./templates/welcome.hbs
    ///
    /// add_template("styles", Some("partials"))
    /// // -> ./templates/partials/styles.hbs
    /// ```
    ///
    /// Leading and trailing slashes in `path` are automatically removed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - The resolved template file does not exist.
    /// - The template cannot be parsed.
    /// - The template cannot be registered with Handlebars.
    pub fn add_template(&mut self, template: &str, path: Option<&str>) -> MailerResult<()> {
        let template = if std::path::Path::new(template)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("hbs"))
        {
            template.strip_suffix(".hbs").unwrap_or(template)
        } else {
            template
        };

        let path = path.map_or_else(
            || Ok(format!("./templates/{template}.hbs")),
            |mut path| -> MailerResult<String> {
                if path.starts_with('/') {
                    path = path.strip_prefix('/').ok_or_else(|| MailerError::IO)?;
                }

                if path.ends_with('/') {
                    path = path.strip_suffix('/').ok_or_else(|| MailerError::IO)?;
                }

                Ok(format!("./templates/{path}/{template}.hbs"))
            },
        )?;

        self.registry.register_template_file(template, path)?;
        Ok(())
    }

    /// Renders a registered template using the provided context data.
    ///
    /// The template must already be registered in the registry. Template
    /// variables are resolved from `locals`, which is provided as a
    /// [`serde_json::Value`].
    ///
    /// Because strict mode is enabled, rendering will fail if the template
    /// references a variable that is not present in the provided context.
    ///
    /// # Arguments
    ///
    /// * `template` - The name of the registered template.
    /// * `locals` - Template context data used during rendering.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - No template exists with the specified name.
    /// - Template rendering fails.
    /// - Required template variables are missing from the context.
    pub fn render_template(
        &self,
        template: &str,
        locals: &serde_json::Value,
    ) -> MailerResult<String> {
        self.registry.render(template, locals).map_err(Into::into)
    }
}

#[allow(async_fn_in_trait)]
pub trait Mailer {
    #[must_use]
    fn opts() -> MailerOpts {
        MailerOpts {
            from: DEFAULT_SENDER.to_string(),
            ..Default::default()
        }
    }

    /// Creates an email transport using the application's mailer configuration.
    ///
    /// The transport is constructed from the SMTP settings defined in the
    /// application's configuration and is used to deliver outgoing email
    /// messages.
    ///
    /// Implementors may override this method to provide a custom transport,
    /// such as a mock transport for testing or an alternative delivery backend.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - The SMTP transport cannot be initialized.
    /// - The configured SMTP settings are invalid.
    fn transport(&self, context: &AppContext) -> MailerResult<EmailSender> {
        let mailer = context.config().mailer();

        EmailSender::sender(mailer.smtp())
    }

    /// Sends an email using the configured transport.
    ///
    /// The email is cloned before sending so that default mailer options can be
    /// applied without mutating the original value. If the email does not specify
    /// a sender address or reply-to address, the values returned by [`Self::opts`]
    /// are used.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - The mail transport cannot be created.
    /// - The email message cannot be constructed.
    /// - An email address is invalid.
    /// - The underlying transport fails to send the message.
    async fn mail(&self, email: &Email, context: &AppContext) -> MailerResult<()> {
        let opts = Self::opts();
        let mut email = email.clone();

        email.from = Some(email.from.unwrap_or_else(|| opts.from.clone()));
        email.reply_to = email.reply_to.or_else(|| opts.reply_to.clone());

        self.transport(context)?.send_email(&email).await
    }
}

/// Arguments used when rendering and constructing an email.
#[derive(Debug, Clone, Default)]
pub struct EmailArgs {
    pub from: Option<String>,
    pub to: String,
    pub reply_to: Option<String>,
    pub locals: serde_json::Value,
    pub bcc: Option<String>,
    pub cc: Option<String>,
}

/// Fully rendered email message ready to be sent.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Email {
    /// Mailbox to `From` header
    pub from: Option<String>,
    /// Mailbox to `To` header
    pub to: String,
    /// Mailbox to `ReplyTo` header
    pub reply_to: Option<String>,
    /// Subject header to message
    pub subject: String,
    /// Plain text message
    pub text: String,
    /// HTML template
    pub html: String,
    /// BCC header to message
    pub bcc: Option<String>,
    /// CC header to message
    pub cc: Option<String>,
}

/// Default configuration applied by a [`Mailer`] implementation.
#[derive(Debug, Default)]
pub struct MailerOpts {
    pub from: String,
    pub reply_to: Option<String>,
}

/// Transport implementation used to deliver emails.
#[derive(Debug, Clone)]
pub enum EmailTransport {
    /// SMTP transport backed by `lettre`.
    Smtp(AsyncSmtpTransport<Tokio1Executor>),

    /// In-memory transport used primarily for testing.
    Test(StubTransport),
}

/// Service responsible for constructing and dispatching email messages.
#[derive(Debug, Clone)]
pub struct EmailSender {
    pub transport: EmailTransport,
}

impl EmailSender {
    /// Creates an SMTP-backed email sender from the provided configuration.
    ///
    /// If SMTP authentication credentials are configured, they are attached to
    /// the transport builder. Depending on the configuration, either a STARTTLS
    /// relay transport or an insecure transport is created.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - The SMTP relay configuration is invalid.
    /// - The SMTP transport cannot be initialized.
    pub fn sender(cfg: &SmtpConfig) -> MailerResult<Self> {
        let mut builder = if cfg.secure() {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(cfg.host())?.port(cfg.port())
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(cfg.host()).port(cfg.port())
        };

        if let Some(auth) = cfg.auth() {
            builder = builder.credentials(Credentials::new(
                auth.username().to_string(),
                auth.password().to_string(),
            ));
        }

        Ok(Self {
            transport: EmailTransport::Smtp(builder.build()),
        })
    }

    /// Creates a stub email sender for testing purposes.
    ///
    /// Emails sent through the returned sender are accepted successfully but
    /// are not delivered to any external SMTP server.
    #[must_use]
    pub fn stub() -> Self {
        Self {
            transport: EmailTransport::Test(StubTransport::new_ok()),
        }
    }

    /// Sends an email using the configured transport.
    ///
    /// Both plain-text and HTML representations are included in the message as
    /// a multipart alternative body. Optional CC, BCC, and Reply-To headers are
    /// added when present.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Any email address cannot be parsed.
    /// - The message cannot be constructed.
    /// - The SMTP transport fails to send the message.
    /// - The test transport reports a delivery failure.
    pub async fn send_email(&self, email: &Email) -> MailerResult<()> {
        let contents: MultiPart =
            MultiPart::alternative_plain_html(email.html.clone(), email.text.clone());

        let mut msg_builder: MessageBuilder = Message::builder()
            .from(
                email
                    .from
                    .clone()
                    .unwrap_or_else(|| DEFAULT_SENDER.to_string())
                    .parse()?,
            )
            .to(email.to.parse()?);

        if let Some(bcc) = &email.bcc {
            msg_builder = msg_builder.bcc(bcc.parse()?);
        }
        if let Some(cc) = &email.cc {
            msg_builder = msg_builder.cc(cc.parse()?);
        }

        if let Some(reply_to) = &email.reply_to {
            msg_builder = msg_builder.reply_to(reply_to.parse()?);
        }

        let message: Message = msg_builder
            .subject(email.subject.clone())
            .multipart(contents)?;

        match &self.transport {
            EmailTransport::Smtp(xp) => {
                xp.send(message).await?;
            }
            EmailTransport::Test(xp) => {
                xp.send(&message)?;
            }
        }

        Ok(())
    }
}
