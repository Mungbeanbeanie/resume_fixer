import { useCallback, useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { base, errorMessage, vault } from "../../ipc";
import type { BasePreview, ExperienceDetail, ExperienceKind, Template } from "../../types";
import DraftBullets from "../../DraftBullets";

// The entries you pick by hand, in the order the vault holds them. Education and
// certifications are not offered: they print on every resume regardless.
const GROUPS: [string, ExperienceKind][] = [
  ["Experience", "work"],
  ["Projects", "project"],
  ["Activities", "activity"],
];

// A resume built by hand: no posting, no model, and nothing you picked is dropped to reach
// one page. Every line is a stored bullet verbatim, same as the base resume.
export default function BuildTab() {
  const [templates, setTemplates] = useState<Template[]>([]);
  const [all, setAll] = useState<ExperienceDetail[]>([]);
  const [picked, setPicked] = useState<Set<string>>(new Set());
  const [selected, setSelected] = useState<string>("");
  const [preview, setPreview] = useState<BasePreview | null>(null);
  // Every compile overwrites the same build.pdf, so the embed needs a changing URL.
  const [revision, setRevision] = useState(0);
  const [name, setName] = useState("");
  const [pendingEdits, setPendingEdits] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);

  const reload = useCallback(async () => {
    try {
      const [list, experiences] = await Promise.all([
        base.listTemplates(),
        vault.listExperiences(),
      ]);
      setTemplates(list);
      setAll(experiences.filter((e) => e.is_active));
      setSelected((s) => s || list.find((t) => t.is_active)?.id || list[0]?.id || "");
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  function toggle(id: string) {
    setPicked((prev) => {
      const next = new Set(prev);
      if (!next.delete(id)) next.add(id);
      return next;
    });
  }

  async function build() {
    setBusy(true);
    setNote(null);
    try {
      setPreview(await base.buildRender([...picked], selected || null));
      setRevision((r) => r + 1);
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  async function save() {
    try {
      const row = await base.buildSave(name);
      setName("");
      setNote(`Saved “${row.name}” — it is in the Base tab with the rest.`);
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  return (
    <div className="col">
      <div className="row">
        <div>
          <span className="field-label">Template</span>
          <select value={selected} onChange={(e) => setSelected(e.target.value)}>
            {templates.map((t) => (
              <option key={t.id} value={t.id}>
                {t.name}
                {t.is_builtin ? " (built in)" : ""}
              </option>
            ))}
          </select>
        </div>
        <span style={{ flex: 1 }} />
        <button
          className="primary"
          onClick={build}
          disabled={busy || !selected || picked.size === 0}
        >
          {busy ? "Compiling…" : `Build from ${picked.size} entr${picked.size === 1 ? "y" : "ies"}`}
        </button>
      </div>

      {error && <div className="error">{error}</div>}
      {note && <div className="muted">{note}</div>}

      {GROUPS.map(([label, kind]) => {
        const rows = all.filter((e) => e.kind === kind);
        if (rows.length === 0) return null;
        return (
          <div className="card col" key={kind} style={{ gap: 6 }}>
            <span className="field-label">{label}</span>
            {rows.map((e) => (
              <label className="row" key={e.id} style={{ gap: 8 }}>
                <input
                  type="checkbox"
                  checked={picked.has(e.id)}
                  onChange={() => toggle(e.id)}
                />
                <strong>{e.org_name}</strong>
                <span className="muted" style={{ fontSize: 12 }}>
                  {e.roles.map((r) => r.title).join(" · ")}
                </span>
                {e.is_pinned && <span className="pill on">always printed</span>}
              </label>
            ))}
          </div>
        );
      })}
      <span className="muted" style={{ fontSize: 12 }}>
        Education and certifications print on every resume, so they are not listed here.
      </span>

      {preview && (
        <>
          <div className="row">
            <span className="muted">
              {preview.template_name} · {preview.bullet_count} bullets ·{" "}
              {preview.page_count} page{preview.page_count === 1 ? "" : "s"}
            </span>
            <span style={{ flex: 1 }} />
            <input
              placeholder="Name this resume"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && name.trim() && !pendingEdits && save()}
            />
            <button className="primary" onClick={save} disabled={!name.trim() || pendingEdits}>
              Save
            </button>
          </div>
          {pendingEdits && (
            <div className="muted">
              You have edits below that this resume does not have yet — apply them first.
            </div>
          )}
          {preview.page_count > 1 && (
            <div className="muted">
              {preview.page_count} pages — nothing you picked was dropped. Untick an entry, or
              remove a line below, to reach one.
            </div>
          )}
          <DraftBullets
            bullets={preview.used_bullets}
            onPending={setPendingEdits}
            onApply={async (edits) => {
              setPreview(await base.buildRevise(edits));
              setRevision((r) => r + 1);
            }}
          />
          <embed
            className="preview"
            src={`${convertFileSrc(preview.pdf_path)}?v=${revision}`}
            type="application/pdf"
          />
        </>
      )}
    </div>
  );
}
