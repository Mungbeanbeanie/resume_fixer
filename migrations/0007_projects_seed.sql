-- Seven projects, counted from the repos in Hackathons_and_projects/.
-- Text is stored plain: no LaTeX markup, no escaping. render/tex.rs owns escaping.
--
-- Role dates come from each repo's first and last commit, not from memory. Same-month
-- spans collapse to one stamp in render::tex::format_dates, so a weekend hackathon prints
-- as "Mar 2026" without a date_override.
--
-- Every number here was counted from a repo. Two claims from the source notes were cut
-- rather than kept: a latency improvement and a propagation time that were never measured.

INSERT INTO skills (name, slug) VALUES
    ('Rust', 'rust'),
    ('Solana', 'solana'),
    ('Anchor', 'anchor'),
    ('WebAssembly', 'webassembly'),
    ('Tauri', 'tauri'),
    ('Vite', 'vite'),
    ('Cargo', 'cargo'),
    ('GitHub Actions', 'github-actions'),
    ('LiteLLM', 'litellm'),
    ('FAISS', 'faiss'),
    ('mem0', 'mem0'),
    ('FastEmbed', 'fastembed'),
    ('ONNX', 'onnx'),
    ('faster-whisper', 'faster-whisper'),
    ('ElevenLabs', 'elevenlabs'),
    ('asyncio', 'asyncio'),
    ('pytest', 'pytest'),
    ('Server-Sent Events', 'server-sent-events'),
    ('HTML5 Canvas', 'html5-canvas'),
    ('OAuth 2.0', 'oauth-2-0'),
    ('Google Gemini', 'google-gemini'),
    ('Google Calendar API', 'google-calendar-api'),
    ('Google Workspace APIs', 'google-workspace-apis'),
    ('Tavily', 'tavily'),
    ('Firecrawl', 'firecrawl'),
    ('GTFS', 'gtfs'),
    ('Railway', 'railway'),
    ('Vercel', 'vercel'),
    ('Jest', 'jest'),
    ('Vitest', 'vitest'),
    ('Tailwind CSS', 'tailwind-css'),
    ('Google Cloud Run', 'google-cloud-run'),
    ('Twilio', 'twilio'),
    ('Ollama', 'ollama'),
    ('VS Code Extension API', 'vs-code-extension-api'),
    ('Bash', 'bash'),
    ('Neural Networks', 'neural-networks'),
    ('Genetic Algorithms', 'genetic-algorithms'),
    ('Sandboxing', 'sandboxing'),
    ('Blockchain', 'blockchain'),
    ('CI/CD', 'ci-cd'),
    ('Unit Testing', 'unit-testing')
ON CONFLICT (slug) DO NOTHING;

UPDATE skills SET aliases = ARRAY['gemini', 'gemini 2.5 flash', 'google ai']    WHERE slug = 'google-gemini';
UPDATE skills SET aliases = ARRAY['wasm', 'wasmi']                              WHERE slug = 'webassembly';
UPDATE skills SET aliases = ARRAY['oauth', 'oauth2']                            WHERE slug = 'oauth-2-0';
UPDATE skills SET aliases = ARRAY['sse']                                        WHERE slug = 'server-sent-events';
UPDATE skills SET aliases = ARRAY['canvas']                                     WHERE slug = 'html5-canvas';
UPDATE skills SET aliases = ARRAY['anchor framework']                           WHERE slug = 'anchor';
UPDATE skills SET aliases = ARRAY['whisper', 'speech to text', 'stt']           WHERE slug = 'faster-whisper';
UPDATE skills SET aliases = ARRAY['continuous integration', 'ci']               WHERE slug = 'ci-cd';
UPDATE skills SET aliases = ARRAY['vscode', 'vs code']                          WHERE slug = 'vs-code-extension-api';
UPDATE skills SET aliases = ARRAY['deep learning', 'machine learning', 'ml']    WHERE slug = 'neural-networks';

