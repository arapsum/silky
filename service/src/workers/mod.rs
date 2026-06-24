mod mail;

pub use self::mail::{MailJob, MailQueue, handle_forgot_password, handle_welcome};
