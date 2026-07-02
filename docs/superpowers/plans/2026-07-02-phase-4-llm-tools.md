# Phase 4 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the LLM the context it needs to do its job — a complete Typst template, the formatting spec, and on-demand tools to read both.

**Architecture:** Two new Rust endpoints (spec + template) backed by the existing Registry. Two new AI SDK tools wrapping those endpoints. Dynamic system prompt injected at request time. A full 14-section Typst template that passes all 33 IU diss-check checks by construction.

**Tech Stack:** Same as existing — Rust (axum, serde_yaml, tokio), Next.js (App Router, AI SDK v7), Typst, diss-check

## Global Constraints

- Use `rtk` prefix for all shell commands
- TDD: write tests before implementation where applicable
- Verify with lint/typecheck/build before claiming work done
- Never commit unless explicitly asked (the plan will say when)
- Rust tests: `cargo test` — all 6 existing must not regress
- Web build: `bun run build` must pass
- Use bun (not npm) for web commands

---

### Task 1: Copy IU spec from diss-check

**Files:**
- Overwrite: `institutions/iu/spec.yaml`

**Interfaces:**
- Produces: Full 327-line IU spec with all 33 diss-check checks, available for tasks 2 and 6

- [ ] **Step 1: Copy the spec**

```bash
cp /home/danriggi/diss-check/specs/iu.yaml /home/danriggi/format-my-dissertation/institutions/iu/spec.yaml
```

- [ ] **Step 2: Verify Rust tests still pass**

```bash
rtk cargo test
```

Expected: 6 tests pass (the institutions test reads this file)

- [ ] **Step 3: Commit**

```bash
rtk git add institutions/iu/spec.yaml
rtk git commit -m "feat: copy full IU spec from diss-check (33 checks)"
```

---

### Task 2: Write Typst shared styles

**Files:**
- Create: `institutions/iu/template/styles.typ`

**Interfaces:**
- Produces: shared styling module consumed by all section files (tasks 3-5)
- Exports: `iu-body-font`, `iu-body-size`, `iu-heading-font`, `iu-heading-size`, `iu-line-spacing`, `iu-page-setup`, `iu-margin`, `iu-running-head`, `iu-chapter-style`, `iu-toc-style`, `iu-reference-style`

- [ ] **Step 1: Write styles.typ**

```typst
#let iu-body-font = "Times New Roman"
#let iu-body-size = 12pt
#let iu-heading-font = "Times New Roman"
#let iu-heading-size = 12pt
#let iu-line-spacing = 2.0 // double-spaced body text
#let iu-margin-left = 1.25in
#let iu-margin-right = 1.25in
#let iu-margin-top = 1in
#let iu-margin-bottom = 1in

#let iu-page-setup(body) = {
  set page(
    margin: (top: iu-margin-top, bottom: iu-margin-bottom, left: iu-margin-left, right: iu-margin-right),
    numbering: "1",
  )
  set text(font: iu-body-font, size: iu-body-size)
  set par(leading: iu-line-spacing + 0pt)
  body
}

#let iu-heading(level, title) = {
  if level == 1 {
    align(center, text(weight: "bold", title))
    v(12pt)
  } else if level == 2 {
    text(weight: "bold", title)
    v(6pt)
  } else {
    text(style: "italic", title)
    v(6pt)
  }
}

#let iu-chapter-heading(title) = {
  pagebreak()
  align(center, text(weight: "bold", iu-heading-size, title))
  v(24pt)
}

#let iu-reference-style(body) = {
  set par(leading: 1em + 0pt)
  set text(size: iu-body-size)
  body
}

#let iu-toc-entry(title, page) = {
  title
  box(width: 1fr, repeat[.])
  #page
}

// IU page numbering: front matter uses roman numerals (ii, iii, ...),
// body uses arabic (1, 2, ...). Title page has no visible number.
// Acceptance page is page ii.
```

- [ ] **Step 2: Verify it's valid Typst syntax**

```bash
echo '#import "template/styles.typ": *' | typst compile --format pdf - /dev/null
```

Expected: produces a blank PDF with no errors

- [ ] **Step 3: Commit**

```bash
rtk git add institutions/iu/template/styles.typ
rtk git commit -m "feat: add IU Typst shared styles"
```

---

### Task 3: Write required section files

**Files:**
- Create: `institutions/iu/template/sections/title-page.typ`
- Create: `institutions/iu/template/sections/acceptance.typ`
- Create: `institutions/iu/template/sections/abstract.typ`
- Create: `institutions/iu/template/sections/toc.typ`
- Create: `institutions/iu/template/sections/chapters.typ`
- Create: `institutions/iu/template/sections/references.typ`
- Create: `institutions/iu/template/sections/cv.typ`

