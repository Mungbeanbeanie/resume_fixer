import { useCallback, useEffect, useState } from "react";
import { vault } from "../../ipc";
import type { ExperienceDetail, Skill } from "../../types";
import { useVault } from "./vaultContext";

// Every row in the skills table, with the ones that reach the base resume checked. A skill
// tagged on a bullet or experience always prints, so its box is checked and locked; the rest
// are the user's own call.
export default function SkillsCard({ experiences }: { experiences: ExperienceDetail[] }) {
  const { report } = useVault();
  const [skills, setSkills] = useState<Skill[]>([]);
  const [name, setName] = useState("");

  const load = useCallback(async () => {
    try {
      setSkills(await vault.listSkills());
    } catch (e) {
      report(e);
    }
  }, [report]);

  useEffect(() => {
    void load();
  }, [load]);

  // The same rule as services/base.rs::vault_skills, from data the tab already holds.
  const tagged = new Set(
    experiences
      .flatMap((e) => [...e.skills, ...e.roles.flatMap((r) => r.bullets.flatMap((b) => b.skills))])
      .map((s) => s.slug),
  );

  const printed = skills.filter((s) => s.always_list || tagged.has(s.slug)).length;

  async function setListed(skill: Skill, listed: boolean) {
    try {
      await vault.setSkillListed(skill.id, listed);
      await load();
    } catch (e) {
      report(e);
    }
  }

  async function add() {
    if (!name.trim()) return;
    try {
      await vault.addSkill(name.trim());
      setName("");
      await load();
    } catch (e) {
      report(e);
    }
  }

  return (
    <details className="experience">
      <summary>
        <strong>Skills</strong>
        <span className="muted" style={{ fontSize: 12 }}>
          {printed} of {skills.length} print on the base resume
        </span>
      </summary>
      <div className="body">
        <div className="tags" style={{ marginTop: 12 }}>
          {skills.map((s) => {
            const inUse = tagged.has(s.slug);
            return (
              <label
                key={s.id}
                className="skill-toggle"
                title={inUse ? "Tagged on a bullet or experience — always printed" : undefined}
              >
                <input
                  type="checkbox"
                  checked={inUse || s.always_list}
                  disabled={inUse}
                  onChange={(e) => setListed(s, e.target.checked)}
                />
                {s.name}
                {inUse && <span className="muted">· in use</span>}
              </label>
            );
          })}
        </div>
        <div className="row" style={{ marginTop: 12 }}>
          <input
            placeholder="A skill you have but never wrote a bullet about"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && add()}
          />
          <button className="quiet" onClick={add} disabled={!name.trim()}>
            + add
          </button>
        </div>
        <span className="muted" style={{ fontSize: 12 }}>
          Unchecking never deletes a skill — it stays available for tagging and job-posting
          matching.
        </span>
      </div>
    </details>
  );
}
