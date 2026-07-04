use crate::error::AppError;
use crate::extract::document::{ParsedDocument, ParsedMetadata, ParsedPage, ParsedParagraph};

pub fn parse_pdf(bytes: &[u8]) -> Result<ParsedDocument, AppError> {
    let doc = pdf_oxide::PdfDocument::from_bytes(bytes.to_vec())
        .map_err(|e| AppError::Extraction(format!("Failed to open PDF: {}", e)))?;

    let page_count = doc
        .page_count()
        .map_err(|e| AppError::Extraction(format!("Failed to get page count: {}", e)))?;

    let mut pages: Vec<ParsedPage> = Vec::with_capacity(page_count);
    let mut paragraphs: Vec<ParsedParagraph> = Vec::new();
    let mut all_text = String::new();
    let mut detected_fonts: Vec<String> = Vec::new();

    for page_index in 0..page_count {
        let (llx, _lly, urx, ury) = doc
            .get_page_media_box(page_index)
            .map_err(|e| AppError::Extraction(format!("Failed to get page box: {}", e)))?;
        let width = urx - llx;
        let height = ury - _lly;

        let chars = doc
            .extract_chars(page_index)
            .map_err(|e| AppError::Extraction(format!("Failed to extract chars: {}", e)))?;

        let mut word_chars: Vec<&pdf_oxide::layout::TextChar> = Vec::new();
        let mut words: Vec<TextSpan> = Vec::new();

        for ch in &chars {
            if ch.char.is_whitespace() {
                if !word_chars.is_empty() {
                    words.push(build_word(&word_chars, height));
                    word_chars.clear();
                }
            } else {
                if !word_chars.is_empty() {
                    let last = word_chars.last().unwrap();
                    let same_line = (last.origin_y - ch.origin_y).abs() < 3.0;
                    let gap = ch.bbox.x - (last.bbox.x + last.bbox.width);
                    let same_font = last.font_name == ch.font_name
                        && (last.font_size - ch.font_size).abs() < 1.0;

                    if !same_line || !same_font || gap > 20.0 {
                        words.push(build_word(&word_chars, height));
                        word_chars.clear();
                    }
                }
                word_chars.push(ch);
            }
        }
        if !word_chars.is_empty() {
            words.push(build_word(&word_chars, height));
        }

        let mut para_spans: Vec<&TextSpan> = Vec::new();
        let mut para_text = String::new();
        let mut line_heights: Vec<f32> = Vec::new();

        let mut prev_origin_y: Option<f32> = None;

        for w in &words {
            let (_top, bottom, _, _) = w.bbox;
            let line_height = bottom - _top;
            line_heights.push(line_height);

            let line_changed = prev_origin_y
                .map(|prev| (w.origin_y - prev).abs() > 3.0)
                .unwrap_or(false);
            prev_origin_y = Some(w.origin_y);

            if line_changed {
                line_heights.clear();
                line_heights.push(line_height);
            }

            let gap = if let Some(last_bottom) = para_spans.last().map(|s: &&TextSpan| s.bbox.1) {
                _top - last_bottom
            } else {
                0.0
            };

            let median_lh = median(&line_heights);
            let is_para_break = line_changed && gap > median_lh.max(1.0);

            if is_para_break {
                let pp = assemble_paragraph(&para_spans, page_index, &mut detected_fonts);
                let pt = pp.text.clone();
                paragraphs.push(pp);
                para_text.push_str(&pt);
                para_text.push_str("\n\n");
                para_spans.clear();
                line_heights.clear();
                line_heights.push(line_height);
            }
            para_spans.push(w);
        }
        if !para_spans.is_empty() {
            let pp = assemble_paragraph(&para_spans, page_index, &mut detected_fonts);
            let pt = pp.text.clone();
            paragraphs.push(pp);
            para_text.push_str(&pt);
        }

        let mut page_text = para_text;
        while page_text.ends_with('\n') {
            page_text.pop();
        }

        all_text.push_str(&page_text);
        pages.push(ParsedPage {
            number: (page_index + 1) as u32,
            text: page_text.clone(),
            width: Some(width),
            height: Some(height),
        });
        all_text.push('\n');
    }

    all_text = all_text.trim().to_string();
    detected_fonts.sort();
    detected_fonts.dedup();

    Ok(ParsedDocument {
        raw_text: all_text,
        pages,
        paragraphs,
        headings: vec![],
        metadata: ParsedMetadata {
            title: None,
            author: None,
            page_count: page_count as u32,
            page_count_estimated: false,
            detected_fonts,
        },
    })
}

struct TextSpan {
    text: String,
    font_name: String,
    font_size: f32,
    bbox: (f32, f32, f32, f32),
    is_bold: bool,
    is_italic: bool,
    origin_y: f32,
}

fn build_word(chars: &[&pdf_oxide::layout::TextChar], page_height: f32) -> TextSpan {
    let first = chars[0];
    let text: String = chars.iter().map(|c| c.char).collect();

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for ch in chars {
        let x = ch.bbox.x;
        let y = ch.bbox.y;
        let w = ch.bbox.width;
        let h = ch.bbox.height;
        min_x = min_x.min(x);
        max_x = max_x.max(x + w);
        min_y = min_y.min(y);
        max_y = max_y.max(y + h);
    }

    let top = page_height - max_y;
    let bottom = page_height - min_y;

    TextSpan {
        text,
        font_name: first.font_name.clone(),
        font_size: first.font_size,
        bbox: (top.max(0.0), bottom, min_x, max_x),
        is_bold: matches!(first.font_weight, pdf_oxide::layout::FontWeight::Bold),
        is_italic: first.is_italic,
        origin_y: first.bbox.y,
    }
}

fn assemble_paragraph(
    spans: &[&TextSpan],
    page_index: usize,
    detected_fonts: &mut Vec<String>,
) -> ParsedParagraph {
    let text: String = spans
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let is_all_caps = text
        .chars()
        .filter(|c| c.is_alphabetic())
        .all(|c| c.is_uppercase())
        && text.chars().filter(|c| c.is_alphabetic()).count() >= 4;
    let is_bold = spans.iter().any(|s| s.is_bold);
    let is_italic = spans.iter().any(|s| s.is_italic);

    let mut sizes: Vec<f32> = spans.iter().map(|s| s.font_size).collect();
    sizes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let font_size = if !sizes.is_empty() {
        Some(sizes[sizes.len() / 2])
    } else {
        None
    };

    let font_name = spans.first().map(|s| {
        let name = s.font_name.clone();
        if !detected_fonts.contains(&name) {
            detected_fonts.push(name.clone());
        }
        name
    });

    ParsedParagraph {
        text,
        page_number: Some((page_index + 1) as u32),
        is_bold,
        is_italic,
        is_underline: false,
        is_all_caps,
        is_heading: false,
        heading_level: None,
        font_size,
        font_name,
    }
}

fn median(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 1.0;
    }
    let mut sorted: Vec<f32> = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    sorted[sorted.len() / 2]
}