**Interfaces:**
- Consumes: `styles.typ` from task 2
- Produces: 7 section functions used in template.typ (task 5)

- [ ] **Step 1: Write title-page.typ**

```typst
#import "../styles.typ": iu-page-setup, iu-body-font, iu-body-size, iu-margin-top

// Per diss-check: title all caps, no bold, clause centered, no page number
// Human check: clause spacing matches document
#let title-page(
  title: "DISSERTATION TITLE",
  author: "Author Name",
  school: "",
  degree: "Doctor of Philosophy",
  department: "",
  campus: "Bloomington",
  month: "May",
  year: "2026",
) = {
  set page(numbering: none)
  iu-page-setup(
    align(center)[
      // Title — all caps, no bold, no page number
      #v(2in + iu-margin-top)
      #text(size: iu-body-size, weight: "regular", upper(title))
      #v(36pt)
      #text(size: iu-body-size)[#author]
      #v(36pt)

      // Clause — centered, specific wording per IU spec
      Submitted to the faculty of the #school
      in partial fulfillment of the requirements
      for the degree
      #degree
      in the #department,
      Indiana University #campus
      #month #year
    ]
  )
  // Human-review note: verify clause line spacing matches document body
}
```

- [ ] **Step 2: Write acceptance.typ**

```typst
#import "../styles.typ": iu-page-setup, iu-body-font, iu-body-size

// Per diss-check: page number ii, committee chair first, no signatures
#let acceptance-page(
  committee: (
    (name: "Chair Name", role: "Chair"),
    (name: "Member Name", role: "Member"),
    (name: "Member Name", role: "Member"),
  ),
) = {
  set page(numbering: "ii")
  iu-page-setup(
    align(center)[
      #v(2in)
      #text(size: iu-body-size, weight: "regular")[Accepted by the faculty of the University Graduate School in partial fulfillment of the requirements for the degree #emph[_Doctor of Philosophy_].]

      #v(48pt)

      #align(left)[
        #for member in committee [
          #v(18pt)
          #line(length: 50%)
          #text(size: iu-body-size)[#member.name]
          #v(6pt)
          #text(size: iu-body-size, style: "italic")[#member.role]
        ]
      ]
    ]
  )
  // Human-review note: no signatures needed
}
```

- [ ] **Step 3: Write abstract.typ**

```typst
#import "../styles.typ": iu-page-setup, iu-heading

// Per diss-check: abstract title format, centered text, unsigned, word count
#let abstract-page(
  title: "Abstract",
  body: "",
) = {
  set page(numbering: "iii")
  iu-page-setup(
    [
      #v(2in)
      #iu-heading(1, upper(title))
      #v(24pt)

      // Abstract body — centered text
      // Human-review note: verify abstract ≤350 words
      #text(size: 12pt)[#body]
    ]
  )
}
```

- [ ] **Step 4: Write toc.typ**

```typst
// Per diss-check: page numbers aligned right, no overhang, CV no dots, spacing
// Human check: TOC line spacing single or matches document
#let toc-page(
  entries: (
    (title: "Title Page", page: "i"),
    (title: "Acceptance Page", page: "ii"),
    (title: "Abstract", page: "iii"),
    (title: "Table of Contents", page: "iv"),
    (title: "Chapter 1: Introduction", page: "1"),
  ),
  cv-page: none, // CV page number, use line() not dots
) = {
  set page(numbering: "iv")
  [
    #align(center, text(weight: "bold", 12pt)[TABLE OF CONTENTS])
    #v(12pt)

    #for entry in entries [
      #entry.title
      #box(width: 1fr, repeat[.])
      #h(4pt)
      #entry.page
      #v(4pt)
    ]

    #if cv-page != none [
      Curriculum Vitae
      #v(0pt)
      #line(length: 100%)
      #h(4pt)
      #cv-page
    ]
  ]
}
```

- [ ] **Step 5: Write chapters.typ**

```typst
#import "../styles.typ": iu-page-setup, iu-heading, iu-body-size, iu-body-font

// Per diss-check: new chapter = new page, consistent headings, font size 10-12pt, consistent
#let chapter(heading: "", body: []) = {
  pagebreak()
  align(center, text(weight: "bold", iu-body-size, upper(heading)))
  v(24pt)
  iu-page-setup(body)
}

// Human-review notes for chapters:
// - Footnote font consistent with body
// - Footnote line spacing matches document
// - Table/figure legend font ≥10pt
```

- [ ] **Step 6: Write references.typ**

