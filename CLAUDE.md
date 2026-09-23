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
6. **No personal data in `migrations/`.** `sqlx::migrate!` compiles the directory into the
   binary, so anything there ships in the `.dmg` and seeds itself into a stranger's database.
   Schema goes in `migrations/`, the maintainer's own vault goes in `seed/`, applied by hand.
7. **Prefer removing a dependency to adding one.** This app is intentionally small. If a
   library is being added, the plan should say why the standard-library or hand-rolled
   version is insufficient.

## Stack

- **Shell:** Tauri v2 (Rust)
- **Frontend:** React 18 + TypeScript + Vite, plain CSS with custom properties. Caprasimo and
  Figtree ship as `@fontsource` packages rather than a Google Fonts `@import`: the app has to
  look right with the network off. Icons are inline SVG in `src/icons.tsx`, not `lucide-react`.
- **Database:** local Postgres via `sqlx` (runtime queries, not macros — no `DATABASE_URL`
  needed at compile time)
- **Inference:** Ollama at `localhost:11434`, model `ornith:35b`
- **PDF:** Tectonic invoked as an external binary
- **Templating:** Tera, over the user's Jake's-Resume `.tex`

State is React Context + `useReducer`. The Library funnel is hand-rolled — four divs and a
width percentage. There is no Redux, no react-query, no Tailwind, no charting library, and
adding one needs a reason in the PR.

## Commands

```bash
# Dev
npm install
npm run tauri dev

# Database (Postgres must be running locally)
createdb resume_fixer
# `db::pool::migrate` runs migrations/ at startup — there is no separate migrate step.
# The vault opens empty; seed it if you want real data to work against:
psql resume_fixer -f seed/0002_seed.sql   # then 0007, 0008, 0010, 0016 in order

# Rust
cd src-tauri
cargo test                       # unit; needs no database, model, or LaTeX
cargo test grounding             # the suite that matters most

# The suites that need the real stack
createdb resume_fixer_test
export TEST_DATABASE_URL=postgresql://localhost/resume_fixer_test
cargo test --test db             # skips itself, rather than failing, without that variable
cargo test --test render_reference -- --ignored     # needs Tectonic
cargo test --test generate_end_to_end -- --ignored  # needs all three
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
migrations/            sqlx migrations, NNNN_name.sql — schema only, run at startup
seed/                  the user's own vault content, same numbering, applied by hand with psql
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
- **A role's calendar dates are optional; `date_override` is what prints when set.**
  `render::tex::format_dates` returns the override whenever it is non-blank, the computed
  range when both dates are there, and an empty string when a role has neither — which is
  how an expected graduation prints as "Spring 2029" with no dates behind it. Roles with no
  start date sort after the dated ones.
- **A project prints a clickable link where its dates would go.** `experiences.url` set
  means the Projects heading emits `\href{url}{\underline{label}}` in the right-hand slot
  and the date range stays in the vault unprinted — one narrow slot, one occupant. The
  label is `experiences.link_text`, falling back to the URL without its scheme. A URL with
  no scheme gets `https://` so the PDF link is not dead. Only the Projects section does
  this; everywhere else the slot is dates.
- **Checked skills print in the order they were checked.** `skills.listed_at` is stamped
  when the box is ticked and cleared when it is unticked, and `db::skill::list` orders by
  it — so unticking and re-ticking is how a skill is moved to the end of the line.
- **An application can be tracked without generating anything.** `services::library::track`
  writes the row and stores an uploaded PDF as that application's current resume, with
  `model = "uploaded"`, no TeX source, and no provenance rows — nothing was selected from
  the vault, so there is nothing to attribute.
- **Skills are tagged on both bullets and experiences.** Bullet tags drive precision,
  experience tags catch relevance the bullet text does not spell out. Both feed retrieval.
- **Skill matching is by `slug` plus `aliases[]`**, always case-folded. Never match on
  `skills.name`.
