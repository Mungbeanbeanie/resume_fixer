-- The user's current resume, so the vault has real content to work with from the first run.
-- Text is stored plain: no LaTeX markup, no escaping. render/tex.rs owns escaping.

INSERT INTO profile (id, full_name, phone, email, links) VALUES (
    TRUE,
    'Michael Chung',
    '(571) 491-5056',
    'mychung007@gmail.com',
    '[{"label": "linkedin.com/in/michaelchung07", "url": "https://linkedin.com/in/michaelchung07"},
      {"label": "github.com/Mungbeanbeanie", "url": "https://github.com/Mungbeanbeanie"}]'
);

INSERT INTO skills (name, slug, category) VALUES
    ('Claude',              'claude',              'tool'),
    ('Agentic Development', 'agentic-development', 'concept'),
    ('Python',              'python',              'language'),
    ('Java',                'java',                'language'),
    ('SQL',                 'sql',                 'language'),
    ('Javascript',          'javascript',          'language'),
    ('TypeScript',          'typescript',          'language'),
    ('Swift',               'swift',               'language'),
    ('C',                   'c',                   'language'),
    ('HTML/CSS',            'html-css',            'language'),
    ('Linux',               'linux',               'platform'),
    ('Node.js',             'node-js',             'framework'),
    ('Express.js',          'express-js',          'framework'),
    ('Next.js',             'next-js',             'framework'),
    ('React',               'react',               'framework'),
    ('SwiftUI',             'swiftui',             'framework'),
    ('Firebase',            'firebase',            'cloud'),
    ('Supabase',            'supabase',            'cloud'),
    ('Git',                 'git',                 'tool'),
    ('Docker',              'docker',              'tool'),
    ('AWS (EC2, Lambda)',   'aws-ec2-lambda',      'cloud'),
    ('Nginx',               'nginx',               'tool'),
    ('Pandas',              'pandas',              'library'),
    ('NumPy',               'numpy',               'library'),
    ('Matplotlib',          'matplotlib',          'library'),
    ('PyTorch',             'pytorch',             'library'),
    ('PostgreSQL',          'postgresql',          'database'),
    ('InfluxDB',            'influxdb',            'database'),
    ('MongoDB',             'mongodb',             'database'),
    ('GitLab CI/CD',        'gitlab-ci-cd',        'tool'),
    ('REST APIs',           'rest-apis',           'concept'),
    ('JWT Authentication',  'jwt-authentication',  'concept'),
    ('Embedded C',          'embedded-c',          'concept'),
    ('DMA',                 'dma',                 'concept'),
    ('UART',                'uart',                'concept');

UPDATE skills SET aliases = '{postgres,psql,pg}'          WHERE slug = 'postgresql';
UPDATE skills SET aliases = '{node,nodejs}'               WHERE slug = 'node-js';
UPDATE skills SET aliases = '{express,expressjs}'         WHERE slug = 'express-js';
UPDATE skills SET aliases = '{next,nextjs}'               WHERE slug = 'next-js';
UPDATE skills SET aliases = '{aws,ec2,lambda}'            WHERE slug = 'aws-ec2-lambda';
UPDATE skills SET aliases = '{js,ecmascript}'             WHERE slug = 'javascript';
UPDATE skills SET aliases = '{ts}'                        WHERE slug = 'typescript';
UPDATE skills SET aliases = '{html,css}'                  WHERE slug = 'html-css';
UPDATE skills SET aliases = '{mongo,"mongodb atlas"}'     WHERE slug = 'mongodb';
UPDATE skills SET aliases = '{rest,"restful api","rest api"}' WHERE slug = 'rest-apis';
UPDATE skills SET aliases = '{jwt}'                       WHERE slug = 'jwt-authentication';
UPDATE skills SET aliases = '{"ci/cd",gitlab}'            WHERE slug = 'gitlab-ci-cd';
UPDATE skills SET aliases = '{"llm agents","ai agents",anthropic}' WHERE slug = 'claude';