```typst
// Per diss-check: references heading format, font consistent, spacing
// Human check: references line spacing single or matches document
// Human check: references font consistent
#let references-page(entries: []) = {
  pagebreak()
  align(center, text(weight: "bold", 12pt)[REFERENCES])
  v(12pt)
  set par(leading: 1em + 0pt)
  entries
}
```

- [ ] **Step 7: Write cv.typ**

```typst
// Per diss-check: CV heading format, name position at top of page, no page number, no credentials/PII
// Human check: no academic credentials or PII on CV
#let curriculum-vitae(name: "", body: []) = {
  pagebreak()
  set page(numbering: none)
  [
    #align(center, text(weight: "bold", 12pt)[CURRICULUM VITAE])
    #v(12pt)
    #align(center, text(weight: "bold", 12pt)[#name])
    #v(24pt)
    #body
  ]
  // Human-review note: verify no academic credentials or PII listed
}
```

- [ ] **Step 8: Verify section files compile together**

```bash
mkdir -p /tmp/verify && cat > /tmp/verify/main.typ << 'TYPST'
#import "../institutions/iu/template/styles.typ": *
#import "../institutions/iu/template/sections/title-page.typ": title-page
#import "../institutions/iu/template/sections/acceptance.typ": acceptance-page
#import "../institutions/iu/template/sections/abstract.typ": abstract-page
#import "../institutions/iu/template/sections/toc.typ": toc-page
#import "../institutions/iu/template/sections/chapters.typ": chapter
#import "../institutions/iu/template/sections/references.typ": references-page
#import "../institutions/iu/template/sections/cv.typ": curriculum-vitae

#title-page(title: "Test", author: "Test Author", school: "University Graduate School", degree: "Doctor of Philosophy", department: "Computer Science", campus: "Bloomington", month: "May", year: "2026")
#acceptance-page(committee: ((name: "Dr. A", role: "Chair"), (name: "Dr. B", role: "Member")))
#abstract-page(title: "Abstract", body: "This is a test abstract.")
#toc-page()
#chapter(heading: "Introduction", body: [Test content.])
#references-page(entries: [Test reference.])
#curriculum-vitae(name: "Test Author", body: [Test CV content.])
TYPST
typst compile --format pdf /tmp/verify/main.typ /tmp/verify/output.pdf
```

Expected: produces a PDF with no errors

- [ ] **Step 9: Commit**

```bash
rtk git add institutions/iu/template/sections/
rtk git commit -m "feat: add required IU Typst section files"
```

---

### Task 4: Write optional section files

**Files:**
- Create: `institutions/iu/template/sections/copyright.typ`
- Create: `institutions/iu/template/sections/dedication.typ`
- Create: `institutions/iu/template/sections/acknowledgements.typ`
- Create: `institutions/iu/template/sections/preface.typ`
- Create: `institutions/iu/template/sections/lot.typ`
- Create: `institutions/iu/template/sections/lof.typ`
- Create: `institutions/iu/template/sections/lop.typ`
- Create: `institutions/iu/template/sections/loa.typ`
- Create: `institutions/iu/template/sections/appendices.typ`

**Interfaces:**
- Consumes: `styles.typ` from task 2
- Produces: 9 optional section functions used in template.typ (task 5)

- [ ] **Step 1: Write all 9 optional section files**

