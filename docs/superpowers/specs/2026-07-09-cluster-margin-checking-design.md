# Design Spec: Cluster-Based Margin Checking

## Motivation

`global_margins` and `margin_symmetry` checkers use a 5th-percentile heuristic that
fails on pages with substantial centered content (headings, tables, figures). The
template-test.typ produces false FAILs even though Typst page margins are set correctly.
Three real ScholarWorks dissertations exhibit the same false positives.

## Change: Replace percentile with clustering

Replace `left_edge_ptile()` and `right_margin_ptile()` with a shared `dominant_cluster()`
function.

### `dominant_cluster(values: &[f32], proximity: f32, min_count: usize) -> Option<f32>`

1. Sort values
2. Walk sorted list forming clusters: a value within `proximity` pt of the previous
   cluster's center joins it; otherwise starts a new cluster
3. Each cluster tracks `center` (mean of members) and `count`
4. Return `center` of the cluster with highest `count`
5. If winning cluster has fewer than `min_count` members, return `None`

**Parameters:** `proximity = 4pt`, `min_count = 5` (global_margins) / `10` (margin_symmetry)

## Changes to `MarginsChecker.check()`

```
for each page:
  - filter spans: same header/footer/short-text filter as today
    (top >= 36pt, bottom <= height - 53pt, text.len() >= 3)
  - collect x0 values for all qualifying spans
  - collect (page_width - x1) values for all qualifying spans
  - skip page if it's page 1 (title page), or matches
    acceptance/copyright/dedication keywords
  - left_edge  = dominant_cluster(x0s, 4, 5)
  - right_margin = dominant_cluster(right_gaps, 4, 5)
  - top edge = min(body span bbox.0)    [UNCHANGED]
  - bottom margin = page_height - max(body span bbox.1)  [UNCHANGED]

collect per-page cluster centers, then mean across pages,
check against required +- tolerance
```

Top/bottom measurement unchanged — the existing min/max approach is correct for
top/bottom margins.

## Changes to `MarginSymmetryChecker.check()`

```
for each page:
  - same span filter + page exclusion
  - left_cluster = dominant_cluster(x0s, 4, 10)
  - right_cluster = dominant_cluster(right_gaps, 4, 10)
  - diff = left_cluster - right_cluster
  - if |diff| > threshold => asymmetrical

Pass if 0 asymmetrical pages
```

Note `min_count` = 10 for symmetry (same 10-span threshold as today), vs 5 for
`global_margins`.

## Page exclusion

| Page type       | Detection                                          |
|-----------------|----------------------------------------------------|
| Title page      | Page 1 (always excluded)                           |
| Acceptance page | `find_section_pages(doc, &["accepted by"])`        |
| Copyright page  | `find_section_pages(doc, &["©", "copyright"])`     |
| Dedication page | `find_section_pages(doc, &["dedication"])`         |

`find_section_pages` is made `pub(crate)` (currently `fn` -> `pub(crate) fn` in
`sections.rs`).

## Files changed

| File                       | Change                                                                                     |
|----------------------------|--------------------------------------------------------------------------------------------|
| `src/checkers/layout.rs`   | Replace `left_edge_ptile` + `right_margin_ptile` with `dominant_cluster`. Add page-exclusion logic. Update both `check()` methods. Rewrite unit tests. |
| `src/checkers/sections.rs` | Make `find_section_pages` `pub(crate)` (one keyword change, no logic change)               |

No changes to spec YAML, extractor, document model, or checker registry.

## Unit tests (in `layout.rs` `mod tests`)

| Test                              | Scenario                                                              |
|-----------------------------------|-----------------------------------------------------------------------|
| `test_cluster_basic`              | 20x90pt + 10x180pt => cluster picks 90pt                              |
| `test_cluster_mixed_indent`       | Body 90pt, block quotes 120pt, headings 200pt => 90pt wins            |
| `test_cluster_all_centered`       | All 180pt => graceful degradation, returns 180pt                      |
| `test_cluster_too_small`          | 3 spans (< min_count 5) => None (page skipped)                        |
| `test_cluster_exact_min`          | 5 spans exactly at min => returns center                              |
| `test_margins_clustered_pass`     | Multi-page, body clusters at 90pt / 518pt => PASS                     |
| `test_margins_clustered_fail`     | Multi-page, body clusters at wrong margins => FAIL                    |
| `test_margins_page_exclusions`    | Title page (pg1), acceptance, copyright, dedication => excluded, rest measured |
| `test_symmetry_clustered_pass`    | Balanced L/R clusters => PASS                                         |
| `test_symmetry_clustered_fail`    | Asymmetric clusters => FAIL                                           |

Existing integration tests (`test_run_against_chambers`, `test_run_against_alexander`)
unchanged — both assert FAIL, which remains correct.
