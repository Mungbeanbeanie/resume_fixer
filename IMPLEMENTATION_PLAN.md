# Resume Fixer — Implementation Plan

A local-first desktop app that stores every piece of a student's experience in Postgres,
tailors an ATS-optimized LaTeX resume to a specific job posting using a locally-run model,
and tracks which resume went to which company.

Everything runs on the user's machine. No experience data, resume text, or job description
is ever sent to a third-party service.

---

## 1. Locked decisions

| Area | Decision | Rationale |
|---|---|---|
| Shell | Tauri v2 | Small binary, native feel, Rust backend owns DB/LLM/LaTeX |
| Frontend | React 18 + TypeScript + Vite | Standard, no framework beyond what Tauri scaffolds |
| Styling | Plain CSS with CSS custom properties | Three simple screens; a utility framework is not warranted |
| State | React Context + `useReducer`, one context per tab | No Redux, no react-query |
| Charts | Hand-rolled SVG | One stacked bar + three counters. A charting library is ~200KB for a rectangle |
| Database | Local Postgres, connection string in config | Personal tool; user manages the server |
| DB access | `sqlx` with **runtime** queries (`query_as::<_, T>`) | Avoids `DATABASE_URL` being required at compile time |
| Migrations | `sqlx::migrate!` against `migrations/` | Built in, no extra tooling |
| Inference | Ollama HTTP API at `localhost:11434`, model `ornith:35b` | Already installed; configurable |
| PDF | Tectonic binary | Self-contained, no TeX Live install |
| Template | Jake's Resume (user's existing `.tex`), rendered via Tera | Known-good ATS output, user already uses it |
| Job ingest | Local HTTP fetch + HTML extraction, manual paste fallback | Fully offline, no API key |
| Bullet policy | Select + light rewording behind a grounding gate | Tailored output, hallucination blocked by verification |
| Statuses | Saved → Applied → Rejected / Interview / Offer | Graph rolls up to applied / rejected / waiting |
| Scope | Three tabs: Generate, Library, Vault | Nothing else in v1 |

**Non-goals for v1:** cover letters, multi-user accounts, cloud sync, resume diffing,
multiple templates, auth, telemetry.

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────┐
│  React + TS (webview)                                   │
│  Generate  │  Library  │  Vault                         │
└───────────────────────┬─────────────────────────────────┘
                        │  Tauri IPC (typed commands)
┌───────────────────────┴─────────────────────────────────┐
│  Rust core                                              │
│                                                         │
│  ingest    → fetch URL, extract job text                │
│  retrieval → deterministic SQL keyword scoring          │
│  llm       → Ollama client (extract / select / reword)  │
│  grounding → verifies reworded text against source      │
│  render    → Tera → .tex → Tectonic → .pdf              │
│  db        → sqlx repositories                          │
└───────┬──────────────────────┬──────────────────────────┘
        │                      │
   ┌────┴─────┐         ┌──────┴──────┐      ┌──────────┐
   │ Postgres │         │   Ollama    │      │ Tectonic │
   │  :5432   │         │   :11434    │      │  binary  │
   └──────────┘         └─────────────┘      └──────────┘
```

**Layering rule:** `commands` (IPC boundary) → `services` (orchestration) → `db` / `llm` /
`render` (single-responsibility modules). Services never touch `sqlx` types directly;
repositories return domain structs.

### Directory layout

```
resume_fixer/
├── CLAUDE.md
├── IMPLEMENTATION_PLAN.md
├── migrations/                  # sqlx migrations, NNNN_name.sql
├── templates/
│   ├── resume.tex.tera          # the generation template
│   └── reference/base_resume.tex  # user's current resume, source of truth for layout
├── src/                         # React frontend
│   ├── main.tsx
│   ├── App.tsx
│   ├── ipc.ts                   # single typed wrapper over invoke()
│   ├── types.ts                 # mirrors Rust domain structs
│   ├── styles/tokens.css
│   ├── components/              # Button, Field, Tabs, StatusPill, StackedBar...
│   └── tabs/
│       ├── generate/
│       ├── library/
│       └── vault/
└── src-tauri/
    ├── tauri.conf.json
    └── src/
        ├── main.rs
        ├── config.rs            # connection string, model name, paths
        ├── error.rs             # AppError + IPC serialization
        ├── domain.rs            # shared structs, serde + FromRow
        ├── commands/            # thin IPC handlers, one file per tab
        ├── db/                  # pool.rs + one repository per aggregate
        ├── ingest/              # fetch.rs, extract.rs
        ├── llm/                 # client.rs, prompts/, schemas.rs
        ├── pipeline/            # retrieval.rs, select.rs, grounding.rs, fit.rs
        └── render/              # tex.rs (escaping + Tera), tectonic.rs