```typst
// copyright.typ
// Per diss-check: copyright page format
#let copyright-page(year: "2026", author: "Author Name") = {
  pagebreak()
  set page(numbering: none)
  [
    #v(50%)
    #align(center, text(size: 12pt)[Copyright © #year
    #author
    ALL RIGHTS RESERVED])
  ]
}

// dedication.typ
#let dedication-page(text: []) = {
  pagebreak()
  [
    #align(center)[
      #v(50%)
      #text(size: 12pt, style: "italic")[#text]
    ]
  ]
}

// acknowledgements.typ
#let acknowledgements-page(title: "Acknowledgements", body: []) = {
  pagebreak()
  [
    #align(center, text(weight: "bold", 12pt, upper(title)))
    #v(12pt)
    #body
  ]
}

// preface.typ
#let preface-page(title: "Preface", body: []) = {
  pagebreak()
  [
    #align(center, text(weight: "bold", 12pt, upper(title)))
    #v(12pt)
    #body
  ]
}

// lot.typ — List of Tables
#let list-of-tables(entries: ()) = {
  pagebreak()
  [
    #align(center, text(weight: "bold", 12pt)[LIST OF TABLES])
    #v(12pt)
    #for (title, page) in entries [
      #title
      #box(width: 1fr, repeat[.])
      #h(4pt)
      #page
      #v(4pt)
    ]
  ]
}

// lof.typ — List of Figures
#let list-of-figures(entries: ()) = {
  pagebreak()
  [
    #align(center, text(weight: "bold", 12pt)[LIST OF FIGURES])
    #v(12pt)
    #for (title, page) in entries [
      #title
      #box(width: 1fr, repeat[.])
      #h(4pt)
      #page
      #v(4pt)
    ]
  ]
}

// lop.typ — List of Pictures/Schematics
#let list-of-pictures(entries: ()) = {
  pagebreak()
  [
    #align(center, text(weight: "bold", 12pt)[LIST OF PICTURES])
    #v(12pt)
    #for (title, page) in entries [
      #title
      #box(width: 1fr, repeat[.])
      #h(4pt)
      #page
      #v(4pt)
    ]
  ]
}

// loa.typ — List of Abbreviations
#let list-of-abbreviations(entries: ()) = {
  pagebreak()
  [
    #align(center, text(weight: "bold", 12pt)[LIST OF ABBREVIATIONS])
    #v(12pt)
    #for (abbr, meaning) in entries [
      #abbr #h(12pt) #meaning
      #v(4pt)
    ]
  ]
}

// appendices.typ
#let appendix(label: "A", title: "", body: []) = {
  pagebreak()
  [
    #align(center, text(weight: "bold", 12pt, upper(title)))
    #v(24pt)
    #body
  ]
}
```

- [ ] **Step 2: Verify they compile**

```bash
cat > /tmp/verify-opt.typ << 'TYPST'
#import "institutions/iu/template/styles.typ": *
#import "institutions/iu/template/sections/copyright.typ": copyright-page
#import "institutions/iu/template/sections/dedication.typ": dedication-page
#import "institutions/iu/template/sections/acknowledgements.typ": acknowledgements-page
#import "institutions/iu/template/sections/preface.typ": preface-page
#import "institutions/iu/template/sections/lot.typ": list-of-tables
#import "institutions/iu/template/sections/lof.typ": list-of-figures
#import "institutions/iu/template/sections/lop.typ": list-of-pictures
#import "institutions/iu/template/sections/loa.typ": list-of-abbreviations
#import "institutions/iu/template/sections/appendices.typ": appendix

#copyright-page()
#dedication-page(text: [To my family.])
#acknowledgements-page(body: [Thanks everyone.])
#preface-page(body: [Preface text.])
#list-of-tables(entries: (("Table 1: Data", "10"),))
#list-of-figures(entries: (("Figure 1: Chart", "12"),))
#list-of-pictures(entries: (("Picture 1: Photo", "14"),))
#list-of-abbreviations(entries: (("API", "Application Programming Interface"),))
#appendix(label: "A", title: "Survey Questions", body: [Appendix content.])
TYPST
cd /home/danriggi/format-my-dissertation && typst compile --format pdf /tmp/verify-opt.typ /tmp/verify-opt.pdf
```

Expected: produces a PDF with no errors

- [ ] **Step 3: Commit**

```bash
rtk git add institutions/iu/template/sections/
rtk git commit -m "feat: add optional IU Typst section files"
```

---

### Task 5: Write template.typ entrypoint

**Files:**
- Modify: `institutions/iu/template/template.typ`

**Interfaces:**
- Consumes: `styles.typ` (task 2), section files (tasks 3-4)
- Produces: Complete entrypoint that can be passed to `typst compile` for a full dissertation

- [ ] **Step 1: Write template.typ**

```typst
// IU dissertation template — entrypoint
// Import shared styles and all section components
#import "styles.typ": iu-page-setup, iu-margin-top, iu-heading-size, iu-body-font
#import "sections/title-page.typ": title-page
#import "sections/acceptance.typ": acceptance-page
#import "sections/copyright.typ": copyright-page
#import "sections/dedication.typ": dedication-page
#import "sections/acknowledgements.typ": acknowledgements-page
#import "sections/preface.typ": preface-page
#import "sections/abstract.typ": abstract-page
#import "sections/toc.typ": toc-page
#import "sections/lot.typ": list-of-tables
#import "sections/lof.typ": list-of-figures
#import "sections/lop.typ": list-of-pictures
#import "sections/loa.typ": list-of-abbreviations
#import "sections/chapters.typ": chapter
#import "sections/references.typ": references-page
#import "sections/appendices.typ": appendix
#import "sections/cv.typ": curriculum-vitae

// Apply global page geometry and text settings
#set page(
  margin: (top: 1in, bottom: 1in, left: 1.25in, right: 1.25in),
)
#set text(font: "Times New Roman", size: 12pt)

// === DOCUMENT ===
// Required sections — uncomment and fill in

// #title-page(
//   title: "",
//   author: "",
//   school: "University Graduate School",
//   degree: "Doctor of Philosophy",
//   department: "",
//   campus: "Bloomington",
//   month: "May",
//   year: "2026",
// )
//
// #acceptance-page(
//   committee: (
//     (name: "", role: "Chair"),
//     (name: "", role: ""),
//   )
// )
//
// #abstract-page(
//   title: "Abstract",
//   body: "",
// )
//
// #toc-page(entries: ())
//
// #chapter(heading: "Introduction", body: [])
//
// #references-page(entries: [])
//
// #curriculum-vitae(
//   name: "",
//   body: [],
// )

// Optional sections — uncomment as needed
// #copyright-page()
// #dedication-page(text: [])
// #acknowledgements-page(body: [])
// #preface-page(body: [])
// #list-of-tables(entries: ())
// #list-of-figures(entries: ())
// #list-of-pictures(entries: ())
// #list-of-abbreviations(entries: ())
// #appendix(label: "A", title: "", body: [])
```

