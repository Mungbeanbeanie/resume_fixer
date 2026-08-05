import { useCallback, useEffect, useState } from "react";
import { vault } from "../../ipc";
import type { Skill } from "../../types";
import { useVault } from "./vaultContext";

// Every row in the skills table, with the ones that reach a resume checked. Tagging a skill
// on a bullet or experience checks it, and the box stays the user's to uncheck.
export default function SkillsCard() {
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

  const printed = skills.filter((s) => s.always_list).length;

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
          {skills.map((s) => (
            <label key={s.id} className={`pill skill-toggle${s.always_list ? " on" : ""}`}>
              <input
                type="checkbox"
                checked={s.always_list}
                onChange={(e) => setListed(s, e.target.checked)}
              />
              {s.name}
            </label>
          ))}
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