INSERT INTO experiences (id, kind, org_name, location, tech_line, display_order) VALUES
    ('11111111-1111-4111-8111-000000000011', 'project', 'Panacea -- Decentralized Autonomous Anti-Virus', NULL, 'Rust, Solana, Anchor, WebAssembly, Tauri, React', 11),
    ('11111111-1111-4111-8111-000000000012', 'project', 'EVE -- Local-First Voice AI Agent',              NULL, 'Python, LiteLLM, FAISS, mem0, faster-whisper, asyncio', 12),
    ('11111111-1111-4111-8111-000000000013', 'project', 'Wrangler -- AI Campus Assistant',                NULL, 'Next.js, React, Express, PostgreSQL, Google Gemini', 13),
    ('11111111-1111-4111-8111-000000000014', 'project', 'Project Lantern -- Healthcare Access Assistant', NULL, 'TypeScript, Next.js, Express, Google Gemini, Twilio, Docker', 14),
    ('11111111-1111-4111-8111-000000000015', 'project', 'Explainable -- VS Code Extension',               NULL, 'TypeScript, VS Code Extension API, Google Gemini, Node.js', 15),
    ('11111111-1111-4111-8111-000000000016', 'project', 'wtfgit -- Git Diagnostic CLI',                   NULL, 'Python, Bash, git plumbing', 16),
    ('11111111-1111-4111-8111-000000000017', 'project', 'Neural Network Self-Driving Car Simulation',     NULL, 'JavaScript, HTML5 Canvas, neural networks', 17);

INSERT INTO roles (id, experience_id, title, start_date, end_date, display_order) VALUES
    ('22222222-2222-4222-8222-000000000011', '11111111-1111-4111-8111-000000000011', 'Panacea',     '2026-07-10', '2026-07-12', 0),
    ('22222222-2222-4222-8222-000000000012', '11111111-1111-4111-8111-000000000012', 'EVE',         '2026-06-26', '2026-07-01', 0),
    ('22222222-2222-4222-8222-000000000013', '11111111-1111-4111-8111-000000000013', 'Wrangler',    '2026-03-21', '2026-03-22', 0),
    ('22222222-2222-4222-8222-000000000014', '11111111-1111-4111-8111-000000000014', 'Project Lantern', '2026-04-11', '2026-04-12', 0),
    ('22222222-2222-4222-8222-000000000015', '11111111-1111-4111-8111-000000000015', 'Explainable', '2026-04-18', '2026-04-18', 0),
    ('22222222-2222-4222-8222-000000000016', '11111111-1111-4111-8111-000000000016', 'wtfgit',      '2026-07-01', '2026-07-07', 0),
    ('22222222-2222-4222-8222-000000000017', '11111111-1111-4111-8111-000000000017', 'Self-Driving Car Simulation', '2026-07-14', '2026-07-22', 0);

