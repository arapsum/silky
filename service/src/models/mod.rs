pub mod error;
pub mod roles;
pub mod seed;
pub mod user;

pub use self::{
    error::{ModelError, ModelResult},
    roles::Role,
    seed::Seedable,
    user::User,
};