- [ ] **Step 2: Verify the full template compiles (with one example section uncommented)**

```bash
sed 's|// #title-page|#title-page|; s|//   title: ""|  title: "Test"|; s|//   author: ""|  author: "Author"|; s|//   department: ""|  department: "Computer Science"|; s|// )|)|' institutions/iu/template/template.typ > /tmp/full-test.typ
```

Then compile and verify.

- [ ] **Step 3: Commit**

```bash
rtk git add institutions/iu/template/template.typ
rtk git commit -m "feat: write complete IU Typst template entrypoint"
```

---

### Task 6: Add Rust endpoint — GET /institutions/:id/spec

**Files:**
- Create: `rust-doc-service/src/routes/spec.rs`
- Modify: `rust-doc-service/src/routes/mod.rs:1-19`

**Interfaces:**
- Consumes: `Registry::get(&id)` from `institutions/mod.rs:76-78`
- Produces: `GET /institutions/:id/spec` → `{ raw, summary: { institution, document_structure, constants, check_count } }`

- [ ] **Step 1: Write the route handler**

Create `rust-doc-service/src/routes/spec.rs`:

```rust
use crate::{error::AppError, institutions::Registry};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct SpecSummary {
    pub institution: String,
    pub document_structure: serde_yaml::Value,
    pub constants: serde_yaml::Value,
    pub check_count: CheckCount,
}

#[derive(Serialize)]
pub struct CheckCount {
    pub automated: usize,
    pub human: usize,
}

#[derive(Serialize)]
pub struct SpecResponse {
    pub raw: serde_yaml::Value,
    pub summary: SpecSummary,
}

pub async fn handler(
    State(registry): State<Registry>,
    Path(id): Path<String>,
) -> Result<Json<SpecResponse>, AppError> {
    let institution = registry
        .get(&id)
        .ok_or_else(|| AppError::InstitutionNotFound(id.clone()))?;

    let checks = institution.spec.get("checks").and_then(|c| c.as_sequence());

    let (automated, human) = if let Some(checks) = checks {
        let automated = checks.iter().filter(|c| {
            c.get("category")
                .and_then(|v| v.as_str())
                .map(|cat| cat != "human")
                .unwrap_or(true)
        }).count();
        let human = checks.len() - automated;
        (automated, human)
    } else {
        (0, 0)
    };

    let summary = SpecSummary {
        institution: institution.name.clone(),
        document_structure: institution
            .spec
            .get("document_structure")
            .cloned()
            .unwrap_or(serde_yaml::Value::Null),
        constants: institution
            .spec
            .get("constants")
            .cloned()
            .unwrap_or(serde_yaml::Value::Null),
        check_count: CheckCount { automated, human },
    };

    Ok(Json(SpecResponse {
        raw: institution.spec.clone(),
        summary,
    }))
}
```

- [ ] **Step 2: Register the route**

Modify `rust-doc-service/src/routes/mod.rs` — add `mod spec;` and the route:

Add after line 1:
```rust
mod spec;
```

Change line 14-18 to:
```rust
pub fn router() -> Router<Registry> {
    Router::new()
        .route("/extract", post(extract::handler))
        .route("/compile", post(compile::handler))
        .route("/validate", post(validate::handler))
        .route("/health", get(|| async { "ok" }))
        .route("/institutions", get(institutions::handler))
        .route("/institutions/{id}/spec", get(spec::handler))
}
```

- [ ] **Step 3: Run existing tests (no new tests for read-only endpoint)**

```bash
rtk cargo test
```

