import { useState } from "react";
import { vault } from "../../ipc";
import type { BulletDetail, Suggestion } from "../../types";
import { useVault } from "./vaultContext";

export default function BulletRow({ bullet }: { bullet: BulletDetail }) {
  const { reload, report } = useVault();
  const [text, setText] = useState(bullet.text);
  const [tags, setTags] = useState(bullet.skills.map((s) => s.name).join(", "));
  const [suggestions, setSuggestions] = useState<Suggestion[] | null>(null);
  const [thinking, setThinking] = useState(false);

  async function saveText() {
    if (text.trim() === bullet.text || !text.trim()) {
      setText(bullet.text);
      return;
    }
    try {
      await vault.upsertBullet({
        id: bullet.id,
        role_id: bullet.role_id,
        text,
        display_order: bullet.display_order,
        is_active: bullet.is_active,
      });
      await reload();
    } catch (e) {
      report(e);
    }
  }

  async function saveTags() {
    const names = tags
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);
    if (names.join(", ") === bullet.skills.map((s) => s.name).join(", ")) return;
    try {
      await vault.setBulletSkills(bullet.id, names);
      await reload();
    } catch (e) {
      report(e);
    }
  }

  async function improve() {
    setThinking(true);
    try {
      setSuggestions(await vault.suggestBulletImprovements(bullet.id));
    } catch (e) {
      report(e);
    } finally {
      setThinking(false);
    }
  }

  async function accept(variantId: string) {
    try {
      await vault.acceptVariant(variantId);
      setSuggestions(null);
      await reload();
    } catch (e) {
      report(e);
    }
  }

  return (
    <div className="bullet">
      <div style={{ flex: 1 }}>
        <textarea value={text} onChange={(e) => setText(e.target.value)} onBlur={saveText} />
        <input
          style={{ marginTop: 4, fontSize: 12 }}
          placeholder="skills, comma separated"
          value={tags}
          onChange={(e) => setTags(e.target.value)}
          onBlur={saveTags}
        />
        {suggestions && (
          <div className="col" style={{ marginTop: 6, gap: 6 }}>
            {suggestions.length === 0 && (
              <span className="muted">
                Nothing survived grounding — the wording you have is the honest one.
              </span>
            )}
            {suggestions.map((s) => (
              <div key={s.variant_id} className="row" style={{ alignItems: "flex-start" }}>
                <span style={{ flex: 1 }}>{s.text}</span>
                <button onClick={() => accept(s.variant_id)}>Use this</button>
              </div>
            ))}
            <button className="quiet" onClick={() => setSuggestions(null)}>
              dismiss
            </button>
          </div>
        )}
      </div>
      <div className="col" style={{ gap: 4 }}>
        <button className="quiet" onClick={improve} disabled={thinking}>
          {thinking ? "…" : "Improve"}
        </button>
        <button
          className="quiet danger"
          onClick={async () => {
            try {
              await vault.deleteBullet(bullet.id);
              await reload();
            } catch (e) {
              report(e);
            }
          }}
        >
          Delete
        </button>
      </div>
    </div>
  );
}