-- ── Experiences ─────────────────────────────────────────────────────────
INSERT INTO experiences (id, kind, org_name, location, tech_line, display_order) VALUES
    ('11111111-1111-4111-8111-000000000001', 'education',  'University of Virginia',      'Charlottesville, VA', NULL, 0),
    ('11111111-1111-4111-8111-000000000002', 'work',       'Rajant Health',               'Malvern, PA',         NULL, 1),
    ('11111111-1111-4111-8111-000000000003', 'work',       'UVA School of Data Science',  'Charlottesville, VA', NULL, 2),
    ('11111111-1111-4111-8111-000000000004', 'work',       'Talent Skincare',             'Remote',              NULL, 3),
    ('11111111-1111-4111-8111-000000000005', 'project',    'Sniped -- Gamified Social Network (iOS)', NULL, 'PostgreSQL, Node.js, Express, Docker, AWS', 4),
    ('11111111-1111-4111-8111-000000000006', 'project',    'Microsoft Garage -- Network Vulnerability Tool', NULL, 'JavaScript, Firebase, Node.js', 5),
    ('11111111-1111-4111-8111-000000000007', 'project',    'PoliDex -- Political Alignment Engine', NULL, 'Python, Java, Next.js, React, TypeScript, MongoDB', 6),
    ('11111111-1111-4111-8111-000000000008', 'certification', 'CompTIA Security+',                NULL, NULL, 7),
    ('11111111-1111-4111-8111-000000000009', 'certification', 'AWS Certified Cloud Practitioner', NULL, NULL, 8),
    ('11111111-1111-4111-8111-00000000000a', 'certification', 'Microsoft Azure Fundamentals',     NULL, NULL, 9),
    ('11111111-1111-4111-8111-00000000000b', 'certification', 'ITS: Artificial Intelligence',     NULL, NULL, 10);

INSERT INTO roles (id, experience_id, title, start_date, end_date, display_order) VALUES
    ('22222222-2222-4222-8222-000000000001', '11111111-1111-4111-8111-000000000001', 'Computer Science and Data Science', '2025-08-01', '2029-05-01', 0),
    ('22222222-2222-4222-8222-000000000002', '11111111-1111-4111-8111-000000000002', 'Software Engineering Intern',       '2026-06-01', '2026-08-01', 0),
    ('22222222-2222-4222-8222-000000000003', '11111111-1111-4111-8111-000000000002', 'Computer Engineering Intern',       '2025-07-01', '2025-08-01', 1),
    ('22222222-2222-4222-8222-000000000004', '11111111-1111-4111-8111-000000000003', 'Undergraduate Research Assistant',  '2026-01-01', '2026-05-01', 0),
    ('22222222-2222-4222-8222-000000000005', '11111111-1111-4111-8111-000000000004', 'Digital Marketing & Analytics Intern', '2025-08-01', '2025-12-01', 0),
    ('22222222-2222-4222-8222-000000000006', '11111111-1111-4111-8111-000000000005', 'Sniped', '2025-12-01', '2026-03-01', 0),
    ('22222222-2222-4222-8222-000000000007', '11111111-1111-4111-8111-000000000006', 'Microsoft Garage', '2025-01-01', '2025-03-01', 0),
    ('22222222-2222-4222-8222-000000000008', '11111111-1111-4111-8111-000000000007', 'PoliDex', '2026-04-01', '2026-04-30', 0);

