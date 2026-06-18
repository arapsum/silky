pub mod error;
pub mod seed;
pub mod user;

pub use self::{
    error::{ModelError, ModelResult},
    seed::Seedable,
    user::User,
};
