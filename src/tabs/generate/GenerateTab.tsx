import { useEffect, useReducer, useRef } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { errorMessage, generate } from "../../ipc";
import type { GenerationResult, HealthReport } from "../../types";

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
      return { ...state, phase: "ready", result: action.result, error: null, feedbackOpen: false };
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

export default function GenerateTab({ health }: { health: HealthReport | null }) {
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
              error: "That page did not give up a job description. Paste it instead.",
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

  // ⌘Enter generates from wherever the cursor is.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "Enter" && !busy) {
        e.preventDefault();
        void run();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  async function save(applied: boolean) {
    if (!state.result) return;
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
    const reworded = r.used_bullets.filter((b) => b.was_reworded).length;
    return (
      <div className="col">
        <div className="row">
          <div>
            <h2>{r.company ?? "Untitled posting"}</h2>
            <span className="muted">{r.role_title ?? "role not stated"}</span>
          </div>
          <span className="spacer" style={{ flex: 1 }} />
          <button onClick={download}>Download</button>
          <button onClick={() => dispatch({ type: "set", patch: { feedbackOpen: !state.feedbackOpen } })}>
            Regenerate
          </button>
          <button className="primary" onClick={() => save(true)} disabled={state.phase === "saving"}>
            Applied — save
          </button>
          <button onClick={() => save(false)} disabled={state.phase === "saving"}>
            Save only
          </button>
          <button className="danger" onClick={discard}>
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

        {r.page_count > 1 && (
          <div className="error">
            Still {r.page_count} pages after dropping {r.dropped_for_fit} bullets. Trim a bullet in
            the Vault, or accept it as is.
          </div>
        )}

        <embed className="preview" src={convertFileSrc(r.pdf_path)} type="application/pdf" />
        <button className="quiet" onClick={() => openPath(r.pdf_path)}>
          Open in the system viewer
        </button>

        <details className="disclosure">
          <summary>
            {r.used_bullets.length} bullets used, {reworded} reworded
            {r.dropped_for_fit > 0 && `, ${r.dropped_for_fit} dropped to fit`}
          </summary>
          <ul className="bullet-list">
            {r.used_bullets.map((b) => (
              <li key={b.bullet_id}>
                <span className="muted">{b.org_name}</span> — {b.rendered_text}
                {b.was_reworded && <span className="pill on" style={{ marginLeft: 6 }}>reworded</span>}
              </li>
            ))}
          </ul>
        </details>

        {r.rejected.length > 0 && (
          <details className="disclosure">
            <summary>
              {r.rejected.length} rewrite{r.rejected.length > 1 ? "s" : ""} rejected, original
              wording used
            </summary>
            <ul className="bullet-list">
              {r.rejected.map((x, i) => (
                <li key={i}>
                  {x.attempted}
                  <div className="muted">{x.reason}</div>
                </li>
              ))}
            </ul>
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

      <button
        className="quiet"
        style={{ marginTop: 8 }}
        onClick={() => dispatch({ type: "set", patch: { pasteOpen: !state.pasteOpen } })}
      >
        or paste the description
      </button>

      {state.pasteOpen && (
        <textarea
          autoFocus
          rows={12}
          style={{ marginTop: 8, textAlign: "left" }}
          placeholder="Paste the job description here"
          value={state.jobText}
          onChange={(e) =>
            dispatch({ type: "set", patch: { jobText: e.target.value, fetched: false } })
          }
        />
      )}

      {busy && <div className="status-line">{STATUS[state.phase]}</div>}
      {busy && health && !health.tectonic && (
        <div className="status-line">Tectonic is not on PATH — the PDF step will fail.</div>
      )}
      {state.error && (
        <div className="error" style={{ marginTop: 16, textAlign: "left" }}>
          {state.error}
        </div>
      )}
      {state.saved && <div className="status-line">{state.saved}</div>}
      {!busy && !state.error && (
        <div className="status-line" style={{ fontSize: 12 }}>
          ⌘↵ to generate · everything stays on this machine
        </div>
      )}
    </div>
  );
}
