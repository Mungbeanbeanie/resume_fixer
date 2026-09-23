// The bullet standard, shown once for the whole vault rather than on every row.
//
// These examples add facts the weak version never stated — the 30%, the 2,000 employees.
// That is exactly what the model is forbidden to do, and exactly what you are here for:
// you were there, so you know the number. Improve only surfaces what a bullet already says.
const PAIRS: [string, string][] = [
  [
    "Worked on backend services for banking applications",
    "Designed Java-based RESTful services supporting high-volume banking transactions, improving request latency by 25%",
  ],
  [
    "Wrote Java code and fixed bugs",
    "Diagnosed and resolved production defects using logs and monitoring tools, reducing customer-reported issues by 30%",
  ],
  [
    "Built UI screens for internal applications",
    "Developed responsive Angular UI components for internal risk-management tools used by 2,000+ employees",
  ],
  [
    "Used Angular and HTML",
    "Improved page load performance by 40% by optimizing Angular change detection and reducing API payload size",
  ],
  [
    "Growing programming skills in JavaScript and web application development",
    "Developed foundational skills in JavaScript and web application development, applying best practices for clean, maintainable code",
  ],
  [
    "Assisted customers with location and product enquiries",
    "Assisted customers with product and location inquiries, delivering timely and accurate information to enhance overall customer experience",
  ],
];

export default function BulletStandard() {
  return (
    <details className="disclosure quiet">
      <summary>How to write a strong bullet</summary>
      <div className="col" style={{ marginTop: "var(--space-3)" }}>
        <span className="muted" style={{ fontSize: 12 }}>
          Action verb → what was built → technology → measurable outcome.
        </span>
        {PAIRS.map(([weak, strong]) => (
          <div className="pair" key={weak}>
            <div className="weak">
              <span className="eyebrow">Weak</span>
              {weak}
            </div>
            <div className="strong">
              <span className="eyebrow">Strong</span>
              {strong}
            </div>
          </div>
        ))}
        <span className="muted" style={{ fontSize: 12 }}>
          The numbers have to come from you. Improve only re-words what a bullet already
          says — it cannot invent the 25%, so write it in yourself.
        </span>
      </div>
    </details>
  );
}
