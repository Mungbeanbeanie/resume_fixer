import { useCallback, useEffect, useState } from "react";
import { errorMessage, vault } from "../../ipc";
import type { ExperienceDetail, ExperienceKind, Profile, RoleDetail } from "../../types";
import BulletStandard from "./BulletStandard";
import ExperienceCard from "./ExperienceCard";
import SkillsCard from "./SkillsCard";
import { useVault, VaultCtx } from "./vaultContext";
import { X } from "../../icons";

// The order the vault reads in. Header Info is the profile card and has no kind.
const SECTIONS: [ExperienceKind, string][] = [
  ["education", "School"],
  ["work", "Work Experience"],
  ["project", "Projects"],
  ["activity", "Activities / Leadership"],
  ["certification", "Certifications"],
];

// The profile is edited from two cards — this one and Interests — so its draft lives in
// VaultTab. Two components each holding a copy would let a save from one write back the
// other's stale fields.
type ProfileProps = {
  draft: Profile;
  setDraft: (p: Profile) => void;
  save: (p: Profile) => void;
};

function ProfileCard({ draft, setDraft, save }: ProfileProps) {
  return (
    <details className="experience">
      <summary>
        <strong>{draft.full_name || "Your name"}</strong>
        <span className="muted" style={{ fontSize: 13 }}>
          {draft.email ?? "no email"} · {draft.links.length} link
          {draft.links.length === 1 ? "" : "s"}
        </span>
      </summary>
      <div className="body">
        <div className="grid-4">
          <div>
            <span className="field-label">Full name</span>
            <input
              value={draft.full_name}
              onChange={(e) => setDraft({ ...draft, full_name: e.target.value })}
              onBlur={() => save(draft)}
            />
          </div>
          <div>
            <span className="field-label">Phone</span>
            <input
              value={draft.phone ?? ""}
              onChange={(e) => setDraft({ ...draft, phone: e.target.value || null })}
              onBlur={() => save(draft)}
            />
          </div>
          <div>
            <span className="field-label">Email</span>
            <input
              value={draft.email ?? ""}
              onChange={(e) => setDraft({ ...draft, email: e.target.value || null })}
              onBlur={() => save(draft)}
            />
          </div>
        </div>
        <div className="col" style={{ gap: 6 }}>
          <span className="field-label">Links printed in the header</span>
          {draft.links.map((l, i) => (
            <div className="link-row" key={i}>
              <input
                placeholder="label"
                value={l.label}
                onChange={(e) => {
                  const links = [...draft.links];
                  links[i] = { ...l, label: e.target.value };
                  setDraft({ ...draft, links });
                }}
                onBlur={() => save(draft)}
              />
              <input
                placeholder="https://…"
                value={l.url}
                onChange={(e) => {
                  const links = [...draft.links];
                  links[i] = { ...l, url: e.target.value };
                  setDraft({ ...draft, links });
                }}
                onBlur={() => save(draft)}
              />
              <button
                className="quiet subtle"
                aria-label={`Remove ${l.label || "link"}`}
                onClick={() => save({ ...draft, links: draft.links.filter((_, j) => j !== i) })}
              >
                <X size={15} />
              </button>
            </div>
          ))}
          <div>
            <button
              className="quiet"
              onClick={() =>
                setDraft({ ...draft, links: [...draft.links, { label: "", url: "" }] })
              }
            >
              + link
            </button>
          </div>
        </div>
      </div>
    </details>
  );
}

// GPA is stored on the degree row, not on the profile, so one school can carry two of them —
// but Header Info is where it is entered, because that is where the user looks for it. It
// prints after the degree title on every layout.
function GpaCard({ roles }: { roles: RoleDetail[] }) {
  const { reload, report } = useVault();
  if (roles.length === 0) {
    return <div className="card muted">Add a School entry below to record a GPA.</div>;
  }
  return (
    <div className="card">
      <div className="grid-4">
        {roles.map((r) => (
          <div key={r.id}>
            <span className="field-label">
              GPA{roles.length > 1 ? ` — ${r.title}` : ""}
            </span>
            <input
              placeholder="e.g. 3.87/4.00"
              defaultValue={r.gpa ?? ""}
              onBlur={async (e) => {
                const gpa = e.target.value.trim() || null;
                if (gpa === r.gpa) return;
                try {
                  await vault.upsertRole({ ...r, gpa });
                  await reload();
                } catch (err) {
                  report(err);
                }
              }}
            />
          </div>
        ))}
      </div>
    </div>
  );
}

