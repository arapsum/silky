use std::path::Path;

use serde::de::DeserializeOwned;
use sqlx::PgPool;

use super::{ModelError, ModelResult};

#[allow(async_fn_in_trait)]
pub trait Seedable: Sized + Send + Sync + DeserializeOwned + Clone {
    async fn seed(db: &PgPool, data: &[Self]) -> ModelResult<()>;

    #[must_use]
    async fn load(file: &str) -> ModelResult<Vec<Self>> {
        let path = Path::new(&format!("src/data/{file}")).to_path_buf();

        if !path.exists() || !path.is_file() {
            return Err(ModelError::FileNotFound);
        }

        let contents = tokio::fs::read_to_string(&path).await?;
        let ext = path.extension().ok_or(ModelError::UnsupportedFileType)?;

        if ext.eq_ignore_ascii_case("json") {
            serde_json::from_str::<Vec<Self>>(&contents).map_err(Into::into)
        } else if ext.eq_ignore_ascii_case("yaml") {
            serde_saphyr::from_str::<Vec<Self>>(&contents).map_err(Into::into)
        } else {
            Err(ModelError::UnsupportedFileType)
        }
    }
}