- **Resumes must fit one page, and must fill it.** The fit loop shrinks in two moves, in
  order: retire the lowest-scoring experience whole, and only when nothing may be retired,
  drop the lowest-scoring bullet from the largest section. Depth beats breadth — three
  entries with three lines each carry more than eight with one. Never emptying an
  experience, never retiring education, a pinned entry, or the last one standing. Once it
  reaches one page it runs the same two moves backwards, breadth first: un-retire the
  strongest experience, then restore the strongest benched bullet, reverting the move that
  spills to a second page. What the cap and the shrink take goes on a bench inside
  `ResumePlan`, which is what the grow pass spends.
- **An entry that earns the page earns its lines.** `select::deepen` fills every experience
  the model chose with the rest of its active bullets, verbatim, before the cap runs. The
  model picks one line at a time and will return a single bullet from each of eight
  experiences; nothing downstream could repair that, because `cap_bullets` only removes and
  the grow pass spends a bench a thin selection never filled. An experience the model passed
  over stays off entirely.
- **Experiences are ranked two ways.** A tailored resume ranks by fit for the posting (the
  score of its strongest bullet from `retrieval`, not the average — the lines `deepen` adds
  score lower, and depth must not cost an entry its place); the base resume has no posting,
  so it ranks by intrinsic merit from `pipeline/strength.rs` — quantified outcome, then recency, then skill density,
  in that order and for the reasons written there. Retired entries are named in the UI: the
  user is never silently edited.
- **Page count comes from the TeX log** line `Output written on ... (N pages, ...)`. Do not
  add a PDF parsing crate for this.
- **Tectonic downloads packages on first compile.** Surface that in the UI; it is not a hang.
- **The app creates its own database.** `db::pool::is_ready` reaches for
  `Postgres::create_database` once `SELECT 1` has failed, so a fresh install needs a running
  server and nothing else — no `createdb`, no schema to import. `migrate` sets
  `ignore_missing` because versions 2, 7, 8 and 10 seeded the maintainer's vault and now
  live in `seed/`; a database written before that move records them with no file behind them.
- **Drafts are in-memory** until the user commits them to the library. Discarding writes
  nothing to the database. A base resume preview follows the same rule.
- **Hand edits to a draft are draft-local.** `ResumePlan::apply_edits` changes `text` and
  leaves `source_text` alone, so the vault keeps its wording and `was_reworded` goes false —
  the model is never credited with a line the user wrote.
- **Templates live in the `templates` table.** Built-ins are re-synced from
  `templates/*.tex.tera` on every health check, so the files stay authoritative and a
  built-in cannot be edited in place — only saved under a new name. One row is `is_active`
  and that is what Generate renders with.
- **A template prints a section only if it names the variable.** `ResumePlan::prune_for`
  drops the sections the source never mentions, so a bullet the template could not print is
  never recorded as used. That is how `activity` entries reach the Simplify layout and stay
  out of Jake's.
- **The base resume runs no model.** Every line it prints is a stored bullet verbatim, so
  grounding has nothing to check. It does run the fit loop, ranked by intrinsic merit rather
  than fit for a posting, and obeys the same three-bullets-per-experience cap. Both
  documents can be trimmed by hand before saving.
- **At most three bullets print under one experience — two under a project**
  (`ResumePlan::cap_bullets`). A project takes a third only when it scores within 0.8 of
  that project's own best line, because three projects with strong bullets say more than two
  padded to three. The cut is by score; ordering stays the vault's. Roles the cap empties
  are dropped, except under education and activities, where the heading is the content.
  Capping lean is deliberate — the grow pass restores from the bench when the page has room.
- **A user's edit may add a bullet, not only remove one.** `apply_edits` takes the vault, so
  a `BulletEdit` with `keep: true` naming a line the plan is not printing fetches it and
  prints it verbatim. Limited to experiences already on the resume: an entry the fit loop
  left off comes back whole or not at all. `revise` runs no cap and no fit loop, so what the
  user adds by hand stays.

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