INSERT INTO bullets (id, role_id, text, display_order) VALUES
    -- Panacea
    ('33333333-3333-4333-8333-000000010101', '22222222-2222-4222-8222-000000000011',
     'Won Best Pitch at United Hacks V7 against 100+ competing teams for a decentralized anti-virus platform modeled on biological immunology, built in 3,200+ lines of Rust across a 3-person team, in which endpoints detect threats locally and publish verified cures on-chain so the whole network inherits immunity', 0),
    ('33333333-3333-4333-8333-000000010102', '22222222-2222-4222-8222-000000000011',
     'Designed a 4-stage verification pipeline -- behavioral trajectory scoring against a 100-point threshold, sandboxed fuzzing, regression testing against a whitelisted app set, and 3-of-5 multisig consensus -- that every candidate cure clears before global publication', 1),
    ('33333333-3333-4333-8333-000000010103', '22222222-2222-4222-8222-000000000011',
     'Guaranteed sandbox isolation by executing candidate remediation genes as WebAssembly bytecode in a wasmi runtime compiled with zero host imports, making host access structurally impossible rather than policy-enforced', 2),
    ('33333333-3333-4333-8333-000000010104', '22222222-2222-4222-8222-000000000011',
     'Deployed an Anchor program to Solana devnet with two PDA-backed registries, executing signed transactions and reading at confirmed and finalized commitment with no mocked chain layer', 3),
    ('33333333-3333-4333-8333-000000010105', '22222222-2222-4222-8222-000000000011',
     'Built a network-wide kill switch in which a 3-of-5 multisig authority flips an on-chain status flag that every agent checks before fetching a gene, halting a bad cure without a chain fork', 4),
    ('33333333-3333-4333-8333-000000010106', '22222222-2222-4222-8222-000000000011',
     'Implemented behavioral malware detection in the Scout daemon by spawning live test processes and identifying file-descriptor bursts and network activity via lsof, rather than cooperative self-reporting', 5),
    ('33333333-3333-4333-8333-000000010107', '22222222-2222-4222-8222-000000000011',
     'Removed an external service and its failure mode by moving gene payloads on-chain after determining a serialized gene was smaller than the 64-character IPFS CID that would have referenced it', 6),
    ('33333333-3333-4333-8333-000000010108', '22222222-2222-4222-8222-000000000011',
     'Guarded every merge with 31 Rust tests and a 3-OS GitHub Actions CI matrix across Ubuntu, macOS, and Windows, enforcing strict Rust-core and React-observability separation across the team', 7),

    -- EVE
    ('33333333-3333-4333-8333-000000010201', '22222222-2222-4222-8222-000000000012',
     'Architected a provider-agnostic voice AI agent in 8,400+ lines of Python, wiring a microphone to faster-whisper speech-to-text, an LLM, and text-to-speech behind abstract interfaces so any subsystem swaps without touching the orchestrator', 0),
    ('33333333-3333-4333-8333-000000010202', '22222222-2222-4222-8222-000000000012',
     'Engineered an autonomous self-improvement loop across 10 modules and 1,470 lines in which researcher, engineer, and reviewer subagents propose and implement codebase changes during idle time, gated on a mechanically verified passing test suite before any commit', 1),
    ('33333333-3333-4333-8333-000000010203', '22222222-2222-4222-8222-000000000012',
     'Enforced self-modification safety through mechanical guardrails rather than prompt instructions -- git-worktree sandboxing, branch-name revalidation at commit, an unreachable memory directory, and automatic rejection of diffs introducing deletion calls', 2),
    ('33333333-3333-4333-8333-000000010204', '22222222-2222-4222-8222-000000000012',
     'Built a three-tier memory system spanning working, procedural, and episodic recall on mem0 and a local FAISS index with in-process FastEmbed ONNX embeddings, running fully on-device with no database or network service', 3),
    ('33333333-3333-4333-8333-000000010205', '22222222-2222-4222-8222-000000000012',
     'Eliminated vendor lock-in by routing all inference through LiteLLM, letting users switch between Anthropic, OpenAI, Gemini, and Ollama models via a single environment variable and zero code changes', 4),
    ('33333333-3333-4333-8333-000000010206', '22222222-2222-4222-8222-000000000012',
     'Wrote 139 hardware-free unit and wiring tests covering memory, tool execution, the audio pipeline, and every self-improvement guardrail, enabling CI without a microphone or API keys', 5),
    ('33333333-3333-4333-8333-000000010207', '22222222-2222-4222-8222-000000000012',
     'Exposed 5 external tools across Gmail, Google Calendar, Drive, and Tavily web search through a typed tool registry with a human confirmation gate on every destructive action', 6),
    ('33333333-3333-4333-8333-000000010208', '22222222-2222-4222-8222-000000000012',
     'Shipped a zero-dependency HTML5 Canvas visualizer driven by Server-Sent Events, mirroring live agent state from listening to thinking to speaking with no Electron or npm build step', 7),

    -- Wrangler
    ('33333333-3333-4333-8333-000000010301', '22222222-2222-4222-8222-000000000013',
     'Shipped a production AI campus assistant to wrangleratuva.us during a weekend hackathon, deploying an Express backend on Railway and a Next.js 15 and React 19 frontend on Vercel', 0),
    ('33333333-3333-4333-8333-000000010302', '22222222-2222-4222-8222-000000000013',
     'Built a multi-step agentic loop on Google Gemini 2.5 Flash exposing 9 callable tools across web search, page extraction, dining menus, library availability, booking guidance, and Google Calendar CRUD, so the model reasons over live campus data instead of stale training knowledge', 1),
    ('33333333-3333-4333-8333-000000010303', '22222222-2222-4222-8222-000000000013',
     'Integrated the LibCal grid API to surface real-time slot availability for 69 study rooms across 9 UVA libraries, handling midnight-spillover hours for late-night bookings', 2),
    ('33333333-3333-4333-8333-000000010304', '22222222-2222-4222-8222-000000000013',
     'Prevented structured-data hallucination by appending tool results as post-response markers, keeping raw JSON entirely out of the model context window', 3),
    ('33333333-3333-4333-8333-000000010305', '22222222-2222-4222-8222-000000000013',
     'Implemented Google OAuth 2.0 and JWT authentication backed by PostgreSQL, persisting conversation history across sessions and personalizing responses by school and year', 4),
    ('33333333-3333-4333-8333-000000010306', '22222222-2222-4222-8222-000000000013',
     'Routed simple factual questions straight to a streaming response with regex-based intent detection, reserving the full tool-calling loop for queries that need live data', 5),
    ('33333333-3333-4333-8333-000000010307', '22222222-2222-4222-8222-000000000013',
     'Parsed and served live UVA bus positions from the TransLoc GTFS feed, rendering route and stop data in a split-panel view alongside the chat stream', 6),

    -- Project Lantern
    ('33333333-3333-4333-8333-000000010401', '22222222-2222-4222-8222-000000000014',
     'Built a multimodal healthcare navigation assistant across 10,900+ lines of TypeScript, turning insurance letters and discharge summaries into eligibility summaries, pre-filled forms, and deadline-tagged checklists for lower-income patients', 0),
    ('33333333-3333-4333-8333-000000010402', '22222222-2222-4222-8222-000000000014',
     'Wrote 251 unit and integration tests across 13 Vitest suites covering document extraction, prior-authorization form schemas, conversation phase logic, session memory, and the SMS webhook', 1),
    ('33333333-3333-4333-8333-000000010403', '22222222-2222-4222-8222-000000000014',
     'Designed a stateless, ephemeral-session architecture that retains no patient data by default, minimizing PHI exposure and compliance surface for an MVP handling sensitive medical documents', 2),
    ('33333333-3333-4333-8333-000000010404', '22222222-2222-4222-8222-000000000014',
     'Drove document, voice, and text reasoning through Google Gemini with a JSON-first response contract, so the UI and messaging channel render structured artifacts instead of raw model prose', 3),
    ('33333333-3333-4333-8333-000000010405', '22222222-2222-4222-8222-000000000014',
     'Extended reach to phone-first users by integrating Twilio WhatsApp and SMS, letting patients upload documents and receive next steps without a browser or account', 4),
    ('33333333-3333-4333-8333-000000010406', '22222222-2222-4222-8222-000000000014',
     'Added ElevenLabs speech-to-text and text-to-speech for low-literacy and hands-free users, an accessibility requirement for the target population', 5),
    ('33333333-3333-4333-8333-000000010407', '22222222-2222-4222-8222-000000000014',
     'Containerized the Express backend with Docker for Google Cloud Run and deployed the Next.js frontend to Vercel, enabling repeatable builds and fast rollback during judging', 6),

    -- Explainable
    ('33333333-3333-4333-8333-000000010501', '22222222-2222-4222-8222-000000000015',
     'Developed a TypeScript VS Code extension in 1,400 lines that closes the comprehension gap for students receiving AI-generated code, delivering plain-English explanations without leaving the editor', 0),
    ('33333333-3333-4333-8333-000000010502', '22222222-2222-4222-8222-000000000015',
     'Built a dual-pane webview pairing a beginner-level explanation generated by Google Gemini 2.5 Flash and grounded in the user actual variable names with an editable, runnable scaffold of the same concept', 1),
    ('33333333-3333-4333-8333-000000010503', '22222222-2222-4222-8222-000000000015',
     'Implemented a local execution runner that spawns the user installed runtime as a Node.js subprocess and streams stdout and exit codes back into the panel', 2),
    ('33333333-3333-4333-8333-000000010504', '22222222-2222-4222-8222-000000000015',
     'Shipped 4 registered commands wired into editor and Explorer context menus, plus an Activity Bar tree view persisting every explanation as a replayable session', 3),
    ('33333333-3333-4333-8333-000000010505', '22222222-2222-4222-8222-000000000015',
     'Secured user credentials by storing the Gemini API key in the VS Code SecretStorage API with a first-use prompt and a dedicated reset command, keeping keys out of settings files and version control', 4),

    -- wtfgit
    ('33333333-3333-4333-8333-000000010601', '22222222-2222-4222-8222-000000000016',
     'Built a zero-dependency Python CLI that diagnoses a broken git repository and explains in plain English what is wrong, why it happened, and the exact commands to fix it', 0),
    ('33333333-3333-4333-8333-000000010602', '22222222-2222-4222-8222-000000000016',
     'Implemented 17 independent detector rules covering merge conflicts, interrupted rebase, cherry-pick, revert and bisect, detached HEAD with stranded commits, diverged branches, stale index locks, and forgotten stashes', 1),
    ('33333333-3333-4333-8333-000000010603', '22222222-2222-4222-8222-000000000016',
     'Kept the tool strictly read-only so it never mutates repository state and only prints commands, eliminating any risk of compounding an already-broken repo', 2),
    ('33333333-3333-4333-8333-000000010604', '22222222-2222-4222-8222-000000000016',
     'Ranked findings by urgency so users see the blocking problem first rather than a flat diagnostic dump, pairing every suggested fix with a one-line description of its effect', 3),
    ('33333333-3333-4333-8333-000000010605', '22222222-2222-4222-8222-000000000016',
     'Parsed git status porcelain v2 into a typed dataclass, correctly handling rename entries, unborn branches, and detached-HEAD states', 4),
    ('33333333-3333-4333-8333-000000010606', '22222222-2222-4222-8222-000000000016',
     'Achieved single-file distribution through a curl-to-bash installer with dependency preflight checks and PATH guidance, shipping using only the Python standard library', 5),
    ('33333333-3333-4333-8333-000000010607', '22222222-2222-4222-8222-000000000016',
     'Covered parser and diagnostic logic with 11 unit tests asserting rule precedence, including that merge conflicts outrank all secondary issues', 6),

    -- Neural Network Self-Driving Car Simulation
    ('33333333-3333-4333-8333-000000010701', '22222222-2222-4222-8222-000000000017',
     'Implemented a feedforward neural network and genetic training loop from scratch in 785 lines of vanilla JavaScript with no ML libraries, build tooling, or framework, converging on a collision-free driving policy in 4 to 5 generations', 0),
    ('33333333-3333-4333-8333-000000010702', '22222222-2222-4222-8222-000000000017',
     'Accelerated convergence by evaluating a 1,000-car parallel population per generation, selecting the furthest-traveling agent as the seed brain and mutating its weights and biases via linear interpolation at a tunable rate', 1),
    ('33333333-3333-4333-8333-000000010703', '22222222-2222-4222-8222-000000000017',
     'Designed the sensory input layer as 5 ray-casting sensors feeding a 5 to 6 to 4 network whose outputs directly drive forward, left, right, and reverse controls', 2),
    ('33333333-3333-4333-8333-000000010704', '22222222-2222-4222-8222-000000000017',
     'Built the physics and collision system by hand, computing car hulls as rotated polygons and detecting damage through segment-intersection tests against road borders and traffic', 3),
    ('33333333-3333-4333-8333-000000010705', '22222222-2222-4222-8222-000000000017',
     'Rendered a live HTML5 Canvas network visualizer with animated dashed connections showing per-neuron activations and weight signs in real time, making the learning process directly observable', 4),
    ('33333333-3333-4333-8333-000000010706', '22222222-2222-4222-8222-000000000017',
     'Persisted the best-performing brain to browser localStorage, letting training resume across sessions instead of restarting from random weights', 5);

