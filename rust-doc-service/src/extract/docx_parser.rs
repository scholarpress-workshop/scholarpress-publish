use crate::error::AppError;
use crate::extract::document::{ParsedDocument, ParsedMetadata, ParsedPage, ParsedParagraph};
use std::collections::HashMap;
use std::io::Read;

pub fn parse_docx(bytes: &[u8]) -> Result<ParsedDocument, AppError> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| AppError::Extraction(format!("Failed to open DOCX: {}", e)))?;

    let doc_xml = read_zip_entry(&mut archive, "word/document.xml")?;
    let styles_xml = read_zip_entry(&mut archive, "word/styles.xml").ok();
    let styles = parse_styles(&styles_xml);
    let paragraphs = parse_paragraphs(&doc_xml, &styles)?;

    let raw_text: String = paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    let word_count: usize = paragraphs
        .iter()
        .map(|p| p.text.split_whitespace().count())
        .sum();
    let page_count = if word_count > 0 {
        (word_count / 250).max(1) as u32
    } else {
        1
    };

    let mut detected_fonts: Vec<String> = paragraphs
        .iter()
        .filter_map(|p| p.font_name.clone())
        .collect();
    detected_fonts.sort();
    detected_fonts.dedup();

    let all_page_text: String = paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(ParsedDocument {
        raw_text,
        pages: vec![ParsedPage {
            number: 1,
            text: all_page_text,
            width: None,
            height: None,
        }],
        paragraphs,
        headings: vec![],
        metadata: ParsedMetadata {
            title: None,
            author: None,
            page_count,
            page_count_estimated: true,
            detected_fonts,
        },
    })
}

fn read_zip_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<String, AppError> {
    let mut file = archive
        .by_name(name)
        .map_err(|e| AppError::Extraction(format!("Missing {} in DOCX: {}", name, e)))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| AppError::Extraction(format!("Failed to read {}: {}", name, e)))?;
    Ok(contents)
}

#[derive(Debug, Clone, Default)]
struct StyleInfo {
    font_name: Option<String>,
    font_size: Option<f32>,
    is_bold: bool,
    is_italic: bool,
    is_underline: bool,
}

fn parse_styles(styles_xml: &Option<String>) -> HashMap<String, StyleInfo> {
    let mut map = HashMap::new();
    let xml = match styles_xml {
        Some(s) => s,
        None => return map,
    };
    let doc = match roxmltree::Document::parse(xml) {
        Ok(d) => d,
        Err(_) => return map,
    };
    let ns = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

    for style_node in doc.descendants().filter(|n| n.has_tag_name((ns, "style"))) {
        let style_id = style_node
            .attribute((ns, "styleId"))
            .unwrap_or("")
            .to_string();
        let mut info = StyleInfo::default();

        if let Some(rpr) = style_node
            .descendants()
            .find(|n| n.has_tag_name((ns, "rPr")))
        {
            for child in rpr.children() {
                if child.has_tag_name((ns, "rFonts")) {
                    info.font_name = child.attribute((ns, "ascii")).map(|s| s.to_string());
                }
                if child.has_tag_name((ns, "sz")) || child.has_tag_name((ns, "szCs")) {
                    if let Some(val) = child.attribute((ns, "val")) {
                        if let Ok(half_pts) = val.parse::<f32>() {
                            info.font_size = Some(half_pts / 2.0);
                        }
                    }
                }
                if child.has_tag_name((ns, "b")) {
                    info.is_bold = true;
                }
                if child.has_tag_name((ns, "i")) {
                    info.is_italic = true;
                }
                if child.has_tag_name((ns, "u")) {
                    info.is_underline = true;
                }
            }
        }

        if !style_id.is_empty() {
            map.insert(style_id, info);
        }
    }

    map
}

fn parse_paragraphs(
    doc_xml: &str,
    styles: &HashMap<String, StyleInfo>,
) -> Result<Vec<ParsedParagraph>, AppError> {
    let doc = roxmltree::Document::parse(doc_xml)
        .map_err(|e| AppError::Extraction(format!("Failed to parse document.xml: {}", e)))?;

    let ns = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let mut paragraphs: Vec<ParsedParagraph> = Vec::new();

    for p_node in doc.descendants().filter(|n| n.has_tag_name((ns, "p"))) {
        let mut para_text = String::new();
        let mut is_bold = false;
        let mut is_italic = false;
        let mut is_underline = false;
        let mut font_size: Option<f32> = None;
        let mut font_name: Option<String> = None;

        let p_pr = p_node.children().find(|n| n.has_tag_name((ns, "pPr")));
        let p_style = p_pr
            .and_then(|pr| pr.children().find(|n| n.has_tag_name((ns, "pStyle"))))
            .and_then(|ps| ps.attribute((ns, "val")))
            .unwrap_or("Normal");
        let resolved = styles.get(p_style);

        for r_node in p_node.children().filter(|n| n.has_tag_name((ns, "r"))) {
            let r_pr = r_node.children().find(|n| n.has_tag_name((ns, "rPr")));

            let run_font_name = r_pr
                .and_then(|rp| rp.children().find(|n| n.has_tag_name((ns, "rFonts"))))
                .and_then(|rf| rf.attribute((ns, "ascii")))
                .map(|s| s.to_string());

            let run_size = r_pr
                .and_then(|rp| {
                    rp.children()
                        .find(|n| n.has_tag_name((ns, "sz")) || n.has_tag_name((ns, "szCs")))
                })
                .and_then(|sz| sz.attribute((ns, "val")))
                .and_then(|v| v.parse::<f32>().ok())
                .map(|v| v / 2.0);

            let run_bold = r_pr.is_some_and(|rp| rp.children().any(|n| n.has_tag_name((ns, "b"))));
            let run_italic =
                r_pr.is_some_and(|rp| rp.children().any(|n| n.has_tag_name((ns, "i"))));
            let run_underline =
                r_pr.is_some_and(|rp| rp.children().any(|n| n.has_tag_name((ns, "u"))));

            is_bold = is_bold || run_bold;
            is_italic = is_italic || run_italic;
            is_underline = is_underline || run_underline;

            if font_name.is_none() {
                font_name = run_font_name.or(resolved.and_then(|s| s.font_name.clone()));
            }
            if font_size.is_none() {
                font_size = run_size.or(resolved.and_then(|s| s.font_size));
            }

            for t_node in r_node.children().filter(|n| n.has_tag_name((ns, "t"))) {
                if let Some(text) = t_node.text() {
                    para_text.push_str(text);
                }
            }
            for _br in r_node.children().filter(|n| n.has_tag_name((ns, "br"))) {
                para_text.push('\n');
            }
        }

        let trimmed = para_text.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }

        let is_all_caps = trimmed
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase())
            && trimmed.chars().filter(|c| c.is_alphabetic()).count() >= 4;

        paragraphs.push(ParsedParagraph {
            text: trimmed,
            page_number: None,
            is_bold,
            is_italic,
            is_underline,
            is_all_caps,
            is_heading: false,
            heading_level: None,
            font_size: font_size.or(Some(12.0)),
            font_name,
        });
    }

    Ok(paragraphs)
}