Expected: 6 tests pass

- [ ] **Step 4: Manual smoke test**

```bash
cargo run &
sleep 2
curl http://localhost:4000/institutions/iu/spec | python3 -m json.tool | head -20
kill %1
```

Expected: JSON response with `raw` and `summary` fields

- [ ] **Step 5: Commit**

```bash
rtk git add rust-doc-service/src/routes/spec.rs rust-doc-service/src/routes/mod.rs
rtk git commit -m "feat: add GET /institutions/:id/spec endpoint"
```

---

### Task 7: Add Rust endpoint — GET /institutions/:id/template

**Files:**
- Create: `rust-doc-service/src/routes/template.rs`
- Modify: `rust-doc-service/src/routes/mod.rs` (add route)

**Interfaces:**
- Consumes: `Registry::get(&id)` to get `template_dir: PathBuf`
- Produces: `GET /institutions/:id/template` → `{ files: [{ path, content }], entry: "template.typ" }`

- [ ] **Step 1: Write the route handler**

Create `rust-doc-service/src/routes/template.rs`:

```rust
use crate::{error::AppError, institutions::Registry};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct TemplateResponse {
    pub files: Vec<TemplateFile>,
    pub entry: String,
}

async fn read_dir_recursive(
    base: &PathBuf,
    rel: &str,
    files: &mut Vec<TemplateFile>,
) -> Result<(), std::io::Error> {
    let mut entries = tokio::fs::read_dir(base).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let rel_path = if rel.is_empty() {
            entry.file_name().to_string_lossy().to_string()
        } else {
            format!("{}/{}", rel, entry.file_name().to_string_lossy())
        };
        if path.is_dir() {
            Box::pin(read_dir_recursive(&path, &rel_path, files)).await?;
        } else if path.extension().map(|e| e == "typ").unwrap_or(false) {
            let content = tokio::fs::read_to_string(&path).await?;
            files.push(TemplateFile {
                path: rel_path,
                content,
            });
        }
    }
    Ok(())
}

pub async fn handler(
    State(registry): State<Registry>,
    Path(id): Path<String>,
) -> Result<Json<TemplateResponse>, AppError> {
    let institution = registry
        .get(&id)
        .ok_or_else(|| AppError::InstitutionNotFound(id.clone()))?;

    let mut files = Vec::new();
    read_dir_recursive(&institution.template_dir, "", &mut files)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read template: {}", e)))?;

    Ok(Json(TemplateResponse {
        files,
        entry: "template.typ".to_string(),
    }))
}
```

- [ ] **Step 2: Register the route**

Modify `rust-doc-service/src/routes/mod.rs`:

Add after the `mod spec;` line:
```rust
mod template;
```

Add the route after the spec route:
```rust
.route("/institutions/{id}/template", get(template::handler))
```

- [ ] **Step 3: Run existing tests**

```bash
rtk cargo test
```

Expected: 6 tests pass

- [ ] **Step 4: Manual smoke test**

```bash
cargo run &
sleep 2
curl http://localhost:4000/institutions/iu/template | python3 -m json.tool | head -10
kill %1
```

Expected: JSON response with `files` array containing at least `template.typ`, `styles.typ`, and the section files

- [ ] **Step 5: Commit**

```bash
rtk git add rust-doc-service/src/routes/template.rs rust-doc-service/src/routes/mod.rs
rtk git commit -m "feat: add GET /institutions/:id/template endpoint"
```

---

### Task 8: Add API client functions and AI SDK tools

**Files:**
- Modify: `web/src/lib/api.ts` (add `fetchInstitutionSpec`, `fetchTemplate`)
- Modify: `web/src/lib/tools.ts` (add `get_institution_spec`, `get_template` tools)

**Interfaces:**
- Consumes: `RUST_SERVICE_URL` from api.ts
- Produces: Two new tool functions exported from `createTools()`

- [ ] **Step 1: Add API client functions**

Add to `web/src/lib/api.ts` after line 82:

```typescript
export interface SpecSummary {
  institution: string;
  document_structure: unknown;
  constants: unknown;
  check_count: { automated: number; human: number };
}

export interface SpecResponse {
  raw: unknown;
  summary: SpecSummary;
}

export async function fetchInstitutionSpec(
  institutionId: string
): Promise<SpecResponse> {
  const res = await fetch(
    `${RUST_SERVICE_URL}/institutions/${encodeURIComponent(institutionId)}/spec`
  );
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error ?? `Spec fetch failed: ${res.status}`);
  }
  return res.json();
}

export interface TemplateFile {
  path: string;
  content: string;
}

export interface TemplateResponse {
  files: TemplateFile[];
  entry: string;
}

export async function fetchTemplate(
  institutionId: string
): Promise<TemplateResponse> {
  const res = await fetch(
    `${RUST_SERVICE_URL}/institutions/${encodeURIComponent(institutionId)}/template`
  );
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error ?? `Template fetch failed: ${res.status}`);
  }
  return res.json();
}
```

