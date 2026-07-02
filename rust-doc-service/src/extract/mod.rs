pub mod document;

use crate::error::AppError;
use document::{Document, DocumentContent, DocumentMetadata, DocumentStructure, Page};

pub async fn extract(file_bytes: &[u8], mime_type: &str) -> Result<Document, AppError> {
    let config = kreuzberg::ExtractionConfig::default();
    let result = kreuzberg::extract_bytes(file_bytes, mime_type, &config)
        .await
        .map_err(|e| AppError::Extraction(e.to_string()))?;

    let text = result.content;
    let page_count = result.pages.as_ref().map_or(0, |p| p.len()) as u32;

    let pages = result
        .pages
        .unwrap_or_default()
        .into_iter()
        .map(|p| Page {
            number: p.page_number,
            text: p.content,
        })
        .collect();

    let author = result.metadata.authors.and_then(|a| a.into_iter().next());

    Ok(Document {
        content: DocumentContent {
            pages,
            raw_text: text,
        },
        structure: DocumentStructure {
            front_matter: vec![],
            body: vec![],
            end_matter: vec![],
        },
        metadata: DocumentMetadata {
            title: result.metadata.title,
            author,
            page_count,
            detected_fonts: vec![],
        },
    })
}
