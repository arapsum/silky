pub mod auth;
pub mod categories;
pub mod orders;
pub mod products;
pub mod request;
pub mod roles;
pub mod validator;

pub use self::{
    auth::{ChangePassword, ForgotPassword, LoginUser, RegisterUser, ResetPassword, UpdateProfile},
    categories::{CategoryAttributesInput, NewCategory, UpdateCategory},
    orders::{NewAddress, NewOrder, NewOrderItem, UpdateOrder},
    products::{
        CreateProduct, CreateProductPicture, CreateProductTag, CreateProductVariant,
        CreateVariantOption, ProductListQuery, StockStatus, UpdateProduct, UpdateProductPicture,
        UpdateProductVariant,
    },
    request::{
        CategoryListQuery, OrderListQuery, PaginationQuery, PermissionListQuery,
        PermissionRoleQuery, UserListQuery,
    },
    roles::{AssignPermission, AssignRole, NewRole, UpdateRole},
    validator::Validator,
};