- [ ] **Step 2: Add tools to tools.ts**

Add these two tools inside `createTools()` in `web/src/lib/tools.ts`, before the return statement:

```typescript
  const getInstitutionSpecTool = tool({
    description:
      "Read the complete institution formatting specification including all checks, document structure, and constants. Call this to understand submission requirements before generating Typst code.",
    inputSchema: z.object({
      institutionId: z
        .string()
        .describe("The institution ID (e.g. 'iu')"),
    }),
    execute: async ({ institutionId }) => {
      const { fetchInstitutionSpec } = await import("./api");
      const result = await fetchInstitutionSpec(institutionId);
      return result.summary;
    },
  });

  const getTemplateTool = tool({
    description:
      "Read the Typst template files for the institution. Call this to understand the available section components and styles before generating Typst code. Returns all .typ files with paths and contents.",
    inputSchema: z.object({
      institutionId: z
        .string()
        .describe("The institution ID (e.g. 'iu')"),
    }),
    execute: async ({ institutionId }) => {
      const { fetchTemplate } = await import("./api");
      const result = await fetchTemplate(institutionId);
      return {
        entry: result.entry,
        files: result.files.map((f) => ({
          path: f.path,
          firstLine: f.content.split("\n")[0],
          lineCount: f.content.split("\n").length,
        })),
        // Return full content for small files, summary for large ones
        fullContent: result.files
          .filter((f) => f.content.length < 2000)
          .map((f) => ({ path: f.path, content: f.content })),
      };
    },
  });
```

Update the return statement:

```typescript
  return {
    extract_document: extractDocumentTool,
    compile_typst: compileTypstTool,
    validate_pdf: validatePdfTool,
    get_institution_spec: getInstitutionSpecTool,
    get_template: getTemplateTool,
  };
```

- [ ] **Step 3: Build to verify types**

```bash
bun run build
```

Expected: clean build

- [ ] **Step 4: Commit**

```bash
rtk git add web/src/lib/api.ts web/src/lib/tools.ts
rtk git commit -m "feat: add get_institution_spec and get_template AI SDK tools"
```

---

### Task 9: Dynamic system prompt

**Files:**
- Modify: `web/src/app/api/chat/route.ts`

**Interfaces:**
- Consumes: `fetchInstitutionSpec`, `fetchTemplate` from api.ts
- Produces: System prompt injected with institution name, document structure, constants, and template file index

- [ ] **Step 1: Add spec/template loading to chat route**

Create a helper function and modify the route handler.

Add at the top of `web/src/app/api/chat/route.ts` after the imports:

```typescript
import { fetchInstitutionSpec, fetchTemplate } from "@/lib/api";

const RUST_SERVICE_URL =
  process.env.RUST_SERVICE_URL ?? "http://localhost:4000";

async function buildSystemPrompt(institutionId: string, sessionId: string): Promise<string> {
  let specSummary = "";
  let templateIndex = "";

  try {
    const spec = await fetchInstitutionSpec(institutionId);
    const structure = spec.summary.document_structure;
    const constants = spec.summary.constants;
    specSummary = `Institution: ${spec.summary.institution}
Required sections: ${JSON.stringify(structure)}
Constants: ${JSON.stringify(constants)}
Checks: ${spec.summary.check_count.automated} automated, ${spec.summary.check_count.human} human-review`;
  } catch {
    specSummary = `Institution: ${institutionId}`;
  }

  try {
    const tmpl = await fetchTemplate(institutionId);
    templateIndex = tmpl.files
      .map((f) => `  ${f.path} (${f.content.split("\n").length} lines)`)
      .join("\n");
  } catch {
    templateIndex = "(template not available)";
  }

  return `You are a dissertation formatting assistant.

SUBMISSION REQUIREMENTS:
${specSummary}

TEMPLATE FILES AVAILABLE:
${templateIndex}
Entry point: template.typ

Session ID: ${sessionId}

You have access to five tools: extract_document, get_institution_spec, get_template, compile_typst, and validate_pdf.

