-- Holds every bullet to at most two rendered lines.
--
-- Nine bullets wrapped onto a third line at the template's 507pt item width. A three-line
-- bullet costs the fit loop an entire entry elsewhere and reads as a paragraph rather than
-- a result, so the ceiling is two.
--
-- Two of the nine carried two separable achievements and were split, which loses nothing.
-- The other seven were tightened: no fact, number, or named technology was dropped, only
-- words that were doing no work. Where an enumeration had to give ground, the count stayed
-- ("9 callable tools") and the examples shrank, so the scale survives the cut.

-- UVA School of Data Science. The pipeline work and the paper are separate achievements
-- and the paper's title does not compress; splitting keeps both at full strength.
UPDATE bullets SET text =
    'Engineered Python data pipelines with Pandas and NumPy to analyze satellite launch datasets across LEO, MEO, and GEO regimes'
WHERE id = '33333333-3333-4333-8333-000000000006';

INSERT INTO bullets (id, role_id, text, display_order) VALUES
    ('33333333-3333-4333-8333-000000030001', '22222222-2222-4222-8222-000000000004',
     'Presented a paper at IEEE SIEDS 2026 on decision support for resilient foundation model scaling in orbital computing', 2);

UPDATE bullets SET text =
    'Quantified communications satellites at approximately 60% of recent payloads against 3-5% for technology satellites, grounding exponential, logistic, and quadratic projection models for orbital compute infrastructure'
WHERE id = '33333333-3333-4333-8333-000000000007';

-- Panacea. The award and the architecture were one 307-character bullet, the longest in the
-- vault. Split so the win leads and the immunity model stands on its own; the rest of the
-- entry shifts down one to keep the pair adjacent.
UPDATE bullets SET text =
    'Won Best Pitch at United Hacks V7 against 100+ competing teams for a decentralized anti-virus platform modeled on biological immunology, built in 3,200+ lines of Rust across a 3-person team'
WHERE id = '33333333-3333-4333-8333-000000010101';

UPDATE bullets SET display_order = display_order + 1
WHERE role_id = '22222222-2222-4222-8222-000000000011' AND display_order >= 1;

INSERT INTO bullets (id, role_id, text, display_order) VALUES
    ('33333333-3333-4333-8333-000000030002', '22222222-2222-4222-8222-000000000011',
     'Built endpoints that detect threats locally and publish verified cures on-chain so the whole network inherits immunity', 1);

UPDATE bullets SET text =
    'Designed a 4-stage verification pipeline every candidate cure clears before publication: behavioral scoring against a 100-point threshold, sandboxed fuzzing, whitelist regression testing, and 3-of-5 multisig consensus'
WHERE id = '33333333-3333-4333-8333-000000010102';

-- EVE. "provider-agnostic" here duplicated the LiteLLM bullet below it, and "mechanically
-- verified" duplicated the guardrails bullet; both were dropped where they repeated.
UPDATE bullets SET text =
    'Architected a voice AI agent in 8,400+ lines of Python, wiring a microphone to faster-whisper speech-to-text, an LLM, and text-to-speech behind abstract interfaces so any subsystem swaps without touching the orchestrator'
WHERE id = '33333333-3333-4333-8333-000000010201';

UPDATE bullets SET text =
    'Engineered an autonomous self-improvement loop across 10 modules and 1,470 lines in which researcher, engineer, and reviewer subagents modify the codebase during idle time, gated on a passing test suite before any commit'
WHERE id = '33333333-3333-4333-8333-000000010202';

UPDATE bullets SET text =
    'Enforced self-modification safety with mechanical guardrails, not prompt instructions: git-worktree sandboxing, branch revalidation at commit, an unreachable memory directory, and rejection of diffs adding deletion calls'
WHERE id = '33333333-3333-4333-8333-000000010203';

-- Wrangler. Library availability and bus data have their own bullets in this entry, so
-- naming them again in the tool list spent two lines to say one thing twice.
UPDATE bullets SET text =
    'Built a multi-step agentic loop on Google Gemini 2.5 Flash exposing 9 callable tools across web search, page extraction, and Google Calendar CRUD, so the model reasons over live campus data instead of stale training knowledge'
WHERE id = '33333333-3333-4333-8333-000000010302';

-- Project Lantern. "navigation" is carried by the rest of the sentence, which describes
-- exactly the navigating.
UPDATE bullets SET text =
    'Built a multimodal healthcare assistant in 10,900+ lines of TypeScript, turning insurance letters and discharge summaries into eligibility answers, pre-filled forms, and deadline-tagged checklists for lower-income patients'
WHERE id = '33333333-3333-4333-8333-000000010401';

-- Tags for the two new lines, matching the bullets they were split from.
INSERT INTO bullet_skills (bullet_id, skill_id)
SELECT b.bullet_id::uuid, s.id
FROM (VALUES
    ('33333333-3333-4333-8333-000000030001', ARRAY['python']),
    ('33333333-3333-4333-8333-000000030002', ARRAY['blockchain','rust'])
) AS b(bullet_id, slugs)
JOIN skills s ON s.slug = ANY(b.slugs)
ON CONFLICT DO NOTHING;

-- The two-line ceiling is a property of the rendered width, not of character count: the
-- boundary sits near 230 characters but moves with word length. Measure, do not count.

-- Every tagged skill prints. Applied here rather than in `migrations/0009_skill_list_flag.sql`
-- because the schema is now current before any of this seed runs, so 0009 sees no tags.
UPDATE skills SET always_list = TRUE
WHERE id IN (SELECT skill_id FROM bullet_skills)
   OR id IN (SELECT skill_id FROM experience_skills);
