import { useCallback, useEffect, useState } from "react";
import { errorMessage, vault } from "../../ipc";
import type { ExperienceDetail, ExperienceKind, Profile } from "../../types";
import BulletStandard from "./BulletStandard";
import ExperienceCard from "./ExperienceCard";
import SkillsCard from "./SkillsCard";
import { VaultCtx } from "./vaultContext";

// The order the vault reads in. Header Info is the profile card and has no kind.
const SECTIONS: [ExperienceKind, string][] = [
  ["education", "School"],
  ["work", "Work Experience"],
  ["project", "Projects"],
  ["activity", "Activities / Leadership"],
  ["certification", "Certifications"],
];

function ProfileCard({ profile, onSaved }: { profile: Profile; onSaved: (p: Profile) => void }) {
  const [draft, setDraft] = useState(profile);
  const [error, setError] = useState<string | null>(null);

  async function save(next: Profile) {
    setDraft(next);
    try {
      onSaved(await vault.upsertProfile(next));
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  return (
    <details className="experience">
      <summary>
        <strong>{draft.full_name || "Your name"}</strong>
        <span className="muted" style={{ fontSize: 12 }}>
          {draft.email ?? "no email"} · {draft.links.length} link
          {draft.links.length === 1 ? "" : "s"}
        </span>
      </summary>
      <div className="body">
        {error && <div className="error">{error}</div>}
        <div className="grid-4" style={{ marginTop: 12 }}>
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
        <div className="col" style={{ marginTop: 12, gap: 6 }}>
          <span className="field-label">Links printed in the header</span>
          {draft.links.map((l, i) => (
            <div className="row" key={i}>
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
                className="quiet danger"
                onClick={() => save({ ...draft, links: draft.links.filter((_, j) => j !== i) })}
              >
                ×
              </button>
            </div>
          ))}
          <button
            className="quiet"
            onClick={() => setDraft({ ...draft, links: [...draft.links, { label: "", url: "" }] })}
          >
            + link
          </button>
        </div>
      </div>
    </details>
  );
}

export default function VaultTab() {
  const [experiences, setExperiences] = useState<ExperienceDetail[]>([]);
  const [profile, setProfile] = useState<Profile | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const reload = useCallback(async () => {
    try {
      const [list, p] = await Promise.all([vault.listExperiences(), vault.getProfile()]);
      setExperiences(list);
      setProfile(p ?? { full_name: "", phone: null, email: null, links: [] });
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

  async function addExperience(kind: ExperienceKind) {
    try {
      await vault.upsertExperience({
        id: null,
        kind,
        org_name: "New entry",
        location: null,
        url: null,
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
        {error && <div className="error">{error}</div>}

        <h3 className="vault-section">Skills</h3>
        <SkillsCard experiences={experiences} />
        <BulletStandard />

        <h3 className="vault-section">Header Info</h3>
        {profile && <ProfileCard profile={profile} onSaved={setProfile} />}

        {loading ? (
          <div className="empty">Loading…</div>
        ) : (
          SECTIONS.map(([kind, label]) => {
            const inSection = experiences.filter((e) => e.kind === kind);
            return (
              <section key={kind} className="col">
                <h3 className="vault-section">
                  {label}
                  <span className="muted">{inSection.length}</span>
                </h3>
                {inSection.map((e) => (
                  <ExperienceCard key={e.id} detail={e} />
                ))}
                <div className="row">
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