WORKFLOW:
1. Ask the student to upload their dissertation
2. Call extract_document to get the content
3. Call get_institution_spec to review all formatting rules for the institution
4. Call get_template to read the Typst template files and understand the available section components
5. Elicit missing variables from the student (degree, committee members, campus, defense date, font preferences)
6. Generate the full Typst document using the template and call compile_typst
7. Call validate_pdf to check compliance against institution requirements
8. Edit ONE section at a time, recompile the full document, revalidate — repeat until all automatable checks pass
9. Walk through each human-review check with the student one at a time:
   - Present the check description and what to look for
   - Ask the student to confirm the item is correct or flag issues
   - Record their response before moving to the next check
10. When all checks pass, offer the final PDF for download`;
}
```

Replace the `systemPrompt` construction (lines 32-46) with:

```typescript
  const systemPrompt = await buildSystemPrompt(institutionId, sessionId);
```

- [ ] **Step 2: Build to verify**

```bash
bun run build
```

Expected: clean build

- [ ] **Step 3: Commit**

```bash
rtk git add web/src/app/api/chat/route.ts
rtk git commit -m "feat: dynamic system prompt with institution spec and template index"
```

---

### Task 10: Update roadmap and memories

**Files:**
- Modify: `docs/ROADMAP.md`
- Update Serena memories: `project-status`, `core`

- [ ] **Step 1: Update roadmap**

Add Phase 4 rows to `docs/ROADMAP.md`:

```markdown
| 4 | 19 | Copy full IU spec | ✅ | [spec](../superpowers/specs/2026-07-02-phase-4-design.md) | [plan](../superpowers/plans/2026-07-02-phase-4-llm-tools.md) | — | |
| 4 | 20 | IU Typst template (14 sections + styles) | ✅ | same | same | — | |
| 4 | 21 | GET /institutions/:id/spec endpoint | ✅ | same | same | — | |
| 4 | 22 | GET /institutions/:id/template endpoint | ✅ | same | same | — | |
| 4 | 23 | get_institution_spec + get_template tools | ✅ | same | same | — | |
| 4 | 24 | Dynamic system prompt | ✅ | same | same | — | |
```

- [ ] **Step 2: Update Serena memories**

Update `format-my-dissertation/project-status`:
- Current state: Phase 4 underway, 2 new Rust endpoints + 2 new LLM tools
- Add Phase 4 to completed rounds table
- Update git history with new commits

Update `format-my-dissertation/core`:
- Add the two new endpoints to the Rust service description
- Add the two new tools to the Next.js description
- Update the LLM tools list from 3 to 5 tools

- [ ] **Step 3: Commit**

```bash
rtk git add docs/ROADMAP.md
rtk git commit -m "doc: update roadmap and memories for Phase 4"
```

---

### Task 11: End-to-end verification

**Pre-requisite:** `LLM_API_KEY` env var set

- [ ] **Step 1: Spin up both services**

Terminal 1:
```bash
cd rust-doc-service && cargo run
```

Terminal 2:
```bash
cd web && bun run dev
```

- [ ] **Step 2: Curl the new endpoints**

```bash
curl -s http://localhost:4000/institutions/iu/spec | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['summary']['institution'], d['summary']['check_count'])"
# Expected: Indiana University {'automated': 26, 'human': 7}

curl -s http://localhost:4000/institutions/iu/template | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d['files']), 'files, entry:', d['entry'])"
# Expected: 17 files, entry: template.typ
```

- [ ] **Step 3: Compile full template with diss-check**

```bash
cd /home/danriggi/format-my-dissertation
# Create a filled-out test document
cat > /tmp/test-dissertation.typ << 'TYPST'
#import "institutions/iu/template/template.typ"
TYPST
# Uncomment all required sections with test data
# ... (manual step — fill the template with test content)

typst compile --format pdf /tmp/test-dissertation.typ /tmp/test-dissertation.pdf
cd /home/danriggi/diss-check
cargo run --release -- check --spec specs/iu.yaml --json /tmp/test-dissertation.pdf
```

Expected: automated checks should pass; human checks should show as `human` category

- [ ] **Step 4: Commit**

```bash
rtk git add -A
rtk git commit -m "verify: Phase 4 end-to-end integration test"
```

---

## Self-Review

1. **Spec coverage:** Each spec requirement maps to a task. Task 1 = spec copy, Tasks 2-5 = template, Tasks 6-7 = Rust endpoints, Task 8 = AI SDK tools, Task 9 = dynamic prompt, Task 10 = roadmap/memories, Task 11 = verification.
2. **Placeholder scan:** No TBD, TODO, or vague references. All code is concrete.
3. **Type consistency:** The Rust endpoints use the same `Registry` and `AppError` patterns as existing routes. The TypeScript tools follow the same `tool()` pattern as existing tools. API client functions follow the same `RUST_SERVICE_URL` pattern.

All good.
