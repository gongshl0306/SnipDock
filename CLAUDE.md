# CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

# SnipDock — Project Context

## Product

SnipDock is a desktop snippet manager: users save commands / code / paths / templates, organize them in user-defined categories, and recall them via fast keyboard-driven search. The full product spec lives in [docs/target.md](docs/target.md) — **treat it as the source of truth**. When the spec and this file disagree, the spec wins. UI reference: [UI.png](UI.png).

## Repo state

Greenfield. As of this writing the repo contains only `CLAUDE.md`, `README.md`, `docs/target.md`, and `UI.png` — no source, no build tooling, no tests. Don't fabricate commands or file paths that don't exist; if something needs scaffolding, scaffold it explicitly rather than pretending it's already there.

Planned stack (from the spec, not yet committed): Tauri shell, SQLite for persistence, Vue + TypeScript on the frontend, Rust commands on the backend. Schema, TS types, and Rust command names are pinned in [docs/target.md](docs/target.md) §5–§7 — match them verbatim when implementing, don't redesign.

## First-version constraints (non-negotiable for v1)

The spec is deliberately conservative. Section §15 forbids over-engineering:

- **No tag feature in v1** — there is no `tags` column, no `tags` table, no `snippet_tags` join table. Categories are the only organization mechanism.
- **Search runs in frontend memory** over the loaded snippet list — do **not** add SQL `LIKE`-fan-out, FTS5, or other server-side search machinery.
- **One-level categories only** — no category tree, no parent_id, no nesting.
- **No advanced query syntax** (`category:bar` etc.) — case-insensitive substring match across `title / content / category_name` is the whole feature.
- **Default category is `默认` only** — don't seed extra example categories on first launch.
- **Deleting a non-empty category must fail** with `Category is not empty`. The "delete and cascade" / "delete and reassign" variants are explicitly deferred.

If a request seems to push against these, surface the conflict before implementing — per §1 of the behavioral guide above. Don't silently "improve" the spec by adding scope.

## Contracts that must match the spec exactly

When you write the code, these names and shapes are fixed by [docs/target.md](docs/target.md) and are the public contract between layers — change one and you have to change all of them together:

- **DB columns** (§5.1, §5.2) — `categories(id, name, description, sort_order, created_at, updated_at)` and `snippets(id, category_id FK, title, content, favorite, used_count, created_at, updated_at, last_used_at)`. v1 has **no** tags / description / language columns on `snippets`.
- **Indexes** (§5.4) — `idx_snippets_category_id`, `idx_snippets_updated_at`, `idx_snippets_used_count`, `idx_snippets_last_used_at`.
- **TS types** (§6) — `Category`, `Snippet`, `CreateSnippetPayload`, `UpdateSnippetPayload`. The optional `category_name` on `Snippet` is populated by joins for global-search display.
- **Rust commands** (§7) — `list_categories / create_category / update_category / delete_category` and `list_snippets / list_snippets_by_category / search_snippets / create_snippet / update_snippet / delete_snippet / copy_snippet / mark_snippet_used`. Don't rename, don't merge, don't add until v1 is complete.
- **`search_snippets` semantics** (§7.3) — `categoryId === null` ⇒ global search; numeric `categoryId` ⇒ scoped search. Don't invent a separate command for the two cases.
- **Sort order in the snippet list** (§9.4) — `favorite desc, last_used_at desc, used_count desc, updated_at desc`. This is the user-visible recency model; preserve it.
- **Keyboard shortcuts** (§11) — including the global toggle `Ctrl+Space`, `Ctrl+Shift+N` for new category, and `Ctrl+↑/↓` for category switching. These are part of the product, not implementation detail.

## Build / test / run

Not yet defined — there is no `package.json` or `Cargo.toml`. When the project is scaffolded, replace this section with the actual commands. Until then, don't invent `npm run dev` or `cargo tauri dev` invocations that haven't been wired up.