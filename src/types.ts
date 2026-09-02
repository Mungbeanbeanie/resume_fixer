// Mirrors src-tauri/src/domain.rs. Change one, change both in the same commit.

export type ExperienceKind =
  | "work"
  | "project"
  | "education"
  | "certification"
  | "activity";
/** In the order an application moves through them. */
export type ApplicationStatus =
  | "saved"
  | "applied"
  | "oa_received"
  | "oa_completed"
  | "interview_1"
  | "interview_2"
  | "interview_3"
  | "offer"
  | "rejected"
  | "withdrawn";
export type JobSource = "fetched" | "pasted";

export interface Link {
  label: string;
  url: string;
}

export interface Profile {
  full_name: string;
  phone: string | null;
  email: string | null;
  links: Link[];
  interests: string | null;
}

export interface Experience {
  id: string;
  kind: ExperienceKind;
  org_name: string;
  location: string | null;
  /** Printed by the Projects section in place of the date range; see `link_text`. */
  url: string | null;
  /** What `url` reads as on the page. Blank falls back to the shortened URL. */
  link_text: string | null;
  tech_line: string | null;
  display_order: number;
  is_active: boolean;
  is_pinned: boolean;
}

export interface Role {
  id: string;
  experience_id: string;
  title: string;
  location: string | null;
  /** Absent when the entry has no calendar range to print; see `date_override`. */
  start_date: string | null;
  end_date: string | null;
  date_override: string | null;
  /** Free text, printed after the degree on education entries. */
  gpa: string | null;
  display_order: number;
  is_active: boolean;
}

export interface Bullet {
  id: string;
  role_id: string;
  text: string;
  display_order: number;
  is_active: boolean;
}

export interface Skill {
  id: string;
  name: string;
  slug: string;
  category: string | null;
  aliases: string[];
  always_list: boolean;
}

export interface BulletVariant {
  id: string;
  bullet_id: string;
  text: string;
  origin: string;
  approved_at: string | null;
}

export type BulletDetail = Bullet & {
  skills: Skill[];
  variants: BulletVariant[];
};

export type RoleDetail = Role & { bullets: BulletDetail[] };

export type ExperienceDetail = Experience & {
  skills: Skill[];
  roles: RoleDetail[];
};

export interface ExperienceInput {
  id: string | null;
  kind: ExperienceKind;
  org_name: string;
  location: string | null;
  url: string | null;
  link_text: string | null;
  tech_line: string | null;
  display_order: number;
  is_active: boolean;
  is_pinned: boolean;
}

export interface RoleInput {
  id: string | null;
  experience_id: string;
  title: string;
  location: string | null;
  start_date: string | null;
  end_date: string | null;
  date_override: string | null;
  gpa: string | null;
  display_order: number;
  is_active: boolean;
}

export interface BulletInput {
  id: string | null;
  role_id: string;
  text: string;
  display_order: number;
  is_active: boolean;
}

export interface Suggestion {
  variant_id: string;
  text: string;
}

export interface BulletEdit {
  bullet_id: string;
  text: string;
  keep: boolean;
}

export interface Application {
  id: string;
  url: string | null;
  company: string | null;
  role_title: string | null;
  job_text: string;
  job_source: JobSource;
  parsed: unknown;
  status: ApplicationStatus;
  applied_at: string | null;
  notes: string | null;
  created_at: string;
}

export interface ApplicationSummary {
  id: string;
  company: string | null;
  role_title: string | null;
  url: string | null;
  status: ApplicationStatus;
  created_at: string;
  applied_at: string | null;
  resume_id: string | null;
  pdf_path: string | null;
}

export interface Resume {
  id: string;
  application_id: string;
  tex_source: string;
  pdf_path: string | null;
  model: string;
  prompt_version: string;
  feedback: string | null;
  page_count: number | null;
  is_current: boolean;
  created_at: string;
}

export interface StatusChange {
  status: ApplicationStatus;
  changed_at: string;
}

export type ApplicationDetail = Application & {
  resume: Resume | null;
  history: StatusChange[];
};

export interface ApplicationPatch {
  company: string | null;
  role_title: string | null;
  notes: string | null;
}

/** Counts behind the Library graph. A status nobody is sitting on is absent, not zero. */
export type StatusStats = Partial<Record<ApplicationStatus, number>>;

export interface IngestResult {
  text: string;
  source: JobSource;
  needs_paste: boolean;
  /** Why the fetch or extraction gave up, when it did. */
  reason: string | null;
}

export interface UsedBullet {
  bullet_id: string;
  source_text: string;
  rendered_text: string;
  was_reworded: boolean;
  org_name: string;
}

export interface RejectedRewrite {
  bullet_id: string;
  attempted: string;
  reason: string;
}

export interface GenerationResult {
  draft_id: string;
  pdf_path: string;
  page_count: number;
  company: string | null;
  role_title: string | null;
  used_bullets: UsedBullet[];
  rejected: RejectedRewrite[];
  dropped_for_fit: number;
  retired: string[];
}

export interface Template {
  id: string;
  name: string;
  source: string;
  is_builtin: boolean;
  is_active: boolean;
}

export interface BaseResume {
  id: string;
  name: string;
  template_name: string;
  pdf_path: string | null;
  page_count: number | null;
  created_at: string;
}

export interface BasePreview {
  template_name: string;
  pdf_path: string;
  page_count: number;
  bullet_count: number;
  used_bullets: UsedBullet[];
  retired: string[];
}

export interface HealthReport {
  postgres: boolean;
  ollama: boolean;
  model_present: boolean;
  tectonic: boolean;
  model: string;
}

/** AppError as it crosses the IPC boundary. */
export interface AppError {
  kind:
    | "config"
    | "db"
    | "not_found"
    | "invalid"
    | "fetch"
    | "extract_failed"
    | "llm"
    | "render"
    | "io";
  message: string;
  detail?: string;
}
