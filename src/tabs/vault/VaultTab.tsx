import { useCallback, useEffect, useState } from "react";
import { errorMessage, vault } from "../../ipc";
import type { ExperienceDetail, Profile } from "../../types";
import ExperienceCard from "./ExperienceCard";
import { VaultCtx } from "./vaultContext";

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

  async function addExperience() {
    try {
      await vault.upsertExperience({
        id: null,
        kind: "work",
        org_name: "New experience",
        location: null,
        url: null,
        tech_line: null,
        display_order: experiences.length,
        is_active: true,
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
        {profile && <ProfileCard profile={profile} onSaved={setProfile} />}

        {loading ? (
          <div className="empty">Loading…</div>
        ) : experiences.length === 0 ? (
          <div className="empty">
            The vault is empty. Add an experience — every resume line comes from here.
          </div>
        ) : (
          experiences.map((e) => <ExperienceCard key={e.id} detail={e} />)
        )}

        <div className="row">
          <button className="primary" onClick={addExperience}>
            + experience
          </button>
          <span className="muted" style={{ fontSize: 12 }}>
            Edits save when a field loses focus.
          </span>
        </div>
      </div>
    </VaultCtx.Provider>
  );
}
