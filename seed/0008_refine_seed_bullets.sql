-- Refines the originally seeded bullets against the bullet standard:
-- action verb -> what was built -> named technology -> measurable outcome.
--
-- No number here is new. Every metric already existed in the seed; the edits surface it,
-- split run-on bullets carrying two ideas into two selectable lines, drop trailing periods
-- for consistency, and remove straight quotes that would render wrong in LaTeX.
-- Where a bullet has no metric, none was invented — see the notes at the bottom.

-- Rajant Health, Software Engineering Intern
UPDATE bullets SET text =
    'Designed semantic-embedding algorithms for deduplication, provenance, and incremental integration in agentic database consolidation, shaping the production agentic architecture and its human-in-the-loop review step'
WHERE id = '33333333-3333-4333-8333-000000000002';

UPDATE bullets SET text =
    'Developed a Backend-for-Frontend service in Python linking hardware sensor data to a third-party proprietary API, moving it through PostgreSQL and InfluxDB behind a modular, authenticated boundary'
WHERE id = '33333333-3333-4333-8333-000000000003';

UPDATE bullets SET text =
    'Built 3 FastAPI endpoints in a Python Backend-for-Frontend service, exporting live InfluxDB3 data as CSV and JSON to an external adapter service'
WHERE id = '7e2fef8c-1608-4d46-84b6-e1ed4c85240c';

UPDATE bullets SET text =
    'Prototyped a graph-based algorithm for semantic incremental database consolidation, shaping how memory management was designed for the team agentic applications'
WHERE id = 'd51f0579-97e1-4893-bae0-650d069974dc';

UPDATE bullets SET text =
    'Re-architected a single-page site into a 12-page, 5-locale information architecture with server-side i18n publishing and mega-menu navigation'
WHERE id = 'ffa3dacb-084f-4a97-a41e-8fb4f7ff7673';

UPDATE bullets SET text =
    'Built a dependency-free vanilla JavaScript search engine with BM25F ranking, fuzzy did-you-mean suggestions, and snippet highlighting, covering 60 pages with full SEO and GEO metadata'
WHERE id = '541a1e6c-6988-49fa-96bf-b631ca53f419';

UPDATE bullets SET text =
    'Cut backend processing time by approximately 89% by optimizing PostgreSQL queries to remove redundant computation'
WHERE id = 'fa1707fb-5946-40d7-aa6c-9d01bf713ab4';

-- Rajant Health, Computer Engineering Intern. One bullet carried two quantified results
-- behind a semicolon; split so retrieval can choose either on its own merit.
UPDATE bullets SET text =
    'Delivered a proof of concept validating the NXP MIMXRT595 as a next-generation hardware platform, driving the decision to deprecate the legacy board'
WHERE id = '33333333-3333-4333-8333-000000000004';

UPDATE bullets SET text =
    'Implemented a lock-free UART pipeline using ring buffers, sustaining 460 kbps continuous transfer for real-time PPG sensor ingestion'
WHERE id = '33333333-3333-4333-8333-000000000005';

INSERT INTO bullets (id, role_id, text, display_order) VALUES
    ('33333333-3333-4333-8333-000000020001', '22222222-2222-4222-8222-000000000003',
     'Designed a 2D renderer with DMA-driven updates, holding frame latency under 15 ms on the embedded display', 2);

-- UVA School of Data Science
UPDATE bullets SET text =
    'Engineered Python data pipelines with Pandas and NumPy to analyze satellite launch datasets across LEO, MEO, and GEO regimes, producing a paper presented at IEEE SIEDS 2026 on decision support for resilient foundation model scaling in orbital computing'
WHERE id = '33333333-3333-4333-8333-000000000006';

UPDATE bullets SET text =
    'Established that communications satellites account for approximately 60% of recent payloads against 3-5% for technology satellites, grounding exponential, logistic, and quadratic projection models for computation-capable orbital infrastructure'
WHERE id = '33333333-3333-4333-8333-000000000007';

