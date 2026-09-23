import { useEffect, useReducer, useRef } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { errorMessage, generate } from "../../ipc";
import type { GenerationResult, HealthReport } from "../../types";
import DraftBullets from "../../DraftBullets";
import { Info } from "../../icons";

type Phase = "idle" | "ingesting" | "generating" | "ready" | "saving";

interface State {
  phase: Phase;
  url: string;
  jobText: string;
  fetched: boolean;
  pasteOpen: boolean;
  feedback: string;
  feedbackOpen: boolean;
  result: GenerationResult | null;
  error: string | null;
  saved: string | null;
  /// Edits typed in the bullet editor and not yet applied. Committing over them would put
  /// the document on screen in the library rather than the one the user just wrote.
  pendingEdits: boolean;
  /// Every compile writes the same resume.pdf, so the preview needs a changing URL.
  revision: number;
}

type Action =
  | { type: "set"; patch: Partial<State> }
  | { type: "started"; phase: Phase }
  | { type: "failed"; error: string }
  | { type: "ready"; result: GenerationResult }
  | { type: "reset" };

const initial: State = {
  phase: "idle",
  url: "",
  jobText: "",
  fetched: false,
  pasteOpen: false,
  feedback: "",
  feedbackOpen: false,
  result: null,
  error: null,
  saved: null,
  pendingEdits: false,
  revision: 0,
};

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case "set":
      return { ...state, ...action.patch };
    case "started":
      return { ...state, phase: action.phase, error: null, saved: null };
    case "failed":
      return { ...state, phase: state.result ? "ready" : "idle", error: action.error };
    case "ready":
      return {
        ...state,
        phase: "ready",
        result: action.result,
        error: null,
        feedbackOpen: false,
        pendingEdits: false,
        revision: state.revision + 1,
      };
    case "reset":
      return { ...initial };
  }
}

const STATUS: Record<Phase, string> = {
  idle: "",
  ingesting: "Reading the posting…",
  generating:
    "Tailoring: parsing the posting, choosing bullets, compiling. A local 35B model takes 30–90 seconds.",
  ready: "",
  saving: "Saving to the library…",
};

