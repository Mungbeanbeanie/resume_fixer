import type { StatusStats } from "../../types";

/// `saved` and `withdrawn` never went anywhere, so they are not on the bar.
const SEGMENTS: [keyof StatusStats, string, string][] = [
  ["applied", "Awaiting", "var(--applied)"],
  ["interview", "Interview", "var(--interview)"],
  ["offer", "Offer", "var(--offer)"],
  ["rejected", "Rejected", "var(--rejected)"],
];

export default function StackedBar({ stats }: { stats: StatusStats }) {
  const total = SEGMENTS.reduce((sum, [key]) => sum + stats[key], 0);
  const answered = stats.interview + stats.offer + stats.rejected;
  const rate = total === 0 ? 0 : Math.round((answered / total) * 100);

  if (total === 0) {
    return <div className="muted">Nothing sent yet — the graph fills in once you apply.</div>;
  }

  let x = 0;
  return (
    <div className="col" style={{ gap: 8 }}>
      <svg width="100%" height="18" role="img" aria-label="Applications by status">
        {SEGMENTS.map(([key, label, color]) => {
          const width = (stats[key] / total) * 100;
          const rect = (
            <rect key={key} x={`${x}%`} y="0" width={`${width}%`} height="18" fill={color} rx="2">
              <title>{`${label}: ${stats[key]}`}</title>
            </rect>
          );
          x += width;
          return rect;
        })}
      </svg>
      <div className="row" style={{ flexWrap: "wrap", fontSize: 12 }}>
        {SEGMENTS.map(([key, label, color]) => (
          <span key={key} className="row" style={{ gap: 5 }}>
            <span
              style={{ width: 9, height: 9, borderRadius: 2, background: color, display: "inline-block" }}
            />
            {label} {stats[key]}
          </span>
        ))}
        <span className="muted">· {rate}% answered · {stats.saved} saved, not sent</span>
      </div>
    </div>
  );
}
