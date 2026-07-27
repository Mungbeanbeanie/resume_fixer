// The only place invoke() is called.

import { invoke } from "@tauri-apps/api/core";
import type {
  AppError,
  ApplicationDetail,
  ApplicationPatch,
  ApplicationStatus,
  ApplicationSummary,
  Bullet,
  BulletInput,
  Experience,
  ExperienceDetail,
  ExperienceInput,
  GenerationResult,
  HealthReport,
  IngestResult,
  Profile,
  Role,
  RoleInput,
  Skill,
  StatusStats,
  Suggestion,
} from "./types";

/** True for anything the backend sent as a typed AppError. */
export function isAppError(e: unknown): e is AppError {
  return typeof e === "object" && e !== null && "kind" in e && "message" in e;
}

export function errorMessage(e: unknown): string {
  if (isAppError(e)) return e.message;
  if (e instanceof Error) return e.message;
  return String(e);
}

export const health = {
  check: () => invoke<HealthReport>("health_check"),
};

export const vault = {
  listExperiences: () => invoke<ExperienceDetail[]>("vault_list_experiences"),
  upsertExperience: (input: ExperienceInput, skills?: string[]) =>
    invoke<Experience>("vault_upsert_experience", { input, skills }),
  deleteExperience: (id: string) => invoke<void>("vault_delete_experience", { id }),
  upsertRole: (input: RoleInput) => invoke<Role>("vault_upsert_role", { input }),
  deleteRole: (id: string) => invoke<void>("vault_delete_role", { id }),
  upsertBullet: (input: BulletInput) => invoke<Bullet>("vault_upsert_bullet", { input }),
  deleteBullet: (id: string) => invoke<void>("vault_delete_bullet", { id }),
  setBulletSkills: (bulletId: string, skillNames: string[]) =>
    invoke<void>("vault_set_bullet_skills", { bulletId, skillNames }),
  suggestBulletImprovements: (bulletId: string) =>
    invoke<Suggestion[]>("vault_suggest_bullet_improvements", { bulletId }),
  acceptVariant: (variantId: string) => invoke<Bullet>("vault_accept_variant", { variantId }),
  listSkills: () => invoke<Skill[]>("vault_list_skills"),
  getProfile: () => invoke<Profile | null>("vault_get_profile"),
  upsertProfile: (profile: Profile) => invoke<Profile>("vault_upsert_profile", { profile }),
};

export const generate = {
  ingestJob: (url: string) => invoke<IngestResult>("generate_ingest_job", { url }),
  fromText: (args: {
    jobText: string;
    url?: string | null;
    fetched?: boolean;
    feedback?: string | null;
  }) => invoke<GenerationResult>("generate_from_text", args),
  discard: (draftId: string) => invoke<void>("generate_discard", { draftId }),
  commit: (draftId: string, applied: boolean) =>
    invoke<string>("generate_commit", { draftId, applied }),
  exportPdf: (resumeId: string, destPath: string) =>
    invoke<string>("generate_export_pdf", { resumeId, destPath }),
  exportDraft: (draftId: string, filename: string) =>
    invoke<string>("generate_export_draft", { draftId, filename }),
};

export const library = {
  listApplications: (filter?: ApplicationStatus | null) =>
    invoke<ApplicationSummary[]>("library_list_applications", { filter: filter ?? null }),
  getApplication: (id: string) => invoke<ApplicationDetail>("library_get_application", { id }),
  setStatus: (id: string, status: ApplicationStatus) =>
    invoke<void>("library_set_status", { id, status }),
  updateApplication: (id: string, patch: ApplicationPatch) =>
    invoke<void>("library_update_application", { id, patch }),
  deleteApplication: (id: string) => invoke<void>("library_delete_application", { id }),
  stats: () => invoke<StatusStats>("library_stats"),
};
