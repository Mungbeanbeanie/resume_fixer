// The four Lucide glyphs the design uses, inlined. Chevrons and the skill-chip check are
// drawn in CSS, so nothing else is needed and `lucide-react` would ship a whole set for four.
const base = {
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 2.75,
  strokeLinecap: "round",
  strokeLinejoin: "round",
} as const;

/** Re-check the local services. */
export const RefreshCw = ({ size = 15 }: { size?: number }) => (
  <svg width={size} height={size} {...base} aria-hidden>
    <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
    <path d="M21 3v5h-5" />
    <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
    <path d="M8 16H3v5" />
  </svg>
);

/** Marks a `.note`: something the app decided and is telling you about. */
export const Info = ({ size = 15 }: { size?: number }) => (
  <svg width={size} height={size} {...base} aria-hidden>
    <circle cx="12" cy="12" r="10" />
    <path d="M12 16v-4" />
    <path d="M12 8h.01" />
  </svg>
);

/** Close a panel, drop a row. */
export const X = ({ size = 18 }: { size?: number }) => (
  <svg width={size} height={size} {...base} aria-hidden>
    <path d="M18 6 6 18" />
    <path d="m6 6 12 12" />
  </svg>
);

export const Plus = ({ size = 14 }: { size?: number }) => (
  <svg width={size} height={size} {...base} aria-hidden>
    <path d="M5 12h14" />
    <path d="M12 5v14" />
  </svg>
);
