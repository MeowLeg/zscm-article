use super::*;
pub mod get_articles;

use axum::{
    Json,
    extract::{Extension, Query, rejection::JsonRejection},
};
use serde_json::{Value, json};

#[allow(dead_code)]
pub trait ExecSql<T> {
    async fn handle_post(
        _cfg: Extension<Arc<Config>>,
        _prms: Result<Json<T>, JsonRejection>,
    ) -> Result<Json<Value>> {
        Ok(Json(json!({})))
    }

    async fn handle_get(
        _cfg: Extension<Arc<Config>>,
        _prms: Option<Query<T>>,
    ) -> Result<Json<Value>> {
        Ok(Json(json!({})))
    }
}
