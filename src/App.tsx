import { useCallback, useEffect, useState } from "react";
import { health } from "./ipc";
import type { HealthReport } from "./types";
import GenerateTab from "./tabs/generate/GenerateTab";
import LibraryTab from "./tabs/library/LibraryTab";
import VaultTab from "./tabs/vault/VaultTab";

const TABS = ["Generate", "Library", "Vault"] as const;
type Tab = (typeof TABS)[number];

function HealthStrip({ report }: { report: HealthReport | null }) {
  if (!report) return <span className="muted">checking…</span>;
  const items: [string, boolean][] = [
    ["Postgres", report.postgres],
    ["Ollama", report.ollama],
    [report.model, report.model_present],
    ["Tectonic", report.tectonic],
  ];
  return (
    <div className="health">
      {items.map(([label, ok]) => (
        <span key={label} className={`pill ${ok ? "on" : "off"}`} title={ok ? "ready" : "not reachable"}>
          {label}
        </span>
      ))}
    </div>
  );
}

export default function App() {
  const [tab, setTab] = useState<Tab>("Generate");
  const [report, setReport] = useState<HealthReport | null>(null);

  const refreshHealth = useCallback(() => {
    health.check().then(setReport).catch(() => setReport(null));
  }, []);

  useEffect(refreshHealth, [refreshHealth]);

  // ⌘1/2/3 switch tabs. Generate owns ⌘Enter itself.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!e.metaKey && !e.ctrlKey) return;
      const i = ["1", "2", "3"].indexOf(e.key);
      if (i >= 0) {
        e.preventDefault();
        setTab(TABS[i]);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <div className="app">
      <nav className="tabs">
        <span className="brand">Resume Fixer</span>
        {TABS.map((t) => (
          <button key={t} aria-current={t === tab} onClick={() => setTab(t)}>
            {t}
          </button>
        ))}
        <span className="spacer" />
        <button className="quiet" onClick={refreshHealth} title="Re-check local services">
          ↻
        </button>
        <HealthStrip report={report} />
      </nav>
      <main className="page">
        <div className="page-inner">
          {tab === "Generate" && <GenerateTab health={report} />}
          {tab === "Library" && <LibraryTab />}
          {tab === "Vault" && <VaultTab />}
        </div>
      </main>
    </div>
  );
}
