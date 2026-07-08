pub mod auth;
pub mod categories;
pub mod products;
pub mod request;
pub mod roles;
pub mod validator;

pub use self::{
    auth::{ChangePassword, ForgotPassword, LoginUser, RegisterUser, ResetPassword, UpdateProfile},
    categories::{NewCategory, UpdateCategory},
    products::{CreateProduct, CreateProductPicture, CreateProductVariant, CreateVariantOption},
    request::{
        CategoryListQuery, PaginationQuery, PermissionListQuery, PermissionRoleQuery, UserListQuery,
    },
    roles::{AssignPermission, AssignRole, NewRole, UpdateRole},
    validator::Validator,
};
