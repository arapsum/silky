pub mod auth;
pub mod categories;
pub mod products;
pub mod request;
pub mod roles;
pub mod validator;

pub use self::{
    auth::{ChangePassword, ForgotPassword, LoginUser, RegisterUser, ResetPassword, UpdateProfile},
    categories::{CategoryAttributesInput, NewCategory, UpdateCategory},
    products::{
        CreateProduct, CreateProductPicture, CreateProductTag, CreateProductVariant,
        CreateVariantOption, ProductListQuery, StockStatus, UpdateProduct, UpdateProductPicture,
        UpdateProductVariant,
    },
    request::{
        CategoryListQuery, PaginationQuery, PermissionListQuery, PermissionRoleQuery, UserListQuery,
    },
    roles::{AssignPermission, AssignRole, NewRole, UpdateRole},
    validator::Validator,
};