```

### Rust dependencies (deliberately short)

`tauri`, `tokio`, `serde`, `serde_json`, `sqlx` (postgres, chrono, uuid, runtime-tokio),
`reqwest` (json), `scraper`, `tera`, `chrono`, `uuid`, `thiserror`, `tracing`.

Tectonic is invoked as an external binary, not linked as a crate — it keeps build times
sane and lets the user swap in their own TeX engine.

---

## 3. Data model

Two clusters: the **Vault** (what the student has done) and the **Library** (where they
applied and with what). They join only through `resume_bullets`, which is the provenance
record.

```sql
-- 0001_init.sql

CREATE TYPE experience_kind AS ENUM ('work', 'project', 'education', 'certification');
CREATE TYPE application_status AS ENUM
    ('saved', 'applied', 'rejected', 'interview', 'offer', 'withdrawn');
CREATE TYPE job_source AS ENUM ('fetched', 'pasted');

-- ── Profile (single row) ────────────────────────────────────────────────
CREATE TABLE profile (
    id          BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),  -- enforces one row
    full_name   TEXT NOT NULL,
    phone       TEXT,
    email       TEXT,
    links       JSONB NOT NULL DEFAULT '[]',  -- [{label, url}]
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Vault ───────────────────────────────────────────────────────────────
CREATE TABLE experiences (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kind          experience_kind NOT NULL,
    org_name      TEXT NOT NULL,          -- "Rajant Health" / "Sniped" / "UVA"
    location      TEXT,                   -- "Malvern, PA" / "Remote"
    url           TEXT,
    tech_line     TEXT,                   -- projects only: "PostgreSQL, Node.js, Docker"
    display_order INT NOT NULL DEFAULT 0,
    is_active     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- One experience can hold several roles (e.g. intern → returning intern at one company).
CREATE TABLE roles (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    experience_id  UUID NOT NULL REFERENCES experiences(id) ON DELETE CASCADE,
    title          TEXT NOT NULL,         -- "Software Engineering Intern"
    location       TEXT,                  -- overrides experience.location when set
    start_date     DATE NOT NULL,
    end_date       DATE,                  -- NULL = present
    date_override  TEXT,                  -- rare formatting escape hatch, e.g. "Summer 2025"
    display_order  INT NOT NULL DEFAULT 0,
    is_active      BOOLEAN NOT NULL DEFAULT TRUE
);
CREATE INDEX ON roles (experience_id);

CREATE TABLE bullets (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id        UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    text           TEXT NOT NULL,         -- plain text, NO LaTeX markup
    display_order  INT NOT NULL DEFAULT 0,
    is_active      BOOLEAN NOT NULL DEFAULT TRUE,
    char_count     INT GENERATED ALWAYS AS (length(text)) STORED,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ON bullets (role_id);

-- Alternate phrasings the user has approved. The resume may use any of these verbatim.
CREATE TABLE bullet_variants (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bullet_id   UUID NOT NULL REFERENCES bullets(id) ON DELETE CASCADE,
    text        TEXT NOT NULL,
    origin      TEXT NOT NULL,            -- 'ai_suggested' | 'manual'
    approved_at TIMESTAMPTZ,              -- NULL = suggested but not accepted
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ON bullet_variants (bullet_id);

-- ── Skills ──────────────────────────────────────────────────────────────
CREATE TABLE skills (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL,             -- display form: "PostgreSQL"
    slug       TEXT NOT NULL UNIQUE,      -- match key: "postgresql"
    category   TEXT,                      -- "language" | "cloud" | "concept" | ...
    aliases    TEXT[] NOT NULL DEFAULT '{}'  -- ["postgres", "psql", "pg"]
);
CREATE INDEX ON skills USING GIN (aliases);

CREATE TABLE bullet_skills (
    bullet_id UUID NOT NULL REFERENCES bullets(id) ON DELETE CASCADE,
    skill_id  UUID NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    weight    REAL NOT NULL DEFAULT 1.0,  -- 1.0 = central, 0.5 = incidental
    PRIMARY KEY (bullet_id, skill_id)
);

CREATE TABLE experience_skills (
    experience_id UUID NOT NULL REFERENCES experiences(id) ON DELETE CASCADE,
    skill_id      UUID NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    PRIMARY KEY (experience_id, skill_id)
);

-- ── Library ─────────────────────────────────────────────────────────────
CREATE TABLE applications (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url           TEXT,
    company       TEXT,
    role_title    TEXT,
    job_text      TEXT NOT NULL,          -- extracted or pasted description
    job_source    job_source NOT NULL,
    parsed        JSONB NOT NULL DEFAULT '{}',  -- model's structured read of the posting
    status        application_status NOT NULL DEFAULT 'saved',
    applied_at    TIMESTAMPTZ,
    notes         TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ON applications (status);
CREATE INDEX ON applications (created_at DESC);

CREATE TABLE application_status_history (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    status         application_status NOT NULL,
    changed_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ON application_status_history (application_id);

CREATE TABLE resumes (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    tex_source     TEXT NOT NULL,
    pdf_path       TEXT,                  -- absolute path in the app data dir
    model          TEXT NOT NULL,
    prompt_version TEXT NOT NULL,
    feedback       TEXT,                  -- user's regeneration instruction, if any
    page_count     INT,
    is_current     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ON resumes (application_id);

-- Provenance: exactly which stored bullet produced which printed line.
CREATE TABLE resume_bullets (
    resume_id      UUID NOT NULL REFERENCES resumes(id) ON DELETE CASCADE,
    bullet_id      UUID NOT NULL REFERENCES bullets(id) ON DELETE RESTRICT,
    rendered_text  TEXT NOT NULL,
    was_reworded   BOOLEAN NOT NULL DEFAULT FALSE,
    display_order  INT NOT NULL,
    PRIMARY KEY (resume_id, bullet_id)
);
```

**Why `resume_bullets` matters:** every printed line traces to a row in `bullets` via a
foreign key. If a line cannot be attributed, it cannot be rendered. That is the structural
half of the anti-hallucination guarantee; section 4.5 is the textual half.

### Beyond 0001

The block above is `0001_init.sql`. `migrations/` is the schema of record — the summary
here exists so a reader knows what to expect, not so it can be read instead:

| Migration | Change |
|---|---|
| `0003_activity_kind` | `experience_kind` gains `activity` — clubs and orgs, which only the Simplify layout prints |
| `0004_templates` | `templates` (name, Tera source, `is_builtin`, `is_active`) and `base_resumes` (a saved posting-less resume, with the template it was built from) |
| `0005` | `skills.always_list` — a skill that prints in the skills section whether or not the posting asks for it |
| `0006_experience_pin` | `experiences.is_pinned` — an entry the fit loop may not retire |
| `0011_profile_interests` | `profile.interests`, one comma-separated line, Simplify only |
| `0012_role_gpa` | `roles.gpa`, free text: it belongs to the degree, not the school |
| `0013_assessment_and_rounds` | `application_status` rebuilt as `saved, applied, oa_received, oa_completed, interview_1..3, offer, rejected, withdrawn`; existing `interview` rows land on round one |

`migrations/` carries schema only, so a fresh database opens to an empty vault. One
person's experience history is content, not structure, and a migration that inserts it
cannot be skipped by anyone else. That content lives in `seed/`, numbered against the
migration it once sat between, and is applied by hand:

```bash
psql resume_fixer -f seed/0002_seed.sql   # and 0007, 0008, 0010 in order
```

`0009_skill_list_flag` stays in `migrations/` and is a no-op on an empty database — the
seed sets `always_list` itself, since by the time it runs the schema is already current.

---

## 4. Generation pipeline

Input: a job URL (or pasted text). Output: a compiled one-page PDF plus a provenance record.

Six stages. Stages 1, 3, 5 and 6 are deterministic; only 2 and 4 involve the model.

### 4.1 Ingest — `ingest/`

1. `reqwest` GET with a browser User-Agent, 10s timeout, redirects followed.
2. Parse with `scraper`. Strip `script`, `style`, `nav`, `header`, `footer`, `svg`.
3. Score candidate containers by text density (text length ÷ tag count), take the best.
4. Normalize whitespace; collapse to plain text.
5. **Quality gate:** if extracted text < 400 chars, or lacks any of the tokens
   `responsibilit`, `qualificat`, `requirement`, `experience`, `skills` — treat as failed.
6. On failure the UI shows a paste box. No retry loop, no headless browser, no third-party
   extraction API.

Known-good: Lever, Ashby, most company career pages, LinkedIn public postings.
Known to need paste: Workday, Greenhouse embeds, anything behind a login.

### 4.2 Parse the posting — `llm/` (model call #1)

Prompt the model with the job text, request JSON via Ollama's `format: "json"`:

```jsonc
{
  "company": "string|null",
  "title": "string|null",
  "hard_skills": ["Python", "PostgreSQL"],   // explicit technologies
  "soft_signals": ["cross-functional", "ownership"],
  "domain": "string|null",                   // "fintech", "medical devices"
  "seniority": "intern|junior|mid|unknown"
}
```

Validate against a serde struct. If parsing fails twice, fall back to a deterministic
keyword scan against the `skills` table and continue — a failed model call degrades
quality, it does not break the flow.

### 4.3 Retrieve candidates — `pipeline/retrieval.rs` (no model)

Pure SQL + Rust scoring. For each active bullet:

```
score = 2.0 * Σ(weight of bullet_skills matching a hard_skill)
      + 1.0 * Σ(weight of experience_skills matching a hard_skill)
      + 0.5 * (literal token overlap between bullet.text and hard_skills)
      + 0.3 * recency_factor(role.end_date)   // 1.0 current, decaying to 0 at 4 years
```

Skill matching is by `slug` and `aliases`, case-folded. Take the top ~24 bullets, but
always retain at least one bullet per active experience so nothing silently disappears.

This stage is the reason skills are tagged per bullet *and* per experience: the bullet tags
drive precision, the experience tags catch relevance the bullet text does not spell out.

### 4.4 Select and reword — `pipeline/select.rs` (model call #2)

The model receives the candidate bullets **as an ID-keyed list** and the parsed job. It
returns only IDs and optional rewrites:

```jsonc
{
  "sections": [
    { "experience_id": "uuid",
      "bullets": [
        { "id": "uuid", "rewrite": null },
        { "id": "uuid", "rewrite": "Designed Java-based RESTful services ..." }
      ]}
  ],
  "skills_line": ["Python", "Java", "SQL"],   // must be a subset of the user's skills
  "reasoning": "one sentence"
}
```

Prompt rules, stated explicitly and repeated at the end of the prompt:

- You may only reference IDs from the provided list. Inventing an ID is a failure.
- A rewrite may **reorder, compress, or re-emphasize** the source bullet. It may not
  introduce any fact — no new number, technology, employer, scope, or outcome.
- If a bullet's fit is unclear, return it with `"rewrite": null`.
- Bullets follow the strong pattern: **action verb → what was built → technology →
  measurable outcome**. Weak: "Worked on backend services for banking applications."
  Strong: "Designed Java-based RESTful services supporting high-volume banking
  transactions, improving request latency by 25%." Note that the strong version adds no
  facts the weak one lacked access to — it surfaces them.
- Output plain text only. No LaTeX, no markdown, no backslashes.

Any ID not in the candidate set is dropped silently and logged.

### 4.5 Ground the rewrites — `pipeline/grounding.rs`

Every `rewrite` passes a deterministic checker before it is allowed anywhere near the
renderer. A rewrite that fails **falls back to the verbatim source bullet** — generation is
never aborted for this reason.

| Check | Rule |
|---|---|
| Numbers | Every numeric token in the rewrite (`25%`, `460 kbps`, `20+`, `2,000`) must appear in the source, normalized for separators and units. New numbers → reject. |
| Named entities | Every capitalized multi-char token and every known skill slug in the rewrite must appear in the source or in that bullet's `bullet_skills`. New entities → reject. |
| Length | 0.7× ≤ rewrite length ≤ 1.25× source length. Outside → reject. |
| Superlatives | Blocklist: "first ever", "industry-leading", "award-winning", "best-in-class", "single-handedly", "revolutionary" — unless present in the source. |
| Markup | Any of `\ { } $ ^ _ ~ #` in the rewrite → reject (the model was told plain text). |
| Skills line | Every entry in `skills_line` must resolve to a row in `skills` by slug or alias. Unresolvable entries are dropped, not rendered. |

Accepted rewrites are written to `resume_bullets.rendered_text` with
`was_reworded = true`, and additionally inserted into `bullet_variants` with
`origin = 'ai_suggested'`, `approved_at = NULL` — so a phrasing you liked is one click from
becoming permanent in the Vault.

**Rejections are surfaced, not hidden.** The Generate tab shows a quiet line:
"3 rewrites rejected, original wording used." Clicking it lists them with the reason.

### 4.6 Render and fit — `render/`

1. Build the Tera context from selected bullets (section 5).
2. **Escape every user string** for LaTeX in `render/tex.rs` — this is the only place
   escaping happens: `\ & % $ # _ { } ~ ^`. Escaping is applied in the Rust context
   builder, not in the template, so a template edit cannot introduce an injection.
3. Render `templates/resume.tex.tera`.
4. Run `tectonic -X compile --outdir <tmp> resume.tex`.
5. Read page count from the TeX log line `Output written on resume.pdf (N pages, ...)`.
6. If N > 1: drop the lowest-scoring bullet from the largest section and re-render. Max 6
   iterations, never dropping an experience's last bullet. If still > 1 page, keep the
   result and warn the user.
7. Store `.tex` in `resumes.tex_source`, PDF in the app data dir, path in `pdf_path`.

Tectonic caches packages in `~/.cache/Tectonic`. First compile downloads; document this so
the first run's delay is not mistaken for a hang.

---

## 5. Template contract

`templates/reference/base_resume.tex` is the user's current resume and defines the target
output. `templates/resume.tex.tera` is that file with the content regions replaced by Tera
loops. The preamble, custom commands, and spacing are copied verbatim and must not be
regenerated by the model.

Commands consumed by the template:

| Command | Arguments | Used for |
|---|---|---|
| `\resumeSubheading` | org, location, title, dates | First role of any work/education experience |
| `\resumeSubSubheading` | title, dates | Additional roles at the same org |
| `\resumeProjectHeading` | `\textbf{name} $\|$ \emph{tech}`, dates | Projects |
| `\resumeItem` | text | A bullet |
| `\resumeItemListStart/End` | — | Wraps bullets |
| `\resumeSubHeadingListStart/End` | — | Wraps a section |

Tera context shape:

```jsonc
{
  "profile": { "full_name": "...", "phone": "...", "email": "...",
               "links": [{ "label": "...", "url": "..." }] },
  "education": [{ "org": "...", "location": "...", "title": "...",
                  "dates": "Aug. 2025 -- May 2029", "bullets": ["..."] }],
  "skills_line": "Python, Java, SQL, ...",
  "experience": [{ "org": "...", "location": "...",
                   "roles": [{ "title": "...", "dates": "...", "bullets": ["..."] }] }],
  "projects": [{ "name": "...", "tech": "...", "dates": "...", "bullets": ["..."] }],
  "certifications": ["CompTIA Security+", "..."]
}
```

Date formatting is done in Rust (`Mon. YYYY`, `--` separator, `Present` for open-ended,
`date_override` wins when set), not in the template.

A section with zero entries emits nothing — no empty `\section` headers.

### Edge cases the reference file establishes

- **Certifications have no roles and no bullets.** They are `experiences` rows with
  `kind = 'certification'` and zero child rows; the template joins their `org_name` values
  with `$|$` on one line. Do not create placeholder roles for them.
- **Education carries the skills line.** In the reference it is a `\resumeItem` containing
  `\textbf{Skills:}` inside the education block. The Tera context keeps `skills_line`
  separate from `education[].bullets` because its contents are job-tailored while coursework
  is not.
- **One org, several roles.** Rajant Health renders as `\resumeSubheading` for the first
  role and `\resumeSubSubheading` for each subsequent one. Roles order by `start_date`
  descending, so the most recent title is the one that gets the org header.
- **The reference defines both `\resumeSubheadingListEnd` and `\resumeSubHeadingListEnd`**
  (differing only in the capital H) and uses both. Copy the preamble verbatim rather than
  tidying it — the generated file must compile identically to the one the user already
  submits.
- **`withdrawn` applications** are excluded from the graph alongside `saved`, and shown in
  the table with a muted pill.

---

## 6. IPC surface

One command per user intent. All return `Result<T, AppError>`; `AppError` serializes to
`{ kind, message, detail? }` so the frontend can branch on `kind` without string matching.

```
// Vault
vault_list_experiences() -> Vec<ExperienceDetail>
vault_upsert_experience(input) -> ExperienceDetail
vault_delete_experience(id)
vault_upsert_role(input) -> Role
vault_delete_role(id)
vault_upsert_bullet(input) -> Bullet
vault_delete_bullet(id)
vault_set_bullet_skills(bullet_id, skill_names: Vec<String>)   // upserts skills by slug
vault_suggest_bullet_improvements(bullet_id) -> Vec<Suggestion> // model, grounded
vault_accept_variant(variant_id)                                // promotes to bullets.text
vault_list_skills() -> Vec<Skill>
vault_add_skill(name) -> Skill
vault_set_skill_listed(skill_id, always_list)   // pins a skill into the skills section
vault_get_profile() -> Option<Profile>
vault_upsert_profile(profile) -> Profile

// Generate
generate_ingest_job(url) -> IngestResult          // { text, source, needs_paste, reason }
generate_from_text(job_text, url?, feedback?) -> GenerationResult
generate_revise(draft_id, edits) -> GenerationResult  // hand edits, no cap and no fit loop
generate_discard(draft_id)
generate_commit(draft_id, applied: bool) -> Uuid  // writes application + resume
generate_export_draft(draft_id, filename) -> String   // uncommitted draft to output_dir
generate_export_pdf(resume_id, dest_path)

// Base — the vault through one template, no posting and no model
base_list_templates() -> Vec<Template>
base_save_template(name, source) -> Template   // a built-in saves as a copy, never in place
base_set_active_template(id)                   // the one Generate renders with
base_delete_template(id)
base_render(template_id?) -> BasePreview
base_revise(edits) -> BasePreview
base_save(name) -> BaseResume
base_list() -> Vec<BaseResume>
base_delete(id)
base_export(id, filename) -> String

// Library
library_list_applications(filter?) -> Vec<ApplicationSummary>
library_get_application(id) -> ApplicationDetail
library_set_status(id, status)
library_update_application(id, patch)
library_delete_application(id)
library_stats() -> StatusStats                    // counts for the graph

// Health
health_check() -> HealthReport  // { postgres, ollama, model_present, tectonic }
```

`GenerationResult` carries the draft id, PDF path, per-bullet provenance, rejected-rewrite
list, and page count — everything the Generate tab needs without a second round trip.

Drafts live in memory (a `Mutex<HashMap<Uuid, Draft>>` in Tauri state) until committed.
Discarding one costs nothing and writes nothing.

---

## 7. UI

Design tone: light, minimal, generous whitespace. One accent color. System font stack
(`-apple-system, ui-sans-serif`). No shadows beyond a 1px border, no gradients, no icons
except a small set of inline SVGs. Roughly: `--bg #FCFCFA`, `--surface #FFFFFF`,
`--border #E8E6E1`, `--text #1A1A18`, `--muted #6B6B66`, `--accent #2E6F5E`.

### Generate (default view)

Opens to near-empty canvas: the app name, one input, one button.

```
                        Paste a job link

     ┌──────────────────────────────────────────────┐
     │  https://...                                 │  [ Generate ]
     └──────────────────────────────────────────────┘
                  or paste the description
```

States: `idle → ingesting → parsing → selecting → rendering → ready`. A single line of
status text, no spinner carousel, no progress bar theater.

`ready` shows the PDF preview (native `<embed>`, no PDF.js) with four actions:
**Download**, **Regenerate** (opens a one-line feedback field), **Save to library**
(prompts "Did you apply?" → sets status `applied` or `saved`), **Discard**.

Below the preview, collapsed: which bullets were used, and the rejected-rewrite notice.

### Library

Table: Company · Role · Status pill · Date · Resume link · Job link. Row click opens a
side panel with the job text, the resume, and a status dropdown.

Above the table, the graph: a single horizontal stacked bar (Awaiting / OA / Interview /
Offer / Rejected) with counts, plus a response-rate figure. Hand-rolled SVG, ~60 lines.
`saved` and `withdrawn` rows are excluded — they were never sent, or were pulled back. The
three interview rounds share one segment: what the bar is asked is how far applications
got, not which round each is sitting on. Anything past `applied` counts as an answer.

### Vault

Accordion by experience. Each experience: org, location, kind, skill tags, then roles, then
editable bullet rows. Inline edit, save on blur. Each bullet has a small **Improve** action
that returns 2–3 grounded suggestions in a diff view; accepting one replaces
`bullets.text` and archives the previous wording as a variant.

Skill input is a comma-separated field that upserts into `skills` by slug — no separate
skill management screen.

### Base

A template picker, a preview, and the same edit-then-save flow as Generate. No URL field
and no model: the tab renders the whole vault through one template, ranked by
`pipeline/strength.rs` rather than by fit, and saves the result under a name.

Editing a template opens its Tera source. A built-in cannot be saved in place — it is
re-synced from `templates/*.tex.tera` on every health check, so the files stay
authoritative and an edit saves a copy. One row is `is_active`; that is what Generate
renders with.

---

## 8. Milestones

Each milestone ends with something runnable. No milestone depends on a later one.

**M0 — Scaffold.** `create-tauri-app`, CSS tokens, tab shell with three empty routes,
`config.rs` reading `~/.config/resume-fixer/config.toml`, `health_check` command wired to a
status strip. *Done when:* the app opens and reports Postgres/Ollama/Tectonic status.

**M1 — Schema and repositories.** Migrations from section 3, `db/pool.rs`, repositories for
profile, experience, role, bullet, skill. Integration tests against a scratch database.
*Done when:* `cargo test` round-trips a full experience tree.

**M2 — Vault tab.** Full CRUD, skill tagging, no AI. Seed the user's existing resume into
the DB via a one-off seed migration so there is real data to work with.
*Done when:* the user's whole resume exists in Postgres and is editable in the UI.

**M3 — Render path.** `render/tex.rs` escaping (unit-tested against adversarial strings),
`resume.tex.tera` derived from the reference file, Tectonic invocation, page counting.
A temporary "render everything" button proves it.
*Done when:* the generated PDF is visually indistinguishable from the reference resume.

**M4 — Ingest.** Fetch, extract, quality gate, paste fallback. Test against 5–6 real
posting URLs of different platforms and record which need paste.
*Done when:* a link yields clean job text or a clean fallback, never a crash.

**M5 — Model integration.** Ollama client with timeout and JSON-mode retry, job parsing,
retrieval scoring, selection, grounding checks. Grounding gets a dedicated test suite of
hand-written source/rewrite pairs including deliberate hallucinations.
*Done when:* every fabrication in the test suite is caught.

**M6 — Generate tab.** Full pipeline behind the UI, preview, download, regenerate with
feedback, commit-to-library, discard. Fit-to-one-page loop.
*Done when:* a link becomes a downloaded one-page PDF without touching the terminal.

**M7 — Library tab.** Table, status transitions with history rows, side panel, stacked bar.
*Done when:* status changes persist and the graph reflects them.

**M8 — Polish.** Error copy, empty states, keyboard flow (⌘1–4 tabs, ⌘Enter generate),
first-run guidance, README covering Postgres/Ollama/Tectonic setup.

**M9 — Base tab and templates.** `templates` and `base_resumes` tables, the Simplify
layout, `pipeline/strength.rs` for posting-less ranking, template editing that copies
rather than overwrites a built-in.
*Done when:* the vault renders to a one-page PDF with no posting and no model call.

---

## 9. Testing

Weighted toward the parts where a bug is silent rather than loud.

- **Grounding** (highest value): a fixture file of `(source, rewrite, expect_pass)` triples.
  Include swapped numbers, added technologies, inflated scope, invented employers, and
  legitimate rewrites that must pass. Run in CI on every change to prompts or checkers.
- **LaTeX escaping**: property test that any input string produces a document Tectonic
  compiles. Explicit cases for `&`, `%`, `$`, `_`, `#`, `\`, `~`, `^`, and combinations.
- **Retrieval scoring**: fixed vault + fixed job → asserted bullet ordering.
- **Repositories**: integration tests on a scratch DB, cascade deletes verified.
- **Ingest**: saved HTML fixtures, no live network in tests.
- **Manual**: the fit-to-page loop and the visual diff against the reference resume.

Model calls are mocked in tests via a `LlmClient` trait with a fixture implementation.
No test requires Ollama to be running.

---

## 10. Configuration

`~/.config/resume-fixer/config.toml`, created with defaults on first run:

```toml
[database]
url = "postgresql://localhost/resume_fixer"

[llm]
endpoint = "http://localhost:11434"
model = "ornith:35b"
timeout_secs = 180
temperature = 0.2        # low: this is a selection task, not a creative one

[render]
tectonic_path = "tectonic"          # or an absolute path
output_dir = "~/Documents/Resumes"  # where downloads land

[ingest]
timeout_secs = 10
min_chars = 400
```

No secrets, so plain TOML is fine. The file is the single source of truth; nothing is
hardcoded in Rust beyond the defaults that write it.

---

## 11. Risks

| Risk | Mitigation |
|---|---|
| A 35B model returns malformed JSON | Ollama `format: "json"`, serde validation, one retry, deterministic fallback path |
| Model invents a bullet anyway | Structurally impossible — the renderer only accepts bullet IDs; text is gated by section 4.5 |
| Resume overflows one page | Fit loop drops lowest-scoring bullets, warns if it cannot converge |
| Job sites block scraping | Paste fallback is a first-class path, not an error state |
| Tectonic first-run download looks like a hang | Explicit "preparing LaTeX packages, first run only" status |
| Generation is slow on a 35B model | Two calls, not per-bullet. Stage-by-stage status text. Budget ~30–90s |
| Skill tags drift (`Postgres` vs `PostgreSQL`) | `slug` uniqueness + `aliases[]`, matching always by slug |

---

## 12. Deferred, with hooks left in place

- Cover letters — same pipeline, different template and prompt.
- Embedding-based retrieval — `pipeline/retrieval.rs` is a single scoring function to swap.
- Response-time analytics — `application_status_history` already records the data.
