# build_document Tool — Design Spec

**Date:** 2026-07-03
**Status:** Approved

## Problem

The LLM generates the entire Typst document as a monolithic string, hitting output token limits on large dissertations (>500 lines, 273+ pages). Content is copy-pasted from `get_document_chunks` results into Typst code, but even copy-pasting hits token ceilings. Result: unclosed delimiters, truncated output, failed compilations.

## Design

### Architecture

Three components work together:

1. **LLM** — generates lightweight Typst structure (~100 lines) with `{MARKER}` placeholders where content goes
2. **`build_document` tool** (new) — replaces placeholders with escaped text from stored extraction chunks, then compiles
3. **Backend** — fetches chunks, escapes Typst special characters (`\`, `#`, `[`, `]`), substitutes, compiles

### Data flow

```
LLM calls extract_document → gets headings + chunk count
LLM calls get_document_chunks → peeks at content to identify sections
LLM calls get_template → knows function signatures
LLM calls build_document({
    typst_structure: "#import...\n#title-page(...)\n#chapter(heading: \"Intro\", body: [{INTRO}])",
    section_chunks: { "INTRO": [4,5,6] },
    institutionId: "iu",
})
         ↓
Backend: get chunks 4-6 → escape → substitute {INTRO} → compile → store PDF
         ↓
LLM calls validate_pdf → fix violations → re-submit build_document with edit → iterate
```

### Why the LLM generates the assembly (not the backend)

The LLM sees template files via `get_template`, knows function signatures, and can skip optional sections. The backend never hardcodes section names or signatures. Adding a new institution requires zero code changes — this preserves the core invariant: "institutions are directories, not code."

### Tool schema

```
build_document({
    typst_structure: string,           // full Typst assembly with {MARKER} placeholders
    section_chunks: {                  // maps marker name → chunk index ranges
        "INTRO": [4,5,6,7],           // chunks 4 through 7
        "CV": [100,101,102],
        "CHAPTER_1": [12,13,14,15,16,17],
    },
    institutionId: string,
})

Returns: { success: true, pdfSize: number } | tool error (Typst stderr)
```

### Backend flow (Rust `POST /build`)

1. For each `{MARKER}` in `section_chunks`, fetch chunk indices from stored `raw_text` via `chunkDocument()`
2. Join chunk text with `\n\n`
3. Escape Typst-special characters in the joined text: `\` → `\\`, `#` → `\#`, `[` → `\[`, `]` → `\]`
4. Replace `{MARKER}` in `typst_structure` with `[escaped text]` (wrapped in Typst content block for safety)
5. Write assembled Typst to a temp `.typ` file in the institution's template directory
6. Run `typst compile` on the temp file (with `--root <template_dir>`)
7. Store resulting PDF bytes in-memory (same session store as `compile_typst`)
8. Return success or Typst stderr

### Error handling

| Failure mode | Behavior |
|---|---|
| Typst syntax error (typo, unclosed bracket) | Returns Typst stderr to LLM; LLM fixes `typst_structure` in next call |
| Marker in `typst_structure` but not in `section_chunks` | Marker stays as `{MISSING}` in source; Typst error; LLM fixes mapping |
| Chunk index out of range | Partial or empty content; Typst may error on empty body; LLM fixes range |
| Raw Typst chars in dissertation text | Backend escapes automatically (`\[`, `\#`, `\\`); no LLM action needed |
| Empty section (no chunks matched) | Section gets empty content block `[]`; LLM adjusts chunk range |

### Refinement loop

After validation finds violations, the LLM re-submits `build_document` with the edited `typst_structure` or adjusted `section_chunks`, then calls `validate_pdf`. Backend recompiles the full document each time. For small edits (typo in title, committee name), values stay directly in `typst_structure` as literal strings — no chunk table needed.

Short fields already resolved during variable elicitation (step 4 in workflow) stay as literal strings and never use chunk markers.

### Implementation scope

**Backend (Rust):**
- New endpoint `POST /build` — fetch chunks, escape, substitute, compile
- Helper: `escape_typst_text(text: &str) -> String` — escapes `\`, `#`, `[`, `]`
- Store helper: `fetchChunks(sessionId, indices[])` — re-chunks from stored `raw_text`

**Frontend (TypeScript):**
- New `build_document` tool in `tools.ts` — calls Rust `/build` endpoint
- Session store: `getChunks(sessionId, indices)` — reads `getStoredExtraction().raw_text` and re-chunks
- System prompt: update step 6 to reference `build_document` instead of `compile_typst`

**Unchanged:**
- `extract_document`, `get_document_chunks`, `get_institution_spec`, `get_template`, `validate_pdf`
- All template files
- `compile_typst` tool (still available for refinement re-submissions)
- `compile::compile()` and `compile::template` modules
