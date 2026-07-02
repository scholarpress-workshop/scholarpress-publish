# Phase 4 — Institution Spec, Template, and LLM Tools

**Date:** 2026-07-02
**Status:** Approved

## Overview

Wire the LLM to understand institution formatting rules and Typst template structure. Write the complete IU Typst template (14 section files). Add `get_institution_spec` and `get_template` tools so the LLM can read requirements on-demand. Make the system prompt dynamic so the LLM starts with full context instead of just an institution ID string.

## Architecture changes

### Two new Rust endpoints

**GET `/institutions/:id/spec`**
```
→ { raw: string, summary: {
    institution: string,
    document_structure: { front_matter, body, end_matter },
    constants: { ... },
    check_count: { automated: number, human: number }
  }}
```

**GET `/institutions/:id/template`**
```
→ {
    files: [{ path: string, content: string }],
    entry: "template.typ"
  }
```

### Two new AI SDK tools

- **`get_institution_spec`** — LLM reads submission rules, required sections, formatting checks, constants. Backed by `GET /institutions/:id/spec`.
- **`get_template`** — LLM reads the full Typst template (all section files and styles). Backed by `GET /institutions/:id/template`.

### Existing tools unchanged

`extract_document`, `compile_typst`, and `validate_pdf` keep their current signatures. The LLM always compiles the full document and validates the full PDF — it edits one section at a time between cycles.

## Dynamic system prompt

At request time, the chat route loads the institution spec summary and template file index and injects them into the prompt:

```
You are a dissertation formatting assistant for {name}.

SUBMISSION REQUIREMENTS:
{structure_summary}
- Required: title page, acceptance page, abstract, body (all chapters), TOC, references, CV
- Optional: copyright, dedication, acknowledgements, preface, lists of tables/figures...
- Constants: {constants}

TEMPLATE FILES AVAILABLE:
{template_index}

WORKFLOW:
1. Ask the student to upload their dissertation
2. Call extract_document to get the content
3. Call get_institution_spec to review all formatting rules
4. Call get_template to read the Typst template and styles
5. Elicit missing variables (degree, committee, campus, defense date, font preference)
6. Generate the full Typst document and call compile_typst
7. Call validate_pdf to check compliance
8. Edit ONE section at a time, recompile, revalidate — repeat until all automatable checks pass
9. Walk through each human-review check with the student one at a time:
   - Present the check description and what to look for
   - Ask the student to confirm or flag issues
   - Record their response before moving to the next
10. When all checks pass, offer the final PDF for download
```

## Complete IU Typst template

Based on the 33 diss-check checks for IU and the reference DOCX at `diss-check/specs/artifacts/iu/formatting-template.docx`.

### File structure

```
institutions/iu/template/
  template.typ          # entrypoint — sets page config, includes all required sections
  styles.typ            # shared styles — fonts, spacing, headings, page numbering
  sections/
    title-page.typ      # title (all caps, no bold), clause (centered, specific wording), no page number
    acceptance.typ      # committee list (chair first), page number ii (no signatures)
    copyright.typ       # optional — copyright page format
    dedication.typ      # optional
    acknowledgements.typ # optional (recommended)
    preface.typ         # optional
    abstract.typ        # required — title format, centered text, unsigned
    toc.typ             # required — page number alignment, no overhang, CV no dots
    lot.typ             # optional — List of Tables
    lof.typ             # optional — List of Figures
    lop.typ             # optional — List of Pictures/Schematics
    loa.typ             # optional — List of Abbreviations
    chapters.typ        # required — body content, consistent headings, new chapter = new page
    references.typ      # required — heading format, font consistent
    appendices.typ      # optional
    cv.typ              # required — heading format, name position, no page number, no credentials/PII
```

### Design principles

- Each section is a function accepting variables (title, author, committee members, chapter content, etc.) so the LLM populates them programmatically.
- Automated checks (margins, fonts, page numbers, heading consistency, clause wording, etc.) pass by construction — the template embeds the correct values.
- Human-check items (clause spacing, footnote spacing, TOC spacing, references spacing, CV-no-credentials, table/figure legend font) are structured as comments in the template source for the LLM to surface during the walkthrough.

## Data flow

```
Upload DOCX
  → extract_document (kreuzberg)
  → LLM receives extracted content
  → LLM calls get_institution_spec (learns rules)
  → LLM calls get_template (reads .typ files)
  → LLM maps content → template functions
  → section-by-section Typst generation
  → compile_typst (full PDF each time)
  → validate_pdf
  → fix one section → recompile → revalidate (loop)
  → walk through human checks with student
  → final PDF download
```

## Execution order

| # | Step | Dependencies |
|---|---|---|
| 1 | Copy full IU spec from diss-check → `institutions/iu/spec.yaml` | none |
| 2 | Write complete IU Typst template (14 section files + styles) | spec for reference |
| 3 | `GET /institutions/:id/spec` — Rust endpoint | step 1 |
| 4 | `GET /institutions/:id/template` — Rust endpoint | step 2 |
| 5 | `get_institution_spec` — AI SDK tool | step 3 |
| 6 | `get_template` — AI SDK tool | step 4 |
| 7 | Dynamic system prompt (inject spec + template) | steps 3, 4 |
| 8 | End-to-end test with LLM API key | all |

## Long-term goals (not this phase)

- Spec and template registry for tracking versions and updates across institutions
- Postgres schema for Document, TypstSnapshot, ValidationRun tables
- S3/MinIO file storage for uploaded dissertations and generated PDFs
- Refinement loop guardrails (max iterations, stuck violation detection)
- PDF page navigation in the preview panel
- Vercel deployment