-- Experience tags: relevance the bullet prose does not spell out.
INSERT INTO experience_skills (experience_id, skill_id)
SELECT '11111111-1111-4111-8111-000000000011', id FROM skills
WHERE slug = ANY(ARRAY['rust','solana','anchor','webassembly','tauri','react','vite','cargo','github-actions','ci-cd','sandboxing','blockchain','unit-testing']);

INSERT INTO experience_skills (experience_id, skill_id)
SELECT '11111111-1111-4111-8111-000000000012', id FROM skills
WHERE slug = ANY(ARRAY['python','litellm','faiss','mem0','fastembed','onnx','faster-whisper','elevenlabs','asyncio','pytest','server-sent-events','html5-canvas','google-workspace-apis','oauth-2-0','tavily','ollama','agentic-development','unit-testing','git']);

INSERT INTO experience_skills (experience_id, skill_id)
SELECT '11111111-1111-4111-8111-000000000013', id FROM skills
WHERE slug = ANY(ARRAY['node-js','express-js','next-js','react','tailwind-css','google-gemini','postgresql','oauth-2-0','jwt-authentication','google-calendar-api','tavily','firecrawl','gtfs','rest-apis','railway','vercel','jest','javascript','typescript','agentic-development']);

INSERT INTO experience_skills (experience_id, skill_id)
SELECT '11111111-1111-4111-8111-000000000014', id FROM skills
WHERE slug = ANY(ARRAY['typescript','next-js','react','express-js','google-gemini','twilio','elevenlabs','docker','google-cloud-run','vercel','vitest','tailwind-css','unit-testing','rest-apis']);