INSERT INTO bullets (id, role_id, text, display_order) VALUES
    ('33333333-3333-4333-8333-000000000001', '22222222-2222-4222-8222-000000000001',
     'Relevant coursework: Data Structures & Algorithms, Foundations of Data Science, Linear Algebra, Discrete Math, Software Development Essentials, Computer Systems and Organization', 0),

    ('33333333-3333-4333-8333-000000000002', '22222222-2222-4222-8222-000000000002',
     'Designed semantic embedding based algorithms to ensure deduplication, provenance, incremental integration, and human-in-the-loop processes in agentic database consolidation, influencing production agentic architecture design', 0),
    ('33333333-3333-4333-8333-000000000003', '22222222-2222-4222-8222-000000000002',
     'Developed Backend-For-Frontend architecture in Python, linking hardware sensor data with a third party proprietary API, allowing modular and secure data transfer through PostgreSQL and InfluxDB', 1),

    ('33333333-3333-4333-8333-000000000004', '22222222-2222-4222-8222-000000000003',
     'Delivered proof-of-concept validating next-generation hardware platform (NXP MIMXRT595), directly influencing strategic decision to deprecate legacy hardware', 0),
    ('33333333-3333-4333-8333-000000000005', '22222222-2222-4222-8222-000000000003',
     'Implemented lock-free UART data pipeline using ring buffers sustaining 460 kbps continuous transfer for real-time PPG sensor data ingestion; designed 2D renderer with DMA-powered updates achieving under 15ms frame latency', 1),

    ('33333333-3333-4333-8333-000000000006', '22222222-2222-4222-8222-000000000004',
     'Engineered data pipelines in Python (Pandas, NumPy) to analyze satellite launch datasets across LEO/MEO/GEO regimes, presented paper at IEEE SIEDS 26: Decision Support for Resilient Foundation Model Scaling in Orbital Computing Systems', 0),
    ('33333333-3333-4333-8333-000000000007', '22222222-2222-4222-8222-000000000004',
     'Identified that communications satellites account for approximately 60% of recent payloads with technology satellites at 3-5%, informing assumption-based projection models (exponential, logistic, quadratic) for computation-capable orbital infrastructure', 1),

    ('33333333-3333-4333-8333-000000000008', '22222222-2222-4222-8222-000000000005',
     'Built serverless Python function (AWS Lambda) integrated with Monday.com API to automate Amazon Ads campaign data extraction, eliminating manual reporting workflows', 0),

    ('33333333-3333-4333-8333-000000000009', '22222222-2222-4222-8222-000000000006',
     'Designed and deployed RESTful API (20+ endpoints) on AWS EC2 with Docker, Nginx reverse proxy, and automated CI/CD via GitLab pipelines', 0),
    ('33333333-3333-4333-8333-00000000000a', '22222222-2222-4222-8222-000000000006',
     'Implemented JWT authentication with Row-Level Security across 10+ PostgreSQL tables, including dual-client authorization pattern for service-role vs. public access', 1),
    ('33333333-3333-4333-8333-00000000000b', '22222222-2222-4222-8222-000000000006',
     'Approved on Apple TestFlight and tested by 20+ beta users', 2),

    ('33333333-3333-4333-8333-00000000000c', '22222222-2222-4222-8222-000000000007',
     'Developed front-end interface and Firebase data pipeline for network vulnerability detection tool; integrated ChatGPT API for vulnerability analysis, optimizing performance and cost for constrained environments', 0),

    ('33333333-3333-4333-8333-00000000000d', '22222222-2222-4222-8222-000000000008',
     'Built and shipped political alignment engine in 24 hours; scores politicians across a 20-dimensional policy vector space using a custom variance-weighted cosine similarity formula, a Java/Python/Next.js architecture, and a k-d tree index for sublinear 20D nearest-neighbor search; won Best Use of MongoDB Atlas at Hackabull VII', 0);

-- ── Tags ────────────────────────────────────────────────────────────────
INSERT INTO experience_skills (experience_id, skill_id)
SELECT e.id, s.id FROM (VALUES
    ('11111111-1111-4111-8111-000000000002'::uuid, 'python'),
    ('11111111-1111-4111-8111-000000000002'::uuid, 'postgresql'),
    ('11111111-1111-4111-8111-000000000002'::uuid, 'influxdb'),
    ('11111111-1111-4111-8111-000000000002'::uuid, 'claude'),
    ('11111111-1111-4111-8111-000000000002'::uuid, 'agentic-development'),
    ('11111111-1111-4111-8111-000000000002'::uuid, 'embedded-c'),
    ('11111111-1111-4111-8111-000000000003'::uuid, 'python'),
    ('11111111-1111-4111-8111-000000000003'::uuid, 'pandas'),
    ('11111111-1111-4111-8111-000000000003'::uuid, 'numpy'),
    ('11111111-1111-4111-8111-000000000004'::uuid, 'python'),
    ('11111111-1111-4111-8111-000000000004'::uuid, 'aws-ec2-lambda'),
    ('11111111-1111-4111-8111-000000000005'::uuid, 'postgresql'),
    ('11111111-1111-4111-8111-000000000005'::uuid, 'node-js'),
    ('11111111-1111-4111-8111-000000000005'::uuid, 'express-js'),
    ('11111111-1111-4111-8111-000000000005'::uuid, 'docker'),
    ('11111111-1111-4111-8111-000000000005'::uuid, 'aws-ec2-lambda'),
    ('11111111-1111-4111-8111-000000000005'::uuid, 'swift'),
    ('11111111-1111-4111-8111-000000000005'::uuid, 'swiftui'),
    ('11111111-1111-4111-8111-000000000006'::uuid, 'javascript'),
    ('11111111-1111-4111-8111-000000000006'::uuid, 'firebase'),
    ('11111111-1111-4111-8111-000000000006'::uuid, 'node-js'),
    ('11111111-1111-4111-8111-000000000007'::uuid, 'python'),
    ('11111111-1111-4111-8111-000000000007'::uuid, 'java'),
    ('11111111-1111-4111-8111-000000000007'::uuid, 'next-js'),
    ('11111111-1111-4111-8111-000000000007'::uuid, 'react'),
    ('11111111-1111-4111-8111-000000000007'::uuid, 'typescript'),
    ('11111111-1111-4111-8111-000000000007'::uuid, 'mongodb')
) AS t(id, slug)
JOIN experiences e ON e.id = t.id
JOIN skills s ON s.slug = t.slug;

