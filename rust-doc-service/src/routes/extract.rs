use crate::error::AppError;
use crate::extract;
use crate::institutions::Registry;
use axum::{
    extract::{Multipart, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct ExtractParams {
    institution: Option<String>,
}

pub async fn handler(
    State(_registry): State<Registry>,
    Query(_params): Query<ExtractParams>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|_| AppError::Extraction("No file provided".into()))?
        .ok_or_else(|| AppError::Extraction("No file provided".into()))?;

    let mime_type = field
        .content_type()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let file_bytes = field
        .bytes()
        .await
        .map_err(|e| AppError::Extraction(e.to_string()))?;

    let doc = extract::extract(&file_bytes, &mime_type).await?;
    Ok(Json(serde_json::to_value(doc).unwrap()))
}
