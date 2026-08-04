//! The visual-parity check for M3: the generated document must carry the same content as
//! the user's reference resume and still come out one page.
//!
//! Needs Tectonic on PATH, so it is ignored by default:
//!     cargo test --test render_reference -- --ignored --nocapture

use resume_fixer_lib::domain::{Link, Profile};
use resume_fixer_lib::render::tectonic;
use resume_fixer_lib::render::templates::{DEFAULT_TEMPLATE, SIMPLIFY_TEMPLATE};
use resume_fixer_lib::render::tex::{ExperienceBlock, ProjectBlock, RenderInput, RoleBlock};

const REFERENCE: &str = include_str!("../../templates/reference/base_resume.tex");

fn role(title: &str, dates: &str, bullets: &[&str]) -> RoleBlock {
    RoleBlock {
        title: title.into(),
        dates: dates.into(),
        bullets: bullets.iter().map(|b| b.to_string()).collect(),
    }
}

/// The reference resume, expressed as the plan would hand it to the renderer.
fn reference_input() -> RenderInput {
    RenderInput {
        profile: Some(Profile {
            full_name: "Michael Chung".into(),
            phone: Some("(571) 491-5056".into()),
            email: Some("mychung007@gmail.com".into()),
            links: vec![
                Link {
                    label: "linkedin.com/in/michaelchung07".into(),
                    url: "https://linkedin.com/in/michaelchung07".into(),
                },
                Link {
                    label: "github.com/Mungbeanbeanie".into(),
                    url: "https://github.com/Mungbeanbeanie".into(),
                },
            ],
        }),
        education: vec![ExperienceBlock {
            org: "University of Virginia".into(),
            location: "Charlottesville, VA".into(),
            roles: vec![role(
                "Computer Science and Data Science",
                "Aug. 2025 -- May 2029",
                &["Relevant coursework: Data Structures & Algorithms, Foundations of Data Science, Linear Algebra, Discrete Math, Software Development Essentials, Computer Systems and Organization"],
            )],
        }],
        skills_line: "Claude, Agentic Development, Python, Java, SQL, Javascript, Swift, C, HTML/CSS, Linux, Node.js, Express.js, Next.js, SwiftUI, Firebase, Supabase, Git, Docker, AWS (EC2, Lambda), Nginx, Pandas, NumPy, Matplotlib, PyTorch".into(),
        experience: vec![
            ExperienceBlock {
                org: "Rajant Health".into(),
                location: "Malvern, PA".into(),
                roles: vec![
                    role("Software Engineering Intern", "June 2026 -- Aug. 2026", &[
                        "Designed semantic embedding based algorithms to ensure deduplication, provenance, incremental integration, and human-in-the-loop processes in agentic database consolidation, influencing production agentic architecture design",
                        "Developed Backend-For-Frontend architecture in Python, linking hardware sensor data with a third party proprietary API, allowing modular and secure data transfer through PostgreSQL and InfluxDB",
                    ]),
                    role("Computer Engineering Intern", "July 2025 -- Aug. 2025", &[
                        "Delivered proof-of-concept validating next-generation hardware platform (NXP MIMXRT595), directly influencing strategic decision to deprecate legacy hardware",
                        "Implemented lock-free UART data pipeline using ring buffers sustaining 460 kbps continuous transfer for real-time PPG sensor data ingestion; designed 2D renderer with DMA-powered updates achieving under 15ms frame latency",
                    ]),
                ],
            },
            ExperienceBlock {
                org: "UVA School of Data Science".into(),
                location: "Charlottesville, VA".into(),
                roles: vec![role("Undergraduate Research Assistant", "Jan. 2026 -- May 2026", &[
                    "Engineered data pipelines in Python (Pandas, NumPy) to analyze satellite launch datasets across LEO/MEO/GEO regimes, presented paper at IEEE SIEDS 26: Decision Support for Resilient Foundation Model Scaling in Orbital Computing Systems",
                    "Identified that communications satellites account for approximately 60% of recent payloads with technology satellites at 3-5%, informing assumption-based projection models (exponential, logistic, quadratic) for computation-capable orbital infrastructure",
                ])],
            },
            ExperienceBlock {
                org: "Talent Skincare".into(),
                location: "Remote".into(),
                roles: vec![role("Digital Marketing & Analytics Intern", "Aug. 2025 -- Dec. 2025", &[
                    "Built serverless Python function (AWS Lambda) integrated with Monday.com API to automate Amazon Ads campaign data extraction, eliminating manual reporting workflows",
                ])],
            },
        ],
        projects: vec![
            ProjectBlock {
                name: "Sniped -- Gamified Social Network (iOS)".into(),
                tech: "PostgreSQL, Node.js, Express, Docker, AWS".into(),
                dates: "Dec. 2025 -- Mar. 2026".into(),
                bullets: vec![
                    "Designed and deployed RESTful API (20+ endpoints) on AWS EC2 with Docker, Nginx reverse proxy, and automated CI/CD via GitLab pipelines".into(),
                    "Implemented JWT authentication with Row-Level Security across 10+ PostgreSQL tables, including dual-client authorization pattern for service-role vs. public access".into(),
                    "Approved on Apple TestFlight and tested by 20+ beta users".into(),
                ],
            },
            ProjectBlock {
                name: "Microsoft Garage -- Network Vulnerability Tool".into(),
                tech: "JavaScript, Firebase, Node.js".into(),
                dates: "Jan. 2025 -- Mar. 2025".into(),
                bullets: vec![
                    "Developed front-end interface and Firebase data pipeline for network vulnerability detection tool; integrated ChatGPT API for vulnerability analysis, optimizing performance and cost for constrained environments".into(),
                ],
            },
            ProjectBlock {
                name: "PoliDex -- Political Alignment Engine".into(),
                tech: "Python, Java, Next.js, React, TypeScript, MongoDB".into(),
                dates: "Apr. 2026".into(),
                bullets: vec![
                    "Built and shipped political alignment engine in 24 hours; scores politicians across a 20-dimensional policy vector space using a custom variance-weighted cosine similarity formula, a Java/Python/Next.js architecture, and a k-d tree index for sublinear 20D nearest-neighbor search; won Best Use of MongoDB Atlas at Hackabull VII".into(),
                ],
            },
        ],
        activities: vec![],
        certifications: vec![
            "CompTIA Security+".into(),
            "AWS Certified Cloud Practitioner".into(),
            "Microsoft Azure Fundamentals".into(),
            "ITS: Artificial Intelligence".into(),
        ],
    }
}