INSERT INTO bullet_skills (bullet_id, skill_id, weight)
SELECT b.id, s.id, t.weight FROM (VALUES
    ('33333333-3333-4333-8333-000000000002'::uuid, 'agentic-development', 1.0),
    ('33333333-3333-4333-8333-000000000002'::uuid, 'claude',              0.5),
    ('33333333-3333-4333-8333-000000000003'::uuid, 'python',              1.0),
    ('33333333-3333-4333-8333-000000000003'::uuid, 'postgresql',          1.0),
    ('33333333-3333-4333-8333-000000000003'::uuid, 'influxdb',            1.0),
    ('33333333-3333-4333-8333-000000000003'::uuid, 'rest-apis',           0.5),
    ('33333333-3333-4333-8333-000000000004'::uuid, 'embedded-c',          1.0),
    ('33333333-3333-4333-8333-000000000005'::uuid, 'embedded-c',          1.0),
    ('33333333-3333-4333-8333-000000000005'::uuid, 'uart',                1.0),
    ('33333333-3333-4333-8333-000000000005'::uuid, 'dma',                 1.0),
    ('33333333-3333-4333-8333-000000000006'::uuid, 'python',              1.0),
    ('33333333-3333-4333-8333-000000000006'::uuid, 'pandas',              1.0),
    ('33333333-3333-4333-8333-000000000006'::uuid, 'numpy',               1.0),
    ('33333333-3333-4333-8333-000000000007'::uuid, 'python',              0.5),
    ('33333333-3333-4333-8333-000000000008'::uuid, 'python',              1.0),
    ('33333333-3333-4333-8333-000000000008'::uuid, 'aws-ec2-lambda',      1.0),
    ('33333333-3333-4333-8333-000000000009'::uuid, 'rest-apis',           1.0),
    ('33333333-3333-4333-8333-000000000009'::uuid, 'aws-ec2-lambda',      1.0),
    ('33333333-3333-4333-8333-000000000009'::uuid, 'docker',              1.0),
    ('33333333-3333-4333-8333-000000000009'::uuid, 'nginx',               1.0),
    ('33333333-3333-4333-8333-000000000009'::uuid, 'gitlab-ci-cd',        1.0),
    ('33333333-3333-4333-8333-00000000000a'::uuid, 'jwt-authentication',  1.0),
    ('33333333-3333-4333-8333-00000000000a'::uuid, 'postgresql',          1.0),
    ('33333333-3333-4333-8333-00000000000b'::uuid, 'swift',               0.5),
    ('33333333-3333-4333-8333-00000000000c'::uuid, 'javascript',          1.0),
    ('33333333-3333-4333-8333-00000000000c'::uuid, 'firebase',            1.0),
    ('33333333-3333-4333-8333-00000000000c'::uuid, 'node-js',             0.5),
    ('33333333-3333-4333-8333-00000000000d'::uuid, 'java',                1.0),
    ('33333333-3333-4333-8333-00000000000d'::uuid, 'python',              1.0),
    ('33333333-3333-4333-8333-00000000000d'::uuid, 'next-js',             1.0),
    ('33333333-3333-4333-8333-00000000000d'::uuid, 'mongodb',             1.0)
) AS t(id, slug, weight)
JOIN bullets b ON b.id = t.id
JOIN skills s ON s.slug = t.slug;
