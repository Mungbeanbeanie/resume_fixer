# Resume Fixer

A local-first desktop app that keeps your full experience history in Postgres, tailors an
ATS-optimized LaTeX resume to one job posting using a model running on your own machine,
and tracks which resume went to which company.

Nothing leaves the machine. The only outbound request is fetching the job posting URL you
type. No analytics, no cloud model, no third-party extraction service.

## What it does

**Generate** — paste a job link (or the description), get a one-page PDF. Every printed
line comes from a bullet you already wrote; the model picks and re-emphasizes, and a
deterministic grounding check rejects any rewrite that adds a fact.

**Library** — every application, the exact resume that was sent, and where it got to:
applied, online assessment, interview round, offer. One stacked bar and a response rate.

**Vault** — your experiences, roles, and bullets, with skill tags that drive retrieval.
The Improve button offers alternate phrasings, grounded against what the bullet already
says.

**Base** — your whole vault through one template, no posting and no model, every line
verbatim. Pick a layout (Jake's Resume or Simplify), save it under a name, keep several.

## Setup

Four local pieces. The status strip in the top right tells you which are missing.

```bash
# 1. Postgres — the server only. The app creates its own database on first launch and
#    migrates it, so there is no createdb step and no schema to import.
brew install postgresql@17
brew services start postgresql@17
# The vault starts empty. Fill it from the Vault tab: Header Info first, then an entry
# under School or Work. Generate and Base both say so until there is a bullet to print.

# 2. Ollama with the model
brew install ollama
ollama serve &
ollama pull ornith:35b

# 3. Tectonic (the LaTeX engine — no TeX Live needed)
brew install tectonic
# The first compile downloads packages into ~/.cache/Tectonic. That pause is not a hang.

# 4. The app
npm install
npm run tauri dev
```

Configuration lives in `~/.config/resume-fixer/config.toml`, written with defaults on first
run: database URL, Ollama endpoint and model, Tectonic path, download directory, ingest
timeouts. Nothing is hardcoded past those defaults.

Installing from a `.dmg` rather than from source: the build is unsigned, so Gatekeeper
refuses it until you right-click the app and choose Open once. Signing it properly needs an
Apple Developer certificate.

## Using it

- `⌘1`–`⌘4` switch tabs, `⌘↵` generates.
- A posting that will not extract (Workday, Greenhouse embeds, anything behind a login)
  falls back to a paste box. That is a normal path, not an error.
- Generation takes 30–90 seconds on a local 35B model: two model calls, not one per bullet.
- Drafts live in memory until you save them. Discarding writes nothing.
- Edit before saving: uncheck a line to drop it, check one to bring it back, or rewrite it.
- To fit one page the app retires whole entries before it drops lines, and tells you which.

## Development

```bash
# Rust
cd src-tauri
cargo test                       # unit tests; no database, model, or LaTeX needed
cargo clippy --all-targets -- -D warnings
cargo fmt

# The suites that matter most
cargo test grounding             # every fabrication in the fixture must be caught
cargo test escaping              # LaTeX escaping against adversarial strings

# Tests that need the real stack, ignored by default
createdb resume_fixer_test
export TEST_DATABASE_URL=postgresql://localhost/resume_fixer_test
cargo test --test db                                        # repositories, cascades
cargo test --test render_reference -- --ignored --nocapture # PDF parity with the reference
cargo test --test generate_end_to_end -- --ignored          # the whole pipeline, live

# Frontend
npm run lint                     # tsc --noEmit
npm run build

# The maintainer's own vault. Not a migration and not compiled into the binary, so a
# distributed build ships an empty database.
for f in ../seed/*.sql; do psql -d resume_fixer -f "$f"; done

# What a released build must not contain
npm run tauri build
strings "target/release/bundle/macos/Resume Fixer.app/Contents/MacOS/resume-fixer" \
  | grep -i "mychung\|rajant\|mungbean"      # expect nothing
```

`IMPLEMENTATION_PLAN.md` holds the schema, the pipeline design, and the milestone order.
`CLAUDE.md` holds the rules that are not negotiable — chief among them that no resume line
may exist without a `bullets` row behind it.

## How the anti-hallucination guarantee works

Two halves, one structural and one textual.

**Structural:** the renderer accepts bullet IDs, not text. `resume_bullets` has a foreign
key to `bullets`, so a line that cannot be attributed cannot be stored — and the pipeline
builds its plan from stored bullets only.

**Textual:** every reworded bullet passes `pipeline/grounding.rs` before rendering. A new
number, a new named entity, a technology the bullet is not tagged with, an unearned
superlative, a length outside 0.7×–1.25× of the source, or any LaTeX markup means the
rewrite is dropped and the stored wording is used instead. Rejections are shown in the
Generate tab, not hidden.