INSERT INTO experience_skills (experience_id, skill_id)
SELECT '11111111-1111-4111-8111-000000000015', id FROM skills
WHERE slug = ANY(ARRAY['typescript','vs-code-extension-api','google-gemini','node-js','javascript']);

INSERT INTO experience_skills (experience_id, skill_id)
SELECT '11111111-1111-4111-8111-000000000016', id FROM skills
WHERE slug = ANY(ARRAY['python','bash','git','linux','unit-testing']);

INSERT INTO experience_skills (experience_id, skill_id)
SELECT '11111111-1111-4111-8111-000000000017', id FROM skills
WHERE slug = ANY(ARRAY['javascript','neural-networks','genetic-algorithms','html5-canvas','html']);

-- Bullet tags: the precise signal retrieval scores on, so each line carries what it proves.
INSERT INTO bullet_skills (bullet_id, skill_id)
SELECT b.bullet_id::uuid, s.id
FROM (VALUES
    ('33333333-3333-4333-8333-000000010101', ARRAY['rust','blockchain']),
    ('33333333-3333-4333-8333-000000010102', ARRAY['rust','sandboxing','blockchain']),
    ('33333333-3333-4333-8333-000000010103', ARRAY['webassembly','rust','sandboxing']),
    ('33333333-3333-4333-8333-000000010104', ARRAY['solana','anchor','blockchain']),
    ('33333333-3333-4333-8333-000000010105', ARRAY['solana','blockchain','rust']),
    ('33333333-3333-4333-8333-000000010106', ARRAY['rust','linux']),
    ('33333333-3333-4333-8333-000000010107', ARRAY['solana','blockchain']),
    ('33333333-3333-4333-8333-000000010108', ARRAY['github-actions','ci-cd','unit-testing','rust']),

    ('33333333-3333-4333-8333-000000010201', ARRAY['python','faster-whisper','asyncio']),
    ('33333333-3333-4333-8333-000000010202', ARRAY['python','agentic-development']),
    ('33333333-3333-4333-8333-000000010203', ARRAY['git','python','sandboxing']),
    ('33333333-3333-4333-8333-000000010204', ARRAY['faiss','mem0','fastembed','onnx','python']),
    ('33333333-3333-4333-8333-000000010205', ARRAY['litellm','ollama','python']),
    ('33333333-3333-4333-8333-000000010206', ARRAY['pytest','unit-testing','python']),
    ('33333333-3333-4333-8333-000000010207', ARRAY['google-workspace-apis','oauth-2-0','tavily']),
    ('33333333-3333-4333-8333-000000010208', ARRAY['server-sent-events','html5-canvas','javascript']),

    ('33333333-3333-4333-8333-000000010301', ARRAY['express-js','next-js','react','railway','vercel']),
    ('33333333-3333-4333-8333-000000010302', ARRAY['google-gemini','agentic-development','google-calendar-api']),
    ('33333333-3333-4333-8333-000000010303', ARRAY['rest-apis','node-js']),
    ('33333333-3333-4333-8333-000000010304', ARRAY['google-gemini','agentic-development']),
    ('33333333-3333-4333-8333-000000010305', ARRAY['oauth-2-0','jwt-authentication','postgresql']),
    ('33333333-3333-4333-8333-000000010306', ARRAY['node-js','google-gemini']),
    ('33333333-3333-4333-8333-000000010307', ARRAY['gtfs','rest-apis','react']),

    ('33333333-3333-4333-8333-000000010401', ARRAY['typescript','google-gemini']),
    ('33333333-3333-4333-8333-000000010402', ARRAY['vitest','unit-testing','typescript']),
    ('33333333-3333-4333-8333-000000010403', ARRAY['typescript','express-js']),
    ('33333333-3333-4333-8333-000000010404', ARRAY['google-gemini','typescript']),
    ('33333333-3333-4333-8333-000000010405', ARRAY['twilio','rest-apis']),
    ('33333333-3333-4333-8333-000000010406', ARRAY['elevenlabs','rest-apis']),
    ('33333333-3333-4333-8333-000000010407', ARRAY['docker','google-cloud-run','vercel','next-js']),

    ('33333333-3333-4333-8333-000000010501', ARRAY['typescript','vs-code-extension-api']),
    ('33333333-3333-4333-8333-000000010502', ARRAY['google-gemini','vs-code-extension-api','typescript']),
    ('33333333-3333-4333-8333-000000010503', ARRAY['node-js','typescript']),
    ('33333333-3333-4333-8333-000000010504', ARRAY['vs-code-extension-api','typescript']),
    ('33333333-3333-4333-8333-000000010505', ARRAY['vs-code-extension-api','google-gemini']),

    ('33333333-3333-4333-8333-000000010601', ARRAY['python','git']),
    ('33333333-3333-4333-8333-000000010602', ARRAY['git','python']),
    ('33333333-3333-4333-8333-000000010603', ARRAY['git','python']),
    ('33333333-3333-4333-8333-000000010604', ARRAY['python']),
    ('33333333-3333-4333-8333-000000010605', ARRAY['git','python']),
    ('33333333-3333-4333-8333-000000010606', ARRAY['bash','python','linux']),
    ('33333333-3333-4333-8333-000000010607', ARRAY['unit-testing','python']),

    ('33333333-3333-4333-8333-000000010701', ARRAY['neural-networks','genetic-algorithms','javascript']),
    ('33333333-3333-4333-8333-000000010702', ARRAY['genetic-algorithms','javascript']),
    ('33333333-3333-4333-8333-000000010703', ARRAY['neural-networks','javascript']),
    ('33333333-3333-4333-8333-000000010704', ARRAY['javascript']),
    ('33333333-3333-4333-8333-000000010705', ARRAY['html5-canvas','javascript','neural-networks']),
    ('33333333-3333-4333-8333-000000010706', ARRAY['javascript'])
) AS b(bullet_id, slugs)
JOIN skills s ON s.slug = ANY(b.slugs);
