import { useEffect, useState } from "react";
import { base, errorMessage } from "../../ipc";
import type { Template } from "../../types";

// Paste or edit a Tera-over-LaTeX template. Saving validates it before it can reach a
// render, and a built-in has to be saved under a new name — startup re-syncs those from
// disk.
export default function TemplateEditor({
  template,
  onSaved,
}: {
  template: Template | null;
  onSaved: (t: Template) => void;
}) {
  const [name, setName] = useState("");
  const [source, setSource] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState<string | null>(null);

  useEffect(() => {
    setName(template?.is_builtin ? `${template.name} (copy)` : (template?.name ?? ""));
    setSource(template?.source ?? "");
    setError(null);
    setSaved(null);
  }, [template]);

  async function save() {
    try {
      const t = await base.saveTemplate(name, source);
      setSaved(`Saved ${t.name}.`);
      setError(null);
      onSaved(t);
    } catch (e) {
      setError(errorMessage(e));
      setSaved(null);
    }
  }

  return (
    <details className="disclosure quiet template-editor">
      <summary>Edit or paste a template</summary>
      <div className="col" style={{ gap: "var(--space-2)", marginTop: "var(--space-3)" }}>
        <div className="row" style={{ gap: "var(--space-2)" }}>
          <input
            placeholder="Template name"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <button className="primary" onClick={save} disabled={!name.trim() || !source.trim()}>
            Save template
          </button>
        </div>
        <textarea
          rows={14}
          spellCheck={false}
          placeholder="Paste a .tex.tera template here"
          value={source}
          onChange={(e) => setSource(e.target.value)}
        />
        {error && <div className="error">{error}</div>}
        {saved && <div className="muted">{saved}</div>}
        <span className="muted" style={{ fontSize: 12 }}>
          Content comes from the vault through Tera variables: full_name, phone, email,
          links, education, experience, projects, activities, certifications, skills_line,
          interests. A
          section is printed only if the template names its variable.
        </span>
      </div>
    </details>
  );
}