/// The second built-in has to survive a real compile, not just a Tera render — a template
/// that only fails inside LaTeX would break the Base tab and every later generation.
#[tokio::test]
#[ignore = "needs tectonic on PATH"]
async fn the_simplify_template_compiles_with_an_activities_section() {
    let mut input = reference_input();
    input.activities = vec![ExperienceBlock {
        org: "Intramural Ice Hockey".into(),
        location: "Charlottesville, VA".into(),
        roles: vec![role("Team Captain", "Jan. 2026 -- Present", &[])],
    }];
    let generated =
        resume_fixer_lib::render::tex::render(SIMPLIFY_TEMPLATE, &input).expect("template renders");
    assert!(generated.contains("\\section{Activities \\& Leadership}"));

    let dir = std::env::temp_dir().join("resume-fixer-simplify");
    let out = tectonic::compile("tectonic", &generated, &dir)
        .await
        .expect("the Simplify document compiles");
    println!(
        "simplify: {} page(s) at {}",
        out.page_count,
        out.pdf_path.display()
    );
}

#[tokio::test]
#[ignore = "needs tectonic on PATH"]
async fn the_generated_resume_matches_the_reference_and_fits_one_page() {
    let dir = std::env::temp_dir().join("resume-fixer-parity");
    let generated = resume_fixer_lib::render::tex::render(DEFAULT_TEMPLATE, &reference_input())
        .expect("template renders");

    // The preamble is copied from the reference; every line of it must still be there.
    // The template only adds `\ifdefined` guards around the two pdfTeX-only lines.
    let reference_preamble = REFERENCE.split("\\begin{document}").next().unwrap();
    let generated_preamble = generated.split("\\begin{document}").next().unwrap();
    for line in reference_preamble
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
    {
        assert!(
            generated_preamble.contains(line),
            "the generated preamble dropped: {line}"
        );
    }

    // Every bullet the reference prints must survive into the generated document.
    for item in REFERENCE.match_indices("\\resumeItem{").map(|(i, _)| i) {
        let body: String = REFERENCE[item + 12..].chars().take(40).collect();
        let probe = body
            .split(['\\', '{', '}', '&', '%'])
            .next()
            .unwrap()
            .trim();
        if probe.len() > 15 {
            assert!(
                generated.contains(probe),
                "the generated document dropped: {probe}"
            );
        }
    }

    let out = tectonic::compile("tectonic", &generated, &dir)
        .await
        .expect("generated document compiles");
    println!(
        "generated: {} page(s) at {}",
        out.page_count,
        out.pdf_path.display()
    );
    assert_eq!(
        out.page_count, 1,
        "the reference content must still fit one page"
    );
}
