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
    org_name      TEXT NOT NULL,
    location      TEXT,
    url           TEXT,
    tech_line     TEXT,
    display_order INT NOT NULL DEFAULT 0,
    is_active     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- One experience can hold several roles (e.g. intern -> returning intern at one company).
CREATE TABLE roles (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    experience_id  UUID NOT NULL REFERENCES experiences(id) ON DELETE CASCADE,
    title          TEXT NOT NULL,
    location       TEXT,
    start_date     DATE NOT NULL,
    end_date       DATE,
    date_override  TEXT,
    display_order  INT NOT NULL DEFAULT 0,
    is_active      BOOLEAN NOT NULL DEFAULT TRUE
);
CREATE INDEX ON roles (experience_id);

CREATE TABLE bullets (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id        UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    text           TEXT NOT NULL,          -- plain text, NO LaTeX markup
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
    category   TEXT,
    aliases    TEXT[] NOT NULL DEFAULT '{}'
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
    job_text      TEXT NOT NULL,
    job_source    job_source NOT NULL,
    parsed        JSONB NOT NULL DEFAULT '{}',
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
    pdf_path       TEXT,
    model          TEXT NOT NULL,
    prompt_version TEXT NOT NULL,
    feedback       TEXT,
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
