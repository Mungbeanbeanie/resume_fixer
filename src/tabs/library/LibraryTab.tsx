import { useCallback, useEffect, useRef, useState } from "react";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { errorMessage, library } from "../../ipc";
import type {
  ApplicationDetail,
  ApplicationStatus,
  ApplicationSummary,
  StatusStats,
} from "../../types";
import Funnel from "./Funnel";
import { Plus, X } from "../../icons";

const STATUSES: ApplicationStatus[] = [
  "saved",
  "applied",
  "oa_received",
  "oa_completed",
  "interview_1",
  "interview_2",
  "interview_3",
  "offer",
  "rejected",
  "withdrawn",
];

// The stored value read out loud: "oa_received" is a database value, not a word.
const label = (status: ApplicationStatus) =>
  status.replace(/_/g, " ").replace(/^oa/, "OA");

// Sage for the stages worth reaching, terracotta for the ones still in play, neutral for the
// ones that went nowhere — so a column of statuses reads without being read.
function StatusTag({ status }: { status: ApplicationStatus }) {
  const tone = status.startsWith("interview") || status === "offer"
    ? "good"
    : status === "applied" || status.startsWith("oa")
      ? "active"
      : "neutral";
  return <span className={`tag ${tone}`}>{label(status)}</span>;
}

const BLANK = {
  company: "",
  role_title: "",
  url: "",
  status: "applied" as ApplicationStatus,
  notes: "",
};

