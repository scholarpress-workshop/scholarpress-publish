# Design Spec: Synthetic Margin Test Suite

## Motivation

The full-width line margin checker (Round 41) correctly measures margins against
real ScholarWorks dissertations and the template-test. But there is no test
fixture that *deliberately* sets wrong margins to confirm the checker FAILs when
it should. All existing test PDFs are either real dissertations (with genuine
formatting issues) or the template-test (with correct margins but placeholder
content). A synthetic test suite provides deterministic boundary testing.

## Structure

Two Typst source documents, both using 12pt Libertinus Serif, US Letter.
Directory: `diss-check/tests/fixtures/synthetic/`.

### `synthetic-body.typ`

Shared import. Defines a `body` function that renders `#lorem(paragraphs: 200)`
in single-spaced body text. No headings, tables, or figures — pure text for
testing the measurement math in isolation.

### Variant wrappers

Nine one-line `.typ` files that import `synthetic-body.typ` and override
`#set page(margin: ...)`:

| Wrapper | Margins (L/R/T/B) | Output PDF | Purpose |
|---------|--------------------|------------|---------|
| `baseline-wrapper.typ` | 1.25 / 1.25 / 1.0 / 1.0 | `baseline.pdf` | All correct |
| `left-narrow.typ` | 0.75 / 1.25 / 1.0 / 1.0 | `left-narrow.pdf` | Left too narrow |
| `right-narrow.typ` | 1.25 / 0.75 / 1.0 / 1.0 | `right-narrow.pdf` | Right too narrow |
| `left-wide.typ` | 1.75 / 1.75 / 1.0 / 1.0 | `left-wide.pdf` | Left too wide (symmetric) |
| `right-wide.typ` | 1.75 / 1.75 / 1.0 / 1.0 | `right-wide.pdf` | Right too wide (symmetric) |
| `top-narrow.typ` | 1.25 / 1.25 / 0.5 / 1.0 | `top-narrow.pdf` | Top too narrow |
| `bottom-narrow.typ` | 1.25 / 1.25 / 1.0 / 0.5 | `bottom-narrow.pdf` | Bottom too narrow |
| `top-wide.typ` | 1.25 / 1.25 / 2.0 / 1.0 | `top-wide.pdf` | Top too wide |
| `asymmetric.typ` | 1.50 / 1.00 / 1.0 / 1.0 | `asymmetric.pdf` | Both sides wrong + asymmetric |

Wide variants use symmetric margins (L=R=1.75) to isolate `global_margins` from
`margin_symmetry`. Only `asymmetric.pdf` triggers both checkers.

### `synthetic-messy.typ`

Standalone document with correct margins (L=R=1.25, T=B=1.0). Three pages of
mixed content testing the full-width line filter against real formatting:

| Page | Content | What it tests |
|------|---------|---------------|
| 1 | Centered heading at 2in, 2 ragged-right body paragraphs | Top margin from heading; left/right from body lines |
| 2 | 1 body paragraph + centered figure (`#rect` + caption) + centered table | Filter rejects narrow table/figure lines |
| 3 | 2 centered dedication-style lines, no body text | Sparse page skipped (< 3 full-width lines) |

Compiles to `messy.pdf`.

## Expected results

| PDF | `global_margins` | `margin_symmetry` | Failing dimension(s) |
|-----|-------------------|-------------------|---------------------|
| `baseline.pdf` | PASS | PASS | — |
| `left-narrow.pdf` | FAIL | PASS | left 54pt < 81pt |
| `right-narrow.pdf` | FAIL | PASS | right 54pt < 81pt |
| `left-wide.pdf` | FAIL | PASS | left 126pt > 99pt |
| `right-wide.pdf` | FAIL | PASS | right 126pt > 99pt |
| `top-narrow.pdf` | FAIL | PASS | top 36pt < 63pt |
| `bottom-narrow.pdf` | FAIL | PASS | bottom 36pt < 63pt |
| `top-wide.pdf` | FAIL | PASS | top 144pt > 81pt |
| `asymmetric.pdf` | FAIL | FAIL | left 108pt > 99pt, right 72pt < 81pt, diff 36pt > 18pt |
| `messy.pdf` | PASS | PASS | — |

`messy.pdf` passes because page 3 is skipped by the `< 3 full-width lines`
fallback for ALL dimensions (top/bottom measurement is already inside the
same guard).

## Integration test

New file `tests/synthetic_margin_test.rs`. One `#[test]` function iterating the
table above. Uses `diss_check::engine::run_checks` and
`diss_check::report::build_report`, matching the existing integration test
pattern. Asserts expected `Status` for both `global_margins` and
`margin_symmetry` per variant.

## Compilation

`tests/fixtures/synthetic/compile.sh` — bash script running `typst compile`
for each variant wrapper and `synthetic-messy.typ`. Run once to generate all
10 PDFs. PDFs are committed to the repo (no Typst dependency in CI).

## Files changed

| File | Change |
|------|--------|
| `tests/fixtures/synthetic/synthetic-body.typ` | New — shared body text source |
| `tests/fixtures/synthetic/synthetic-messy.typ` | New — mixed-content source |
| `tests/fixtures/synthetic/*-wrapper.typ` | New — 9 variant wrappers (or inline in compile.sh) |
| `tests/fixtures/synthetic/*.pdf` | New — 10 compiled PDFs (committed) |
| `tests/fixtures/synthetic/compile.sh` | New — regeneration script |
| `tests/synthetic_margin_test.rs` | New — integration test |
| `src/checkers/layout.rs` | No change needed (sparse-page guard already covers all dims) |

## Self-review

1. **Placeholder scan:** No TBDs, TODOs, or incomplete sections.
2. **Internal consistency:** Margin values match expected 5th percentile
   output (72pt = 1in, 36pt = 0.5in, etc. at 1pt = 1/72in). Tolerance is
   ±0.125in (±9pt). All FAIL conditions correctly computed.
3. **Scope check:** 12 files total (10 PDFs + 2 source .typ + compile.sh).
   Integration test is one function. Well-scoped.
4. **Ambiguity check:** Typst font availability (Libertinus Serif) — may
   need to use built-in font or install font package. Spacing
   (single-spaced) — lorem paragraph length needs to produce dense pages.
   Compile.sh needs `--root` flag so imports resolve correctly.
