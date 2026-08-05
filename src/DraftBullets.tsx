import { useEffect, useState } from "react";
import { errorMessage } from "./ipc";
import type { BulletEdit, UsedBullet } from "./types";

// The lines a resume prints, editable. Edits apply to the document in front of you — the
// vault keeps the wording it had, so trimming one resume never rewrites your master copy.
// Shared by the generated draft and the base resume preview.
export default function DraftBullets({
  bullets,
  dropped = 0,
  onApply,
}: {
  bullets: UsedBullet[];
  dropped?: number;
  onApply: (edits: BulletEdit[]) => Promise<void>;
}) {
  const [edits, setEdits] = useState<Record<string, { text: string; keep: boolean }>>({});
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // A recompile returns the printed lines afresh; anything typed against the old ones is
  // already baked into them.
  useEffect(() => {
    setEdits({});
    setError(null);
  }, [bullets]);

  const stateOf = (b: UsedBullet) => edits[b.bullet_id] ?? { text: b.rendered_text, keep: true };
  const dirty = bullets.some((b) => {
    const e = stateOf(b);
    return !e.keep || e.text.trim() !== b.rendered_text;
  });
  const keeping = bullets.filter((b) => stateOf(b).keep).length;

  function patch(b: UsedBullet, next: Partial<{ text: string; keep: boolean }>) {
    setEdits((prev) => ({ ...prev, [b.bullet_id]: { ...stateOf(b), ...next } }));
  }

  async function apply() {
    setBusy(true);
    try {
      await onApply(bullets.map((b) => ({ bullet_id: b.bullet_id, ...stateOf(b) })));
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  const reworded = bullets.filter((b) => b.was_reworded).length;
  const edited = bullets.filter(
    (b) => !b.was_reworded && b.rendered_text !== b.source_text,
  ).length;

  return (
    <details className="disclosure">
      <summary>
        {bullets.length} bullets printed
        {reworded > 0 && `, ${reworded} reworded`}
        {edited > 0 && `, ${edited} edited`}
        {dropped > 0 && `, ${dropped} dropped to fit`}
      </summary>

      <div className="col" style={{ gap: 10, marginTop: 10 }}>
        {bullets.map((b) => {
          const e = stateOf(b);
          return (
            <div className="bullet" key={b.bullet_id}>
              <div style={{ flex: 1, opacity: e.keep ? 1 : 0.45 }}>
                <div className="row" style={{ gap: 6, marginBottom: 3 }}>
                  <span className="muted" style={{ fontSize: 12 }}>
                    {b.org_name}
                  </span>
                  {b.was_reworded && <span className="pill on">reworded</span>}
                  {!b.was_reworded && b.rendered_text !== b.source_text && (
                    <span className="pill on">edited</span>
                  )}
                </div>
                <textarea
                  rows={2}
                  value={e.text}
                  disabled={!e.keep}
                  onChange={(ev) => patch(b, { text: ev.target.value })}
                />
              </div>
              <button className="quiet" onClick={() => patch(b, { keep: !e.keep })}>
                {e.keep ? "Remove" : "Restore"}
              </button>
            </div>
          );
        })}

        {error && <div className="error">{error}</div>}

        <div className="row">
          <button className="primary" onClick={apply} disabled={!dirty || busy || keeping === 0}>
            {busy ? "Recompiling…" : "Apply changes"}
          </button>
          {dirty && (
            <button className="quiet" onClick={() => setEdits({})} disabled={busy}>
              reset
            </button>
          )}
          <span className="muted" style={{ fontSize: 12 }}>
            {keeping === 0
              ? "Keep at least one bullet."
              : "Edits apply to this resume only — the vault keeps its wording."}
          </span>
        </div>
      </div>
    </details>
  );
}
