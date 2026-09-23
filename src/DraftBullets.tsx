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

/// Pinned entries that did not reach this document after all.
///
/// A pin means "always print this", so the remaining way to miss is a template with no
/// section for the entry — which is how a pinned Activity vanishes under Jake's. Matches on
/// bullet ids, not org names, because two entries may share a name. An entry with no active
/// bullet is skipped: it prints as a heading alone, which leaves no bullet here to see it by,
/// and guessing would name entries that are on the page.
function missingPins(all: ExperienceDetail[], printed: UsedBullet[]): string[] {
  const used = new Set(printed.map((b) => b.bullet_id));
  return all
    .filter((e) => e.is_active && e.is_pinned)
    .filter((e) => {
      const bullets = e.roles.flatMap((r) => r.bullets.filter((b) => b.is_active));
      return bullets.length > 0 && !bullets.some((b) => used.has(b.id));
    })
    .map((e) => e.org_name);
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
  const [unprinted, setUnprinted] = useState<string[]>([]);
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
      .then((all) => {
        if (!live) return;
        setAvailable(spares(all, bullets));
        setUnprinted(missingPins(all, bullets));
      })
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

  const meta = [
    reworded > 0 && `${reworded} reworded`,
    edited > 0 && `${edited} edited`,
    dropped > 0 && `${dropped} dropped to fit`,
    available.length > 0 && `${available.length} more available`,
    unprinted.length > 0 && `${unprinted.length} “always print” missing`,
  ].filter(Boolean);

  return (
    <details className="disclosure" open>
      <summary>
        <span className="title">{bullets.length} bullets printed</span>
        <span className="meta">{meta.join(", ")}</span>
      </summary>

      <div className="col" style={{ marginTop: "var(--space-3)" }}>
        {unprinted.length > 0 && (
          <div className="muted">
            Marked “always print” but not on this resume: {unprinted.join(", ")}. This
            template has no section for it — build with one that does.
          </div>
        )}
        {bullets.map((b) => {
          const e = stateOf(b);
          return (
            <div className="draft-bullet" key={b.bullet_id}>
              <div className="body" style={{ opacity: e.keep ? 1 : 0.45 }}>
                <div className="row" style={{ gap: 6 }}>
                  <span className="org">{b.org_name}</span>
                  {b.was_reworded && <span className="tag good">reworded</span>}
                  {!b.was_reworded && b.rendered_text !== b.source_text && (
                    <span className="tag good">edited</span>
                  )}
                </div>
                <textarea
                  rows={2}
                  value={e.text}
                  disabled={!e.keep}
                  onChange={(ev) => patch(b, { text: ev.target.value })}
                />
              </div>
              <button className="quiet subtle" onClick={() => patch(b, { keep: !e.keep })}>
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
            {available.map((s) => {
              const added = adds.has(s.bullet_id);
              return (
                <div
                  className={`draft-bullet${added ? "" : " spare"}`}
                  key={s.bullet_id}
                >
                  <div className="body">
                    <span className="org">{s.org_name}</span>
                    <div className="text">{s.text}</div>
                  </div>
                  <button className="quiet" onClick={() => toggleAdd(s.bullet_id)}>
                    {added ? "Undo" : "Add"}
                  </button>
                </div>
              );
            })}
          </>
        )}

        {error && <div className="error">{error}</div>}

        <div className="row" style={{ marginTop: "var(--space-2)" }}>
          <button className="primary" onClick={apply} disabled={!dirty || busy || keeping === 0}>
            {busy ? "Recompiling…" : "Apply changes"}
          </button>
          {dirty && (
            <button
              className="quiet subtle"
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
