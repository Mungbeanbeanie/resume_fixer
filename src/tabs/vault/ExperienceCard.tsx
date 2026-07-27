import { useState } from "react";
import { vault } from "../../ipc";
import type { ExperienceDetail, ExperienceKind, RoleDetail } from "../../types";
import BulletRow from "./BulletRow";
import { useVault } from "./vaultContext";

const KINDS: ExperienceKind[] = ["work", "project", "education", "certification"];

function RoleBlock({ role }: { role: RoleDetail }) {
  const { reload, report } = useVault();
  const [draft, setDraft] = useState(role);

  async function save(patch: Partial<RoleDetail>) {
    const next = { ...draft, ...patch };
    setDraft(next);
    try {
      await vault.upsertRole({
        id: next.id,
        experience_id: next.experience_id,
        title: next.title,
        location: next.location,
        start_date: next.start_date,
        end_date: next.end_date,
        date_override: next.date_override,
        display_order: next.display_order,
        is_active: next.is_active,
      });
      await reload();
    } catch (e) {
      report(e);
    }
  }

  return (
    <div className="role">
      <div className="grid-4">
        <div>
          <span className="field-label">Title</span>
          <input
            value={draft.title}
            onChange={(e) => setDraft({ ...draft, title: e.target.value })}
            onBlur={() => save({})}
          />
        </div>
        <div>
          <span className="field-label">Start</span>
          <input
            type="date"
            value={draft.start_date}
            onChange={(e) => save({ start_date: e.target.value })}
          />
        </div>
        <div>
          <span className="field-label">End (blank = present)</span>
          <input
            type="date"
            value={draft.end_date ?? ""}
            onChange={(e) => save({ end_date: e.target.value || null })}
          />
        </div>
        <div>
          <span className="field-label">Date override</span>
          <input
            placeholder="e.g. Summer 2025"
            value={draft.date_override ?? ""}
            onChange={(e) => setDraft({ ...draft, date_override: e.target.value || null })}
            onBlur={() => save({})}
          />
        </div>
      </div>

      {role.bullets.map((b) => (
        <BulletRow key={b.id} bullet={b} />
      ))}

      <div className="row" style={{ marginTop: 8 }}>
        <button
          className="quiet"
          onClick={async () => {
            try {
              await vault.upsertBullet({
                id: null,
                role_id: role.id,
                text: "New bullet",
                display_order: role.bullets.length,
                is_active: true,
              });
              await reload();
            } catch (e) {
              report(e);
            }
          }}
        >
          + bullet
        </button>
        <button
          className="quiet danger"
          onClick={async () => {
            try {
              await vault.deleteRole(role.id);
              await reload();
            } catch (e) {
              report(e);
            }
          }}
        >
          delete role
        </button>
      </div>
    </div>
  );
}

export default function ExperienceCard({ detail }: { detail: ExperienceDetail }) {
  const { reload, report } = useVault();
  const [draft, setDraft] = useState(detail);
  const [tags, setTags] = useState(detail.skills.map((s) => s.name).join(", "));

  async function save(patch: Partial<ExperienceDetail> = {}) {
    const next = { ...draft, ...patch };
    setDraft(next);
    try {
      await vault.upsertExperience(
        {
          id: next.id,
          kind: next.kind,
          org_name: next.org_name,
          location: next.location,
          url: next.url,
          tech_line: next.tech_line,
          display_order: next.display_order,
          is_active: next.is_active,
        },
        tags
          .split(",")
          .map((t) => t.trim())
          .filter(Boolean),
      );
      await reload();
    } catch (e) {
      report(e);
    }
  }

  const roleCount = detail.roles.length;
  const bulletCount = detail.roles.reduce((n, r) => n + r.bullets.length, 0);

  return (
    <details className="experience">
      <summary>
        <strong>{detail.org_name}</strong>
        <span className="pill">{detail.kind}</span>
        <span className="muted" style={{ fontSize: 12 }}>
          {roleCount} role{roleCount === 1 ? "" : "s"} · {bulletCount} bullet
          {bulletCount === 1 ? "" : "s"}
        </span>
        {!detail.is_active && <span className="pill off">hidden</span>}
      </summary>

      <div className="body">
        <div className="grid-4" style={{ marginTop: 12 }}>
          <div>
            <span className="field-label">Organization</span>
            <input
              value={draft.org_name}
              onChange={(e) => setDraft({ ...draft, org_name: e.target.value })}
              onBlur={() => save()}
            />
          </div>
          <div>
            <span className="field-label">Location</span>
            <input
              value={draft.location ?? ""}
              onChange={(e) => setDraft({ ...draft, location: e.target.value || null })}
              onBlur={() => save()}
            />
          </div>
          <div>
            <span className="field-label">Kind</span>
            <select
              value={draft.kind}
              onChange={(e) => save({ kind: e.target.value as ExperienceKind })}
            >
              {KINDS.map((k) => (
                <option key={k} value={k}>
                  {k}
                </option>
              ))}
            </select>
          </div>
          <div>
            <span className="field-label">Tech line (projects)</span>
            <input
              value={draft.tech_line ?? ""}
              onChange={(e) => setDraft({ ...draft, tech_line: e.target.value || null })}
              onBlur={() => save()}
            />
          </div>
        </div>

        <div style={{ marginTop: 12 }}>
          <span className="field-label">Skills on this experience</span>
          <input
            placeholder="comma separated"
            value={tags}
            onChange={(e) => setTags(e.target.value)}
            onBlur={() => save()}
          />
        </div>

        {detail.roles.map((r) => (
          <RoleBlock key={r.id} role={r} />
        ))}

        <div className="row" style={{ marginTop: 12 }}>
          <button
            onClick={async () => {
              try {
                await vault.upsertRole({
                  id: null,
                  experience_id: detail.id,
                  title: "New role",
                  location: null,
                  start_date: new Date().toISOString().slice(0, 10),
                  end_date: null,
                  date_override: null,
                  display_order: detail.roles.length,
                  is_active: true,
                });
                await reload();
              } catch (e) {
                report(e);
              }
            }}
          >
            + role
          </button>
          <button onClick={() => save({ is_active: !draft.is_active })}>
            {draft.is_active ? "Hide from resumes" : "Show on resumes"}
          </button>
          <span style={{ flex: 1 }} />
          <button
            className="danger"
            onClick={async () => {
              try {
                await vault.deleteExperience(detail.id);
                await reload();
              } catch (e) {
                report(e);
              }
            }}
          >
            Delete experience
          </button>
        </div>
      </div>
    </details>
  );
}
