-- ── Templates ───────────────────────────────────────────────────────────
-- Built-in rows are re-synced from templates/*.tex.tera on every startup; user rows are
-- whatever was pasted into the editor.
CREATE TABLE templates (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL UNIQUE,
    source     TEXT NOT NULL,          -- Tera over LaTeX
    is_builtin BOOLEAN NOT NULL DEFAULT FALSE,
    is_active  BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Base resumes ────────────────────────────────────────────────────────
-- A rendered resume with no job behind it. `template_name` is a snapshot, not a foreign
-- key: tex_source is self-contained, so deleting a template must not restrict this row.
CREATE TABLE base_resumes (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          TEXT NOT NULL,
    template_name TEXT NOT NULL,
    tex_source    TEXT NOT NULL,
    pdf_path      TEXT,
    page_count    INT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ON base_resumes (created_at DESC);
