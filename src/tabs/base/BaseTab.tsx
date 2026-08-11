import { useCallback, useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { base, errorMessage } from "../../ipc";
import type { BasePreview, BaseResume, Template } from "../../types";
import DraftBullets from "../../DraftBullets";
import TemplateEditor from "./TemplateEditor";

// The base resume is the whole vault through one template: no posting, no model, no
// bullets dropped to reach one page.
export default function BaseTab() {
  const [templates, setTemplates] = useState<Template[]>([]);
  const [saved, setSaved] = useState<BaseResume[]>([]);
  const [selected, setSelected] = useState<string>("");
  const [preview, setPreview] = useState<BasePreview | null>(null);
  // Every compile overwrites the same preview.pdf, so the embed needs a changing URL.
  const [revision, setRevision] = useState(0);
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);

  const reload = useCallback(async () => {
    try {
      const [list, resumes] = await Promise.all([base.listTemplates(), base.list()]);
      setTemplates(list);
      setSaved(resumes);
      setSelected((s) => s || list.find((t) => t.is_active)?.id || list[0]?.id || "");
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const current = templates.find((t) => t.id === selected) ?? null;

  async function build(templateId?: string) {
    setBusy(true);
    setNote(null);
    try {
      const result = await base.render(templateId ?? (selected || null));
      setPreview(result);
      setRevision((r) => r + 1);
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  // Reopening rebuilds the preview through the template the saved resume was built with,
  // so it can be trimmed and saved again. The saved PDF itself is untouched — the lines
  // come from today's vault, not from the stored .tex.
  async function reopen(row: BaseResume) {
    const template = templates.find((t) => t.name === row.template_name);
    if (!template) {
      setError(`The template “${row.template_name}” is no longer here, so this cannot be reopened.`);
      return;
    }
    setSelected(template.id);
    await build(template.id);
  }

  async function useForGenerated() {
    if (!selected) return;
    try {
      await base.setActiveTemplate(selected);
      await reload();
      setNote(`${current?.name} is now used for generated resumes too.`);
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  async function save() {
    try {
      const row = await base.save(name);
      setName("");
      setNote(`Saved “${row.name}”.`);
      await reload();
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  async function download(row: BaseResume) {
    try {
      setNote(`Saved to ${await base.exportPdf(row.id, row.name)}`);
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  async function remove(row: BaseResume) {
    try {
      await base.remove(row.id);
      await reload();
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  async function deleteTemplate() {
    if (!current) return;
    try {
      await base.deleteTemplate(current.id);
      setSelected("");
      await reload();
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
                {t.is_active ? " ·  in use" : ""}
              </option>
            ))}
          </select>
        </div>
        <span style={{ flex: 1 }} />
        <button className="primary" onClick={() => build()} disabled={busy || !selected}>
          {busy ? "Compiling…" : "Build base resume"}
        </button>
        <button onClick={useForGenerated} disabled={!current || current.is_active}>
          Use for generated resumes
        </button>
        {current && !current.is_builtin && (
          <button className="quiet danger" onClick={deleteTemplate}>
            Delete template
          </button>
        )}
      </div>

      {error && <div className="error">{error}</div>}
      {note && <div className="muted">{note}</div>}

      <TemplateEditor template={current} onSaved={() => void reload()} />

      {preview && (
        <>
          <div className="row">
            <span className="muted">
              {preview.template_name} · {preview.bullet_count} bullets ·{" "}
              {preview.page_count} page{preview.page_count === 1 ? "" : "s"}
            </span>
            <span style={{ flex: 1 }} />
            <input
              placeholder="Name this base resume"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && name.trim() && save()}
            />
            <button className="primary" onClick={save} disabled={!name.trim()}>
              Save
            </button>
          </div>
          {preview.retired.length > 0 && (
            <div className="muted">
              Left off to reach one page: {preview.retired.join(", ")}. Pin an experience in
              the Vault to keep it regardless.
            </div>
          )}
          <DraftBullets
            bullets={preview.used_bullets}
            onApply={async (edits) => {
              setPreview(await base.revise(edits));
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

      <h3 className="vault-section">Saved base resumes</h3>
      {saved.length === 0 ? (
        <div className="empty">
          None yet. Pick a template, build it, and name what comes out.
        </div>
      ) : (
        <ul className="bullet-list">
          {saved.map((row) => (
            <li key={row.id}>
              <div className="row">
                <strong>{row.name}</strong>
                <span className="muted" style={{ fontSize: 12 }}>
                  {row.template_name} · {row.page_count ?? "?"} page
                  {row.page_count === 1 ? "" : "s"} ·{" "}
                  {new Date(row.created_at).toLocaleDateString()}
                </span>
                <span style={{ flex: 1 }} />
                <button
                  className="quiet"
                  onClick={() =>
                    openPath(row.pdf_path!).catch((e) => setError(errorMessage(e)))
                  }
                  disabled={!row.pdf_path}
                >
                  Open
                </button>
                <button className="quiet" onClick={() => reopen(row)} disabled={busy}>
                  Reopen
                </button>
                <button className="quiet" onClick={() => download(row)}>
                  Download
                </button>
                <button className="quiet danger" onClick={() => remove(row)}>
                  Delete
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
