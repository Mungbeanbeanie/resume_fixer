import type { ApplicationStatus, StatusStats } from "../../types";

/// The stages an application passes through, in order. The rounds are grouped: what is being
/// asked is how far applications got, not which round each one is in. `saved` is not a stage
/// — it never went anywhere — and `rejected` and `withdrawn` are exits from any stage rather
/// than steps, so both sit under the funnel as a footnote.
const STAGES: [string, ApplicationStatus[], string][] = [
  ["Applied", ["applied"], "var(--applied)"],
  ["OA", ["oa_received", "oa_completed"], "var(--oa)"],
  ["Interview", ["interview_1", "interview_2", "interview_3"], "var(--interview)"],
  ["Offer", ["offer"], "var(--offer)"],
];

const count = (stats: StatusStats, keys: ApplicationStatus[]) =>
  keys.reduce((n, k) => n + (stats[k] ?? 0), 0);

/// `furthest` counts where each application stopped, so reaching a stage means stopping at it
/// or anywhere past it. Summing forwards is what makes the funnel narrow monotonically.
export default function Funnel({
  furthest,
  stats,
}: {
  furthest: StatusStats;
  stats: StatusStats;
}) {
  const reached = STAGES.map((_, i) =>
    STAGES.slice(i).reduce((sum, [, keys]) => sum + count(furthest, keys), 0),
  );
  const sent = reached[0];

  if (sent === 0) {
    return <div className="muted">Nothing sent yet — the funnel fills in once you apply.</div>;
  }

  // Anything that got past Applied is an answer of some kind, which is the number the
  // headline is read for: rejections are counted at the stage they happened, not here.
  const response = Math.round((reached[1] / sent) * 100);

  return (
    <>
      <div className="headline">
        <span className="rate">{response}%</span>
        <span className="muted">
          response rate · {sent} sent
        </span>
      </div>

      {STAGES.map(([label, , color], i) => {
        // Share of everything sent, so the taper is the drop-off itself.
        const width = (reached[i] / sent) * 100;
        // Against the stage above, which is the number a funnel is actually read for.
        const rate = i === 0 ? null : Math.round((reached[i] / reached[i - 1]) * 100);
        return (
          <div className="funnel-row" key={label}>
            <span>{label}</span>
            <div className="funnel-track" title={`${label}: ${reached[i]} of ${sent}`}>
              <div className="funnel-bar" style={{ width: `${width}%`, background: color }} />
            </div>
            <span className="count">
              <b>{reached[i]}</b>
              {rate !== null && <span className="muted"> · {rate}%</span>}
            </span>
          </div>
        );
      })}

      <div className="funnel-foot">
        {stats.rejected ?? 0} rejected · {stats.withdrawn ?? 0} withdrawn ·{" "}
        {stats.saved ?? 0} saved, not sent
      </div>
    </>
  );
}
