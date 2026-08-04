# CLAUDE.md

Guidance for Claude Code when working in this repository.

## What this is

A local-first Tauri desktop app. It stores a student's full experience history in Postgres,
tailors an ATS-optimized LaTeX resume to a specific job posting using a locally-run model,
and tracks which resume went to which company.

Read `IMPLEMENTATION_PLAN.md` before making structural changes. It holds the schema,
pipeline design, and milestone order.

## Hard rules

These are not preferences. Violating one is a bug regardless of test status.

1. **No resume line may exist without a `bullets` row behind it.** The renderer accepts
   bullet IDs, not free text. If you find yourself passing a model-authored string into the
   Tera context without a `bullet_id`, stop.
2. **Every reworded bullet passes `pipeline/grounding.rs` before rendering.** New numbers,
   new named entities, or length outside 0.7×–1.25× of the source mean rejection and
   fallback to verbatim text. Never widen these thresholds to make a demo look better.
3. **No user data leaves the machine.** Postgres and Ollama are local. The only outbound
   request is fetching the job posting URL the user typed. No analytics, no cloud LLM, no
   third-party extraction API.
4. **LaTeX escaping happens in exactly one place:** `render/tex.rs`, in the context builder.
   Never in a template, never at a call site.
5. **The model outputs plain text.** Any of `\ { } $ ^ _ ~ #` in model output is a rejection
   signal, not something to sanitize and pass through.
6. **Prefer removing a dependency to adding one.** This app is intentionally small. If a
   library is being added, the plan should say why the standard-library or hand-rolled
   version is insufficient.

## Stack

- **Shell:** Tauri v2 (Rust)
- **Frontend:** React 18 + TypeScript + Vite, plain CSS with custom properties
- **Database:** local Postgres via `sqlx` (runtime queries, not macros — no `DATABASE_URL`
  needed at compile time)
- **Inference:** Ollama at `localhost:11434`, model `ornith:35b`
- **PDF:** Tectonic invoked as an external binary
- **Templating:** Tera, over the user's Jake's-Resume `.tex`

State is React Context + `useReducer`. Charts are hand-rolled SVG. There is no Redux, no
react-query, no Tailwind, no charting library, and adding one needs a reason in the PR.

## Commands

```bash
# Dev
npm install
npm run tauri dev

# Database (Postgres must be running locally)
createdb resume_fixer
cargo sqlx migrate run --source migrations

# Rust
cd src-tauri
cargo test                       # unit + integration; needs a scratch DB
cargo test grounding             # the suite that matters most
cargo clippy -- -D warnings
cargo fmt

# Frontend
npm run lint
npm run build

# Release
npm run tauri build
```

Model calls are mocked in tests behind the `LlmClient` trait. No test requires Ollama.

## Layout

```
migrations/            sqlx migrations, NNNN_name.sql
templates/
  resume.tex.tera      Jake's Resume, the default generation template
  simplify.tex.tera    the Simplify layout — the one built-in with an Activities section
  reference/           the user's current resume — layout source of truth, do not edit
src/                   React frontend
  ipc.ts               the only place invoke() is called
  types.ts             mirrors src-tauri/src/domain.rs
  tabs/{generate,library,vault,base}/
src-tauri/src/
  commands/            thin IPC handlers — no business logic here
  db/                  repositories, one per aggregate; returns domain structs
  ingest/              URL fetch + HTML extraction
  llm/                 Ollama client, prompts, response schemas
  pipeline/            retrieval, select, grounding, fit
  render/              LaTeX escaping, Tera, Tectonic
  domain.rs            shared structs (serde + FromRow)
  error.rs             AppError, serialized to the frontend as { kind, message, detail }
```

Layering: `commands` → `services` → `db`/`llm`/`render`. Commands do not query. Services do
not touch `sqlx` types. Repositories do not call the model.

## Code conventions

- Modules stay small and single-purpose. If a file passes ~300 lines, it is doing two jobs.
- Every public Rust function has a doc comment stating what it does and what it rejects.
  Grounding checks and scoring formulas get a comment explaining *why* the threshold is what
  it is — a future reader must not tune it blind.
- Errors are typed (`thiserror`), never `anyhow` at the IPC boundary, never `unwrap()`
  outside tests.
- Frontend types in `src/types.ts` mirror `domain.rs` exactly. Change one, change both in
  the same commit.
- SQL lives in repository functions, not in services or commands.
- Prompts live in `llm/prompts/` as separate `.txt` files with a version constant, never
  inline in Rust. Changing a prompt means bumping `PROMPT_VERSION` so `resumes` rows stay
  attributable.
- Comments describe present behavior. No "previously", "now uses", "new approach", or
  changelog narration in code or docs — that is what git is for.

## Domain notes worth knowing

- **One experience can have several roles.** Rajant Health has both a Computer Engineering
  Intern and a Software Engineering Intern stint. The template renders the first with
  `\resumeSubheading` and the rest with `\resumeSubSubheading`.
- **Skills are tagged on both bullets and experiences.** Bullet tags drive precision,
  experience tags catch relevance the bullet text does not spell out. Both feed retrieval.
- **Skill matching is by `slug` plus `aliases[]`**, always case-folded. Never match on
  `skills.name`.
- **Resumes must fit one page.** The fit loop drops the lowest-scoring bullet from the
  largest section and recompiles, up to 6 times, never emptying an experience.
- **Page count comes from the TeX log** line `Output written on ... (N pages, ...)`. Do not
  add a PDF parsing crate for this.
- **Tectonic downloads packages on first compile.** Surface that in the UI; it is not a hang.
- **Drafts are in-memory** until the user commits them to the library. Discarding writes
  nothing to the database. A base resume preview follows the same rule.
- **Templates live in the `templates` table.** Built-ins are re-synced from
  `templates/*.tex.tera` on every health check, so the files stay authoritative and a
  built-in cannot be edited in place — only saved under a new name. One row is `is_active`
  and that is what Generate renders with.
- **A template prints a section only if it names the variable.** `ResumePlan::prune_for`
  drops the sections the source never mentions, so a bullet the template could not print is
  never recorded as used. That is how `activity` entries reach the Simplify layout and stay
  out of Jake's.
- **The base resume runs no model and no fit loop.** Every active bullet, verbatim, through
  the chosen template. Dropping bullets to reach one page would be the wrong document.

## Bullet quality standard

Bullets follow: **action verb → what was built → technology → measurable outcome.**

Weak: "Worked on backend services for banking applications."
Strong: "Designed Java-based RESTful services supporting high-volume banking transactions,
improving request latency by 25%."

The strong version adds no facts the weak one lacked — it surfaces them. That distinction is
the entire license the model has to reword. Anything beyond it is fabrication.

## When touching the pipeline

Changes to `pipeline/`, `llm/prompts/`, or `render/tex.rs` require running
`cargo test grounding` and `cargo test escaping` before the change is considered done. Both
suites exist because their failure modes are silent: a hallucinated bullet and a broken
`&` look fine until a recruiter reads them.
