# build_document Tool — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `build_document` tool that lets the LLM specify Typst structure with `{MARKER}` placeholders while the backend fetches content from stored extraction chunks, escapes Typst special characters, and compiles — keeping LLM output well under token limits.

**Architecture:** All logic runs in the Next.js TypeScript tool layer (extraction data lives there). A new `build_document` tool fetches chunks from `getStoredExtraction`, escapes Typst-special characters, substitutes markers, then delegates to the existing `compileTypst` for compilation. No new Rust endpoint.

**Tech Stack:** TypeScript (Next.js API routes), existing `compileTypst` Rust endpoint

## Global Constraints

- No new Rust endpoints or modules
- No changes to template files or institution config
- `extract_document`, `get_document_chunks`, `get_institution_spec`, `get_template`, `validate_pdf` — all unchanged
- Existing `compile_typst` tool remains available for refinement re-submissions

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `web/src/lib/tools.ts` | Modify | Add `build_document` tool + `escapeTypstText()` + `assembleDocument()` |
| `web/src/app/api/chat/route.ts` | Modify  | Update system prompt step 6 to reference `build_document` |
| `web/src/lib/store.ts` | Modify | Add `getChunks(sessionId, indices)` helper |

---

### Task 1: Escape helper + assembleDocument + build_document tool

**Files:**
- Modify: `web/src/lib/tools.ts:1-6` (imports) and after line 100 (after `compileTypstTool`)

**Interfaces:**
- Consumes: `getStoredExtraction` from `@/lib/store`, `compileTypst` from `@/lib/api`, `chunkDocument` (existing, same file)
- Produces: `buildDocumentTool` — a new `tool()` that is added to the returned tools object

- [ ] **Step 1: Add `escapeTypstText` function**

Insert after the `compileTypstTool` definition (after line 100), before the tools return object:

```typescript
function escapeTypstText(text: string): string {
  return text
    .replace(/\\/g, "\\\\")
    .replace(/#/g, "\\#")
    .replace(/\[/g, "\\[")
    .replace(/\]/g, "\\]");
}
```

- [ ] **Step 2: Add `assembleDocument` function**

Insert after `escapeTypstText`:

```typescript
function getChunksFromText(
  rawText: string,
  indices: number[],
  charsPerChunk: number,
  overlap: number
): string {
  const allChunks = chunkDocument(rawText, charsPerChunk, overlap);
  return indices
    .filter((i) => i >= 0 && i < allChunks.length)
    .map((i) => allChunks[i].text)
    .join("\n\n");
}

function assembleDocument(
  typstStructure: string,
  sectionChunks: Record<string, number[]>,
  rawText: string
): string {
  let result = typstStructure;
  for (const [marker, indices] of Object.entries(sectionChunks)) {
    const chunkText = getChunksFromText(rawText, indices, 5000, 500);
    const escaped = escapeTypstText(chunkText);
    result = result.replace(
      new RegExp(`\\{${marker}\\}`, "g"),
      `[${escaped}]`
    );
  }
  return result;
}
```

- [ ] **Step 3: Add `build_document` tool definition**

Insert after `assembleDocument`, before the return statement:

```typescript
const buildDocumentTool = tool({
  description:
    "Build and compile the full Typst document by combining the LLM's structure with text from stored extraction chunks. Use {MARKER} placeholders in typst_structure for content — the backend fetches the text, escapes special characters, and substitutes before compiling. Short fields (title, author, dates) should be literal strings in typst_structure, not markers.",
  inputSchema: z.object({
    typst_structure: z
      .string()
      .describe(
        "The complete Typst assembly code (imports + function calls) with {MARKER} placeholders where body text goes. Short fields should be literal strings."
      ),
    section_chunks: z
      .record(z.array(z.number()))
      .describe(
        "Map of marker names to chunk index arrays. e.g. { INTRO: [4,5,6], CV: [100,101] }"
      ),
    institutionId: z
      .string()
      .describe("The institution ID (e.g. 'iu')"),
  }),
  execute: async ({ typst_structure, section_chunks, institutionId }) => {
    const extraction = getStoredExtraction(sessionId);
    if (!extraction) {
      return {
        error:
          "No document has been extracted yet. Ask the student to upload their dissertation file first.",
      };
    }

    const assembled = assembleDocument(
      typst_structure,
      section_chunks,
      extraction.raw_text
    );

    const pdfBuffer = await compileTypst(assembled, institutionId);
    storePdf(sessionId, new Uint8Array(pdfBuffer));
    return {
      success: true,
      pdfSize: new Uint8Array(pdfBuffer).byteLength,
    };
  },
});
```

- [ ] **Step 4: Add `build_document` to the returned tools object**

In the return statement at line ~174, add the new tool:

```typescript
return {
  extract_document: extractDocumentTool,
  get_document_chunks: getDocumentChunksTool,
  compile_typst: compileTypstTool,
  validate_pdf: validatePdfTool,
  get_institution_spec: getInstitutionSpecTool,
  get_template: getTemplateTool,
  build_document: buildDocumentTool,   // ADD THIS LINE
};
```

- [ ] **Step 5: Run typecheck**

```bash
bunx tsc --noEmit
```
Expected: no output (passes cleanly)

- [ ] **Step 6: Commit**

```bash
git add web/src/lib/tools.ts
git commit -m "feat: add build_document tool with chunk substitution"
```

---

### Task 2: Update system prompt to reference build_document

**Files:**
- Modify: `web/src/app/api/chat/route.ts:54` (tools list) and line 63 (workflow step 6)

- [ ] **Step 1: Add build_document to tools list in prompt**

Change line 54 from:
```typescript
You have access to six tools: extract_document, get_document_chunks, get_institution_spec, get_template, compile_typst, and validate_pdf.
```
To:
```typescript
You have access to seven tools: extract_document, get_document_chunks, get_institution_spec, get_template, build_document, compile_typst, and validate_pdf.
```

- [ ] **Step 2: Update workflow step 6**

Change line 63 from:
```
6. Generate the complete Typst document using the template, filling in all variables and content. Call compile_typst.
```
To:
```
6. Generate the complete Typst assembly (imports + section function calls with variable values). For body content (chapters, abstract, CV, acknowledgements, appendices), use {MARKER} placeholders mapped to chunk indices. Call build_document to assemble and compile.
```

- [ ] **Step 3: Run typecheck**

```bash
bunx tsc --noEmit
```
Expected: no output (passes cleanly)

- [ ] **Step 4: Commit**

```bash
git add web/src/app/api/chat/route.ts
git commit -m "chore: update system prompt for build_document tool"
```
