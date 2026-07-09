# Phase 6 — LLM-Accessible Document Extraction

**Date:** 2026-07-03
**Status:** In design

## Problem

After uploading a dissertation, the `extract_document` tool returns the full 532K+ character raw text as a single JSON string. This is unusable by the LLM (exceeds context window, can't be processed). The tool result is effectively discarded. Additionally, the Rust parser now produces heading/structure information that is thrown away — only raw text is stored in the in-memory store.

Two symptoms:
1. User has to prompt the LLM to proceed after upload (it stalls because it doesn't have usable document content)
2. LLM doesn't understand the document was extracted and asks the user to "upload using the paperclip button"

## Fix

### Store full extraction result

The in-memory store currently holds only `extractedText: string`. Change to hold the full `ExtractResult` including `structure.headings`, `metadata`, and `content.raw_text`.

```typescript
interface SessionState {
  pdf: Uint8Array | null;
  extraction: StoreExtractResult | null;  // replaces extractedText
  violations: Array<...>;
  passCount: number;
  failCount: number;
}

interface StoreExtractResult {
  raw_text: string;
  headings: Array<{ text: string; level: number; page_number: number | null }>;
  page_count: number;
  page_count_estimated: boolean;
  detected_fonts: string[];
}
```

### Rework extract_document tool

Instead of returning raw text, return a structural summary + metadata:

```json
{
  "page_count": 273,
  "page_count_estimated": true,
  "detected_fonts": ["Times New Roman"],
  "headings": [
    { "text": "DEDICATION", "level": 1, "page": null },
    { "text": "ACKNOWLEDGEMENTS", "level": 1, "page": null },
    ...
  ],
  "total_chars": 532115,
  "available_chunks": 42,
  "first_chunk": "<first 4000 chars of raw text>"
}
```

The `first_chunk` gives the LLM immediate context (title page, abstract start) so it can answer basic questions without an extra tool call.

### New get_document_chunks tool

Returns specific chunks of the raw text by index or heading name:

```json
// Input: { start_index: 3, count: 2 }
// Output:
{
  "chunks": [
    { "index": 3, "text": "<chunk text>", "char_range": [12000, 20000] },
    { "index": 4, "text": "<chunk text>", "char_range": [19500, 28000] }
  ]
}
```

```json
// Input: { heading: "Chapter 1" }
// Output:
{
  "heading": "Chapter 1: The Erosion of the American Promise...",
  "level": 1,
  "chunks": [{ "index": 12, "text": "<chapter text start>", "char_range": [45000, 50000] }]
}
```

Chunks are 5000 chars, 500 overlap, split on paragraph boundaries. Total chunk count is shipped to the LLM in extract_document so it can request ranges.

### Slimmer upload message

The chat message on upload changes from:
```
I've uploaded my dissertation: NAME.docx (273 pages estimated)

Extracted content:
<8000 chars of text>
```

To:
```
I've uploaded my dissertation: NAME.docx (273 pages estimated)
```

The LLM gets the text content from `extract_document` and `get_document_chunks` — not from the upload message.

### Updated system prompt (steps 2-6)

```
2. Call extract_document to get the document's page count, headings, and structure.
3. Present findings to the student: list detected sections, state the page count, and ask for
   confirmation that the extraction looks correct.
4. After student confirms, call get_institution_spec to review formatting requirements.
5. Call get_template to read the available Typst template files.
6. Elicit missing variables from the student one at a time (degree, committee members, campus,
   defense date, font preferences). Ask one question per response.
```

The key addition is step 3: "present findings and ask for confirmation." This prevents the LLM from racing ahead and gives the user a natural checkpoint.

### Files changed

- `src/lib/store.ts` — add `StoreExtractResult` type, store/replace extraction
- `src/components/chat-panel.tsx` — store full result, remove truncated text from upload message
- `src/lib/tools.ts` — rewrite `extract_document`, add `get_document_chunks`
- `src/app/api/chat/route.ts` — update system prompt steps 2-6

No Rust changes, no new dependencies.
