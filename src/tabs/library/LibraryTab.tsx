import { useCallback, useEffect, useState } from "react";
import { openPath } from "@tauri-apps/plugin-opener";
import { errorMessage, library } from "../../ipc";
import type {
  ApplicationDetail,
  ApplicationStatus,
  ApplicationSummary,
  StatusStats,
} from "../../types";
import StackedBar from "./StackedBar";

const STATUSES: ApplicationStatus[] = [
  "saved",
  "applied",
  "interview",
  "offer",
  "rejected",
  "withdrawn",
];

function StatusPill({ status }: { status: ApplicationStatus }) {
  const muted = status === "saved" || status === "withdrawn";
  const good = status === "interview" || status === "offer";
  return <span className={`pill ${good ? "on" : muted ? "" : ""}`}>{status}</span>;
}

function day(iso: string | null) {
  return iso ? new Date(iso).toLocaleDateString(undefined, { month: "short", day: "numeric" }) : "—";
}

export default function LibraryTab() {
  const [rows, setRows] = useState<ApplicationSummary[]>([]);
  const [stats, setStats] = useState<StatusStats | null>(null);
  const [open, setOpen] = useState<ApplicationDetail | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const [list, s] = await Promise.all([library.listApplications(), library.stats()]);
      setRows(list);
      setStats(s);
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  async function setStatus(id: string, status: ApplicationStatus) {
    try {
      await library.setStatus(id, status);
      await load();
      if (open?.id === id) setOpen(await library.getApplication(id));
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  async function remove(id: string) {
    try {
      await library.deleteApplication(id);
      setOpen(null);
      await load();
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  return (
    <div className="col">
      {error && <div className="error">{error}</div>}
      {stats && <div className="card">{<StackedBar stats={stats} />}</div>}

      {rows.length === 0 ? (
        <div className="empty">
          Nothing here yet. Generate a resume and save it to start tracking.
        </div>
      ) : (
        <table className="apps">
          <thead>
            <tr>
              <th>Company</th>
              <th>Role</th>
              <th>Status</th>
              <th>Sent</th>
              <th>Resume</th>
              <th>Posting</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr
                key={r.id}
                onClick={async () => {
                  try {
                    setOpen(await library.getApplication(r.id));
                  } catch (e) {
                    setError(errorMessage(e));
                  }
                }}
              >
                <td>{r.company ?? "—"}</td>
                <td>{r.role_title ?? "—"}</td>
                <td>
                  <StatusPill status={r.status} />
                </td>
                <td className="muted">{day(r.applied_at ?? r.created_at)}</td>
                <td>
                  {r.pdf_path ? (
                    <button
                      className="quiet"
                      onClick={(e) => {
                        e.stopPropagation();
                        void openPath(r.pdf_path!);
                      }}
                    >
                      PDF
                    </button>
                  ) : (
                    <span className="muted">—</span>
                  )}
                </td>
                <td>
                  {r.url ? (
                    <button
                      className="quiet"
                      onClick={(e) => {
                        e.stopPropagation();
                        void openPath(r.url!);
                      }}
                    >
                      link
                    </button>
                  ) : (
                    <span className="muted">—</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {open && (
        <aside className="panel col">
          <div className="row">
            <h2>{open.company ?? "Untitled"}</h2>
            <span style={{ flex: 1 }} />
            <button className="quiet" onClick={() => setOpen(null)}>
              close
            </button>
          </div>
          <div className="muted">{open.role_title ?? "role not stated"}</div>

          <div className="row">
            <select
              value={open.status}
              onChange={(e) => setStatus(open.id, e.target.value as ApplicationStatus)}
            >
              {STATUSES.map((s) => (
                <option key={s} value={s}>
                  {s}
                </option>
              ))}
            </select>
            {open.resume?.pdf_path && (
              <button onClick={() => openPath(open.resume!.pdf_path!)}>Open PDF</button>
            )}
          </div>

          <div>
            <span className="field-label">History</span>
            {open.history.map((h, i) => (
              <div key={i} className="muted" style={{ fontSize: 12 }}>
                {h.status} · {new Date(h.changed_at).toLocaleString()}
              </div>
            ))}
          </div>

          {open.resume && (
            <div className="muted" style={{ fontSize: 12 }}>
              {open.resume.page_count} page · {open.resume.model} · prompts{" "}
              {open.resume.prompt_version}
            </div>
          )}

          <div>
            <span className="field-label">Job description</span>
            <div className="job-text">{open.job_text}</div>
          </div>

          <button className="danger" onClick={() => remove(open.id)}>
            Delete this application
          </button>
        </aside>
      )}
    </div>
  );
}
