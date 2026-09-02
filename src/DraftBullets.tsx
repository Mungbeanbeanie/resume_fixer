import { useEffect, useRef, useState } from "react";
import { errorMessage, vault } from "./ipc";
import type { BulletEdit, ExperienceDetail, UsedBullet } from "./types";

/** A vault bullet this document is not printing, offered under the entry it belongs to. */
interface Spare {
  bullet_id: string;
  text: string;
  org_name: string;
}

/// The vault bullets that could join this document but are not on it.
///
/// Limited to experiences already printing something: adding a line to an entry the resume
/// left off would print a heading with one bullet under it, and that entry has to come back
/// whole or not at all.
function spares(all: ExperienceDetail[], printed: UsedBullet[]): Spare[] {
  const used = new Set(printed.map((b) => b.bullet_id));
  return all.flatMap((e) => {
    const bullets = e.roles.flatMap((r) => r.bullets.filter((b) => b.is_active));
    if (!e.is_active || !bullets.some((b) => used.has(b.id))) return [];
    return bullets
      .filter((b) => !used.has(b.id))
      .map((b) => ({ bullet_id: b.id, text: b.text, org_name: e.org_name }));
  });
}

// The lines a resume prints, editable. Edits apply to the document in front of you — the
// vault keeps the wording it had, so trimming one resume never rewrites your master copy.
// Shared by the generated draft and the base resume preview.
export default function DraftBullets({
  bullets,
  dropped = 0,
  onApply,
  onPending,
}: {
  bullets: UsedBullet[];
  dropped?: number;
  onApply: (edits: BulletEdit[]) => Promise<void>;
  /** Told whenever edits are typed here but not yet applied to the document. */
  onPending?: (pending: boolean) => void;
}) {
  const [edits, setEdits] = useState<Record<string, { text: string; keep: boolean }>>({});
  const [available, setAvailable] = useState<Spare[]>([]);
  const [adds, setAdds] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // A recompile returns the printed lines afresh; anything typed against the old ones is
  // already baked into them.
  useEffect(() => {
    setEdits({});
    setAdds(new Set());
    setError(null);
  }, [bullets]);

  useEffect(() => {
    let live = true;
    vault
      .listExperiences()
      .then((all) => live && setAvailable(spares(all, bullets)))
      .catch(() => live && setAvailable([]));
    return () => {
      live = false;
    };
  }, [bullets]);

  const stateOf = (b: UsedBullet) => edits[b.bullet_id] ?? { text: b.rendered_text, keep: true };
  const dirty =
    adds.size > 0 ||
    bullets.some((b) => {
      const e = stateOf(b);
      return !e.keep || e.text.trim() !== b.rendered_text;
    });
  const keeping = bullets.filter((b) => stateOf(b).keep).length + adds.size;

  // Typed edits live here until Apply recompiles them, and this section is collapsed by
  // default. Saying so lets the page above refuse to save a document without them. Only on
  // a change: a caller that dispatches into a reducer would otherwise re-render forever.
  const reported = useRef(false);
  useEffect(() => {
    if (reported.current === dirty) return;
    reported.current = dirty;
    onPending?.(dirty);
  }, [dirty, onPending]);

  function patch(b: UsedBullet, next: Partial<{ text: string; keep: boolean }>) {
    setEdits((prev) => ({ ...prev, [b.bullet_id]: { ...stateOf(b), ...next } }));
  }

  function toggleAdd(id: string) {
    setAdds((prev) => {
      const next = new Set(prev);
      if (!next.delete(id)) next.add(id);
      return next;
    });
  }

  async function apply() {
    setBusy(true);
    try {
      // One list: a printed line kept or dropped, a spare one added by keeping it. The
      // backend takes the bullet's own wording for anything it is not already printing.
      await onApply([
        ...bullets.map((b) => ({ bullet_id: b.bullet_id, ...stateOf(b) })),
        ...available
          .filter((s) => adds.has(s.bullet_id))
          .map((s) => ({ bullet_id: s.bullet_id, text: s.text, keep: true })),
      ]);
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
        {available.length > 0 && `, ${available.length} more available`}
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

        {available.length > 0 && (
          <>
            <span className="field-label" style={{ marginTop: 6 }}>
              Not on this resume — from the entries it already shows
            </span>
            {available.map((s) => (
              <div className="bullet" key={s.bullet_id}>
                <div style={{ flex: 1, opacity: adds.has(s.bullet_id) ? 1 : 0.55 }}>
                  <div className="row" style={{ gap: 6, marginBottom: 3 }}>
                    <span className="muted" style={{ fontSize: 12 }}>
                      {s.org_name}
                    </span>
                  </div>
                  <textarea rows={2} value={s.text} disabled readOnly />
                </div>
                <button className="quiet" onClick={() => toggleAdd(s.bullet_id)}>
                  {adds.has(s.bullet_id) ? "Undo" : "Add"}
                </button>
              </div>
            ))}
          </>
        )}

        {error && <div className="error">{error}</div>}

        <div className="row">
          <button className="primary" onClick={apply} disabled={!dirty || busy || keeping === 0}>
            {busy ? "Recompiling…" : "Apply changes"}
          </button>
          {dirty && (
            <button
              className="quiet"
              onClick={() => {
                setEdits({});
                setAdds(new Set());
              }}
              disabled={busy}
            >
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
