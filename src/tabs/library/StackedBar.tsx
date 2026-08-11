import type { ApplicationStatus, StatusStats } from "../../types";

/// `saved` and `withdrawn` never went anywhere, so they are not on the bar. The rounds are
/// grouped: a bar of ten slivers reads as noise, and what is being asked of it is how far
/// the applications got, not which round each one is in.
const SEGMENTS: [string, ApplicationStatus[], string][] = [
  ["Awaiting", ["applied"], "var(--applied)"],
  ["OA", ["oa_received", "oa_completed"], "var(--oa)"],
  ["Interview", ["interview_1", "interview_2", "interview_3"], "var(--interview)"],
  ["Offer", ["offer"], "var(--offer)"],
  ["Rejected", ["rejected"], "var(--rejected)"],
];

const count = (stats: StatusStats, keys: ApplicationStatus[]) =>
  keys.reduce((n, k) => n + (stats[k] ?? 0), 0);

export default function StackedBar({ stats }: { stats: StatusStats }) {
  const total = SEGMENTS.reduce((sum, [, keys]) => sum + count(stats, keys), 0);
  // Anything past `applied` is somebody writing back, an assessment included.
  const answered = total - count(stats, ["applied"]);
  const rate = total === 0 ? 0 : Math.round((answered / total) * 100);

  if (total === 0) {
    return <div className="muted">Nothing sent yet — the graph fills in once you apply.</div>;
  }

  let x = 0;
  return (
    <div className="col" style={{ gap: 8 }}>
      <svg width="100%" height="18" role="img" aria-label="Applications by status">
        {SEGMENTS.map(([label, keys, color]) => {
          const width = (count(stats, keys) / total) * 100;
          const rect = (
            <rect key={label} x={`${x}%`} y="0" width={`${width}%`} height="18" fill={color} rx="2">
              <title>{`${label}: ${count(stats, keys)}`}</title>
            </rect>
          );
          x += width;
          return rect;
        })}
      </svg>
      <div className="row" style={{ flexWrap: "wrap", fontSize: 12 }}>
        {SEGMENTS.map(([label, keys, color]) => (
          <span key={label} className="row" style={{ gap: 5 }}>
            <span
              style={{ width: 9, height: 9, borderRadius: 2, background: color, display: "inline-block" }}
            />
            {label} {count(stats, keys)}
          </span>
        ))}
        <span className="muted">
          · {rate}% answered · {stats.saved ?? 0} saved, not sent
        </span>
      </div>
    </div>
  );
}