export default function GenerateTab({
  health,
  active,
}: {
  health: HealthReport | null;
  active: boolean;
}) {
  const [state, dispatch] = useReducer(reducer, initial);
  const busy = state.phase === "ingesting" || state.phase === "generating";
  const urlRef = useRef<HTMLInputElement>(null);

  async function run(feedback?: string) {
    let text = state.jobText.trim();
    let fetched = state.fetched;

    if (!text && state.url.trim()) {
      dispatch({ type: "started", phase: "ingesting" });
      try {
        const ingested = await generate.ingestJob(state.url.trim());
        if (ingested.needs_paste) {
          dispatch({
            type: "set",
            patch: {
              phase: "idle",
              pasteOpen: true,
              error: `${ingested.reason ?? "That page did not give up a job description."} Paste it instead.`,
            },
          });
          return;
        }
        text = ingested.text;
        fetched = true;
        dispatch({ type: "set", patch: { jobText: text, fetched: true } });
      } catch (e) {
        dispatch({ type: "failed", error: errorMessage(e) });
        return;
      }
    }

    if (!text) {
      dispatch({ type: "failed", error: "Paste a job link or the description first." });
      return;
    }

    dispatch({ type: "started", phase: "generating" });
    try {
      const result = await generate.fromText({
        jobText: text,
        url: state.url.trim() || null,
        fetched,
        feedback: feedback ?? null,
      });
      dispatch({ type: "ready", result });
    } catch (e) {
      dispatch({ type: "failed", error: errorMessage(e) });
    }
  }

  // ⌘Enter generates from wherever the cursor is. This tab stays mounted while another is
  // showing, so the shortcut has to check that it is the one on screen.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!active) return;
      if ((e.metaKey || e.ctrlKey) && e.key === "Enter" && !busy) {
        e.preventDefault();
        void run();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  // Typing shows immediately; the draft is told on blur, so `revise` and `commit` read the
  // name the user gave rather than the model's guess at it.
  function retitle(patch: { company?: string; role_title?: string }) {
    if (state.result) {
      dispatch({ type: "set", patch: { result: { ...state.result, ...patch } } });
    }
  }

  async function saveTitle() {
    const r = state.result;
    if (!r) return;
    try {
      await generate.rename(r.draft_id, r.company, r.role_title);
    } catch (e) {
      dispatch({ type: "failed", error: errorMessage(e) });
    }
  }

  async function save(applied: boolean) {
    if (!state.result) return;
    // Clicking a button blurs the field first, but the rename it fires is not awaited —
    // commit takes the draft away, so send the name before asking for it.
    await saveTitle();
    dispatch({ type: "started", phase: "saving" });
    try {
      await generate.commit(state.result.draft_id, applied);
      dispatch({ type: "set", patch: { phase: "idle", saved: "Saved to the library." } });
      dispatch({ type: "set", patch: { result: null, jobText: "", url: "" } });
    } catch (e) {
      dispatch({ type: "failed", error: errorMessage(e) });
    }
  }

  async function discard() {
    if (state.result) await generate.discard(state.result.draft_id).catch(() => {});
    dispatch({ type: "reset" });
  }

  async function download() {
    if (!state.result) return;
    const name = [state.result.company, state.result.role_title]
      .filter(Boolean)
      .join(" - ");
    try {
      const path = await generate.exportDraft(state.result.draft_id, name || "resume");
      dispatch({ type: "set", patch: { saved: `Saved to ${path}` } });
    } catch (e) {
      dispatch({ type: "failed", error: errorMessage(e) });
    }
  }

  if (state.result && state.phase !== "generating") {
    const r = state.result;
    return (
      <div className="col">
        <div className="row" style={{ flexWrap: "wrap", gap: "var(--space-2)" }}>
          <div className="title-edit">
            <input
              aria-label="Company"
              placeholder="Untitled posting"
              value={r.company ?? ""}
              onChange={(e) => retitle({ company: e.target.value })}
              onBlur={saveTitle}
            />
            <input
              aria-label="Role"
              placeholder="role not stated"
              value={r.role_title ?? ""}
              onChange={(e) => retitle({ role_title: e.target.value })}
              onBlur={saveTitle}
            />
          </div>
          <span className="spacer" style={{ flex: 1 }} />
          <button onClick={download}>Download</button>
          <button onClick={() => dispatch({ type: "set", patch: { feedbackOpen: !state.feedbackOpen } })}>
            Regenerate
          </button>
          <button
            className="primary"
            onClick={() => save(true)}
            disabled={state.phase === "saving" || state.pendingEdits}
          >
            Applied — save
          </button>
          <button
            onClick={() => save(false)}
            disabled={state.phase === "saving" || state.pendingEdits}
          >
            Save only
          </button>
          <button className="quiet" onClick={discard}>
            Discard
          </button>
        </div>

        {state.feedbackOpen && (
          <div className="row">
            <input
              autoFocus
              placeholder="What should change? e.g. lead with the embedded work"
              value={state.feedback}
              onChange={(e) => dispatch({ type: "set", patch: { feedback: e.target.value } })}
              onKeyDown={(e) => e.key === "Enter" && run(state.feedback)}
            />
            <button className="primary" onClick={() => run(state.feedback)}>
              Go
            </button>
          </div>
        )}

        {state.error && <div className="error">{state.error}</div>}
        {state.saved && <div className="muted">{state.saved}</div>}
        {state.pendingEdits && (
          <div className="note warn">
            <Info />
            <span>
              You have edits below that this resume does not have yet — apply them first.
            </span>
          </div>
        )}

        {r.retired.length > 0 && (
          <div className="note">
            <Info />
            <span>
              Left off to reach one page: {r.retired.join(", ")}. Pin an experience in the
              Vault to keep it regardless.
            </span>
          </div>
        )}

        {r.page_count > 1 && (
          <div className="error">
            Still {r.page_count} pages after dropping {r.dropped_for_fit} bullets. Trim a bullet in
            the Vault, or accept it as is.
          </div>
        )}

        <div className="preview-frame">
          <embed
            className="preview"
            src={`${convertFileSrc(r.pdf_path)}?v=${state.revision}`}
            type="application/pdf"
          />
          <button
            className="quiet"
            onClick={() =>
              openPath(r.pdf_path).catch((e) =>
                dispatch({ type: "failed", error: errorMessage(e) }),
              )
            }
          >
            Open in the system viewer
          </button>
        </div>

        <DraftBullets
          bullets={r.used_bullets}
          dropped={r.dropped_for_fit}
          onPending={(pendingEdits) => dispatch({ type: "set", patch: { pendingEdits } })}
          onApply={async (edits) =>
            dispatch({ type: "ready", result: await generate.revise(r.draft_id, edits) })
          }
        />

        {r.rejected.length > 0 && (
          <details className="disclosure quiet">
            <summary>
              {r.rejected.length} rewrite{r.rejected.length > 1 ? "s" : ""} rejected, original
              wording used
            </summary>
            <div className="col" style={{ margin: "var(--space-3) 0 0 22px" }}>
              {r.rejected.map((x, i) => (
                <div key={i}>
                  {x.attempted}
                  <div className="muted" style={{ fontSize: 12 }}>
                    {x.reason}
                  </div>
                </div>
              ))}
            </div>
          </details>
        )}
      </div>
    );
  }

  return (
    <div className="generate-empty">
      <h2>Paste a job link</h2>
      <div className="row">
        <input
          ref={urlRef}
          autoFocus
          placeholder="https://…"
          value={state.url}
          disabled={busy}
          onChange={(e) => dispatch({ type: "set", patch: { url: e.target.value } })}
          onKeyDown={(e) => e.key === "Enter" && run()}
        />
        <button className="primary" onClick={() => run()} disabled={busy}>
          Generate
        </button>
      </div>

      <div>
        <button
          className="quiet"
          onClick={() => dispatch({ type: "set", patch: { pasteOpen: !state.pasteOpen } })}
        >
          or paste the description
        </button>
      </div>

      {state.pasteOpen && (
        <textarea
          autoFocus
          rows={12}
          placeholder="Paste the job description here"
          value={state.jobText}
          onChange={(e) =>
            dispatch({ type: "set", patch: { jobText: e.target.value, fetched: false } })
          }
        />
      )}

      {/* Tectonic's warning rides the same line rather than stacking a second card. */}
      {busy && (
        <div className="status-line">
          <span>
            {STATUS[state.phase]}
            {health && !health.tectonic && " Tectonic is not on PATH — the PDF step will fail."}
          </span>
        </div>
      )}
      {state.error && <div className="error">{state.error}</div>}
      {state.saved && <div className="status-line">{state.saved}</div>}
      {!busy && !state.error && (
        <div className="hint">⌘↵ to generate · everything stays on this machine</div>
      )}
    </div>
  );
}