// An application sent without generating anything here: the row, and optionally the PDF that
// went with it. The file input is the platform's — nothing about picking a file needs a
// dialog plugin when the bytes are what the backend wants.
function TrackForm({
  onSaved,
  onError,
}: {
  onSaved: () => void;
  onError: (e: unknown) => void;
}) {
  const [form, setForm] = useState(BLANK);
  const [pdf, setPdf] = useState<number[] | null>(null);
  const [busy, setBusy] = useState(false);
  const fileInput = useRef<HTMLInputElement>(null);

  async function save() {
    setBusy(true);
    try {
      await library.trackApplication({
        company: form.company.trim() || null,
        role_title: form.role_title.trim() || null,
        url: form.url.trim() || null,
        status: form.status,
        notes: form.notes.trim() || null,
        pdf,
      });
      setForm(BLANK);
      setPdf(null);
      if (fileInput.current) fileInput.current.value = "";
      onSaved();
    } catch (e) {
      onError(e);
    } finally {
      setBusy(false);
    }
  }

  return (
    <details className="disclosure">
      <summary>
        <span className="add-circle">
          <Plus />
        </span>
        <span style={{ flex: 1 }}>Track an application I already sent</span>
      </summary>
      <div className="col" style={{ marginTop: "var(--space-4)" }}>
        <div className="grid-4">
          <div>
            <span className="field-label">Company</span>
            <input
              value={form.company}
              onChange={(e) => setForm({ ...form, company: e.target.value })}
            />
          </div>
          <div>
            <span className="field-label">Role</span>
            <input
              value={form.role_title}
              onChange={(e) => setForm({ ...form, role_title: e.target.value })}
            />
          </div>
          <div>
            <span className="field-label">Posting link</span>
            <input
              placeholder="https://"
              value={form.url}
              onChange={(e) => setForm({ ...form, url: e.target.value })}
            />
          </div>
          <div>
            <span className="field-label">Status</span>
            <select
              value={form.status}
              onChange={(e) =>
                setForm({ ...form, status: e.target.value as ApplicationStatus })
              }
            >
              {STATUSES.map((s) => (
                <option key={s} value={s}>
                  {label(s)}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div>
          <span className="field-label">Resume sent (PDF)</span>
          <input
            ref={fileInput}
            type="file"
            accept="application/pdf"
            onChange={async (e) => {
              const file = e.target.files?.[0];
              setPdf(file ? Array.from(new Uint8Array(await file.arrayBuffer())) : null);
            }}
          />
        </div>

        <div>
          <span className="field-label">Notes</span>
          <textarea
            rows={2}
            value={form.notes}
            onChange={(e) => setForm({ ...form, notes: e.target.value })}
          />
        </div>

        <div className="row">
          <button onClick={save} disabled={busy || !form.company.trim()}>
            {busy ? "Saving…" : "Track it"}
          </button>
        </div>
      </div>
    </details>
  );
}

function day(iso: string | null) {
  return iso ? new Date(iso).toLocaleDateString(undefined, { month: "short", day: "numeric" }) : "—";
}

export default function LibraryTab({ active }: { active: boolean }) {
  const [rows, setRows] = useState<ApplicationSummary[]>([]);
  const [stats, setStats] = useState<StatusStats | null>(null);
  const [furthest, setFurthest] = useState<StatusStats | null>(null);
  const [open, setOpen] = useState<ApplicationDetail | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const [list, s, f] = await Promise.all([
        library.listApplications(),
        library.stats(),
        library.funnel(),
      ]);
      setRows(list);
      setStats(s);
      setFurthest(f);
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    }
  }, []);

  // This tab stays mounted while another is showing, so a resume saved from Generate lands
  // in the database behind its back. Re-read every time it comes to the front.
  useEffect(() => {
    if (active) void load();
  }, [active, load]);

  async function setStatus(id: string, status: ApplicationStatus) {
    try {
      await library.setStatus(id, status);
      await load();
      if (open?.id === id) setOpen(await library.getApplication(id));
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  // The name fields are uncontrolled: the input holds the text while it is being typed, so
  // nothing re-renders per keystroke. A blur that changed nothing writes nothing, and a
  // blank snaps back — the patch's COALESCE leaves a null field alone, so a name can be
  // changed but not cleared.
  async function rename(
    field: "company" | "role_title",
    e: React.FocusEvent<HTMLInputElement>,
  ) {
    const stored = (open && open[field]) ?? "";
    const value = e.target.value.trim();
    if (!open || !value || value === stored) {
      e.target.value = stored;
      return;
    }
    const id = open.id;
    try {
      await library.updateApplication(id, {
        company: null,
        role_title: null,
        notes: null,
        [field]: value,
      });
      await load();
      setOpen(await library.getApplication(id));
    } catch (err) {
      setError(errorMessage(err));
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
      {stats && furthest && (
        <div className="funnel-card">
          <Funnel furthest={furthest} stats={stats} />
        </div>
      )}

      <TrackForm onSaved={load} onError={(e) => setError(errorMessage(e))} />

      {rows.length === 0 ? (
        <div className="empty">
          Nothing here yet. Generate a resume and save it, or track one you already sent.
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
                  <StatusTag status={r.status} />
                </td>
                <td className="muted">{day(r.applied_at ?? r.created_at)}</td>
                <td>
                  {r.pdf_path ? (
                    <button
                      className="quiet"
                      onClick={(e) => {
                        e.stopPropagation();
                        openPath(r.pdf_path!).catch((err) => setError(errorMessage(err)));
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
                        openUrl(r.url!).catch((err) => setError(errorMessage(err)));
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
        <>
          <div className="panel-backdrop" onClick={() => setOpen(null)} />
          <aside className="panel">
            <div className="row" style={{ alignItems: "flex-start" }}>
              <div className="title-edit">
                <input
                  key={`${open.id}-company`}
                  aria-label="Company"
                  placeholder="Untitled"
                  defaultValue={open.company ?? ""}
                  onBlur={(e) => rename("company", e)}
                />
                <input
                  key={`${open.id}-role`}
                  aria-label="Role"
                  placeholder="role not stated"
                  defaultValue={open.role_title ?? ""}
                  onBlur={(e) => rename("role_title", e)}
                />
              </div>
              <span style={{ flex: 1 }} />
              <button className="close" onClick={() => setOpen(null)} aria-label="Close">
                <X />
              </button>
            </div>

            <div className="row">
              <select
                style={{ maxWidth: 220 }}
                value={open.status}
                onChange={(e) => setStatus(open.id, e.target.value as ApplicationStatus)}
              >
                {STATUSES.map((s) => (
                  <option key={s} value={s}>
                    {label(s)}
                  </option>
                ))}
              </select>
              {open.resume?.pdf_path && (
                <button
                  onClick={() =>
                    openPath(open.resume!.pdf_path!).catch((e) => setError(errorMessage(e)))
                  }
                >
                  Open PDF
                </button>
              )}
            </div>

            <div className="col" style={{ gap: 8 }}>
              <span className="eyebrow">History</span>
              {open.history.map((h, i) => (
                <div key={i} className="history-row">
                  <span>{label(h.status)}</span>
                  <span className="at">{new Date(h.changed_at).toLocaleString()}</span>
                </div>
              ))}
            </div>

            {open.resume && (
              <div className="muted" style={{ fontSize: 12 }}>
                {open.resume.model === "uploaded" ? (
                  "uploaded PDF — nothing here generated it"
                ) : (
                  <>
                    {open.resume.page_count} page · {open.resume.model} · prompts{" "}
                    {open.resume.prompt_version}
                  </>
                )}
              </div>
            )}

            {open.job_text && (
              <div className="col" style={{ gap: 8 }}>
                <span className="eyebrow">Job description</span>
                <div className="job-text">{open.job_text}</div>
              </div>
            )}

            <div style={{ marginTop: "auto" }}>
              <button className="quiet" onClick={() => remove(open.id)}>
                Delete this application
              </button>
            </div>
          </aside>
        </>
      )}
    </div>
  );
}