-- Talent Skincare
UPDATE bullets SET text =
    'Automated Amazon Ads campaign data extraction with a serverless Python function on AWS Lambda wired to the Monday.com API, replacing a manual reporting workflow'
WHERE id = '33333333-3333-4333-8333-000000000008';

-- Sniped
UPDATE bullets SET text =
    'Designed and deployed a RESTful API of 20+ endpoints on AWS EC2 with Docker, an Nginx reverse proxy, and automated CI/CD through GitLab pipelines'
WHERE id = '33333333-3333-4333-8333-000000000009';

UPDATE bullets SET text =
    'Implemented JWT authentication with Row-Level Security across 10+ PostgreSQL tables, including a dual-client authorization pattern separating service-role from public access'
WHERE id = '33333333-3333-4333-8333-00000000000a';

UPDATE bullets SET text =
    'Released the iOS client to 20+ beta testers through Apple TestFlight review, served by the deployed EC2 backend'
WHERE id = '33333333-3333-4333-8333-00000000000b';

-- Microsoft Garage. Split on the semicolon: the interface and the model integration are
-- separate pieces of work and match different postings.
UPDATE bullets SET text =
    'Developed the front-end interface and Firebase data pipeline for a network vulnerability detection tool'
WHERE id = '33333333-3333-4333-8333-00000000000c';

INSERT INTO bullets (id, role_id, text, display_order) VALUES
    ('33333333-3333-4333-8333-000000020002', '22222222-2222-4222-8222-000000000007',
     'Integrated the ChatGPT API for vulnerability analysis, tuning request patterns for cost and performance in constrained environments', 1);

-- PoliDex. One bullet held three ideas and buried the award at the end; split so the win
-- leads and the algorithm work stands on its own.
UPDATE bullets SET text =
    'Won Best Use of MongoDB Atlas at Hackabull VII for a political alignment engine built and shipped in 24 hours'
WHERE id = '33333333-3333-4333-8333-00000000000d';

INSERT INTO bullets (id, role_id, text, display_order) VALUES
    ('33333333-3333-4333-8333-000000020003', '22222222-2222-4222-8222-000000000008',
     'Scored politicians across a 20-dimensional policy vector space using a custom variance-weighted cosine similarity formula over a Java, Python, and Next.js architecture', 1),
    ('33333333-3333-4333-8333-000000020004', '22222222-2222-4222-8222-000000000008',
     'Indexed the policy space with a k-d tree for sublinear 20-dimensional nearest-neighbor search', 2);

-- Tags for the new lines, and for the two originals that had none.
INSERT INTO bullet_skills (bullet_id, skill_id)
SELECT b.bullet_id::uuid, s.id
FROM (VALUES
    ('33333333-3333-4333-8333-000000020001', ARRAY['embedded-c','c','dma','uart']),
    ('33333333-3333-4333-8333-000000020002', ARRAY['javascript','rest-apis','firebase']),
    ('33333333-3333-4333-8333-000000020003', ARRAY['java','python','next-js','mongodb']),
    ('33333333-3333-4333-8333-000000020004', ARRAY['java','python']),
    ('d51f0579-97e1-4893-bae0-650d069974dc', ARRAY['python','agentic-development','postgresql'])
) AS b(bullet_id, slugs)
JOIN skills s ON s.slug = ANY(b.slugs)
-- Some ids below were authored in the Vault rather than by a seed, so they exist in the
-- author's database and not in a fresh one. Joining `bullets` skips what is not there
-- instead of failing the whole migration on a foreign key.
JOIN bullets bl ON bl.id = b.bullet_id::uuid
ON CONFLICT DO NOTHING;

-- Notes on what was NOT changed, so the next reader does not redo this work:
--
--   * The University of Virginia coursework line has no verb and no outcome by design.
--     It is a list, which is what a coursework line should be.
--   * No number was added anywhere. The bullets still missing a measurable outcome are
--     Rajant's Backend-for-Frontend and semantic-embedding lines, Talent Skincare, and
--     Microsoft Garage. Each needs a figure only the author can supply: rows consolidated,
--     hours of reporting saved, requests served, vulnerabilities found.