// One comma-separated line, printed only by templates that name `interests` — Simplify does,
// Jake's does not.
function InterestsCard({ draft, setDraft, save }: ProfileProps) {
  return (
    <div className="card">
      <span className="field-label">Comma separated — the Simplify layout prints these</span>
      <input
        placeholder="Ice hockey, chess, orbital mechanics"
        value={draft.interests ?? ""}
        onChange={(e) => setDraft({ ...draft, interests: e.target.value || null })}
        onBlur={() => save(draft)}
      />
    </div>
  );
}

export default function VaultTab() {
  const [experiences, setExperiences] = useState<ExperienceDetail[]>([]);
  const [profile, setProfile] = useState<Profile | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(true);

  const reload = useCallback(async () => {
    try {
      const [list, p] = await Promise.all([vault.listExperiences(), vault.getProfile()]);
      setExperiences(list);
      setProfile(p ?? { full_name: "", phone: null, email: null, links: [], interests: null });
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const report = useCallback((e: unknown) => setError(errorMessage(e)), []);

  async function saveProfile(next: Profile) {
    setProfile(next);
    try {
      setProfile(await vault.upsertProfile(next));
      setError(null);
    } catch (e) {
      report(e);
    }
  }

  // Commits the profile draft and re-reads the vault. Every other field already saves on
  // blur, so this is a flush plus a receipt: the counts come back from the database rather
  // than from the state on screen, so an edit that failed to persist shows as a number that
  // does not match what is displayed.
  async function saveAll() {
    setSaving(true);
    try {
      if (profile) setProfile(await vault.upsertProfile(profile));
      const [list, skills] = await Promise.all([vault.listExperiences(), vault.listSkills()]);
      setExperiences(list);
      const entries = list.filter((e) => e.is_active).length;
      const listed = skills.filter((s) => s.always_list).length;
      setNote(
        `Saved. Generation reads ${entries} active ${entries === 1 ? "entry" : "entries"} ` +
          `and prints ${listed} of ${skills.length} skills.`,
      );
      setError(null);
    } catch (e) {
      report(e);
    } finally {
      setSaving(false);
    }
  }

  async function addExperience(kind: ExperienceKind) {
    try {
      await vault.upsertExperience({
        id: null,
        kind,
        org_name: "New entry",
        location: null,
        url: null,
        link_text: null,
        tech_line: null,
        display_order: experiences.length,
        is_active: true,
        is_pinned: false,
      });
      await reload();
    } catch (e) {
      report(e);
    }
  }

  return (
    <VaultCtx.Provider value={{ reload, report }}>
      <div className="col">
        <div className="row" style={{ gap: "var(--space-4)" }}>
          <span className="muted" style={{ fontSize: 13, maxWidth: 520 }}>
            Fields save when they lose focus. Save to commit everything and see what the
            Generate and Base tabs will read.
          </span>
          <span style={{ flex: 1 }} />
          <button className="primary" onClick={saveAll} disabled={saving}>
            {saving ? "Saving…" : "Save vault"}
          </button>
        </div>
        {error && <div className="error">{error}</div>}
        {note && (
          <div style={{ fontSize: 13, color: "var(--color-accent-2-800)" }}>{note}</div>
        )}

        <h3 className="vault-section">Skills</h3>
        <SkillsCard />
        <BulletStandard />

        <h3 className="vault-section">Header Info</h3>
        {profile && <ProfileCard draft={profile} setDraft={setProfile} save={saveProfile} />}
        <GpaCard
          roles={experiences
            .filter((e) => e.kind === "education")
            .flatMap((e) => e.roles)}
        />

        <h3 className="vault-section">Interests</h3>
        {profile && <InterestsCard draft={profile} setDraft={setProfile} save={saveProfile} />}

        {loading ? (
          <div className="empty">Loading…</div>
        ) : (
          SECTIONS.map(([kind, label]) => {
            const inSection = experiences.filter((e) => e.kind === kind);
            return (
              <section key={kind} className="col">
                <h3 className="vault-section">
                  {label}
                  <span className="pill">{inSection.length}</span>
                </h3>
                {inSection.map((e) => (
                  <ExperienceCard key={e.id} detail={e} />
                ))}
                <div>
                  <button className="quiet" onClick={() => addExperience(kind)}>
                    + add
                  </button>
                </div>
              </section>
            );
          })
        )}

        <span className="muted" style={{ fontSize: 12 }}>
          Edits save when a field loses focus. Move an entry between sections with its Kind
          field.
        </span>
      </div>
    </VaultCtx.Provider>
  );
}
