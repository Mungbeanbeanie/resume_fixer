import { useCallback, useEffect, useState } from "react";
import { vault } from "../../ipc";
import type { Skill } from "../../types";
import { useVault } from "./vaultContext";

// Every row in the skills table, with the ones that reach a resume checked. Tagging a skill
// on a bullet or experience checks it, and the box stays the user's to uncheck.
export default function SkillsCard() {
  const { report } = useVault();
  const [skills, setSkills] = useState<Skill[]>([]);
  // The checked ones in the order they print, which the alphabetical list above cannot show.
  const [printOrder, setPrintOrder] = useState<string[]>([]);
  const [name, setName] = useState("");

  // Alphabetical, always. The backend hands these back in the order they print on the base
  // resume, which is the right order for a resume and the wrong one for a picker: two
  // hundred checkboxes are only findable by name, and a list that re-sorts under the cursor
  // when a box is ticked moves the next box out from under it.
  const load = useCallback(async () => {
    try {
      const rows = await vault.listSkills();
      setPrintOrder(rows.filter((s) => s.always_list).map((s) => s.name));
      setSkills([...rows].sort((a, b) => a.name.localeCompare(b.name)));
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
    <details className="experience" open>
      <summary>
        <strong>Skills</strong>
        <span className="muted" style={{ fontSize: 13 }}>
          {printed} of {skills.length} print on the base resume
        </span>
      </summary>
      <div className="body">
        <div className="tags">
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
        <div className="row" style={{ gap: "var(--space-2)" }}>
          <input
            placeholder="A skill you have but never wrote a bullet about"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && add()}
          />
          <button className="quiet" onClick={add} disabled={!name.trim()} style={{ flex: "none" }}>
            + add
          </button>
        </div>
        {printOrder.length > 0 && (
          <div style={{ fontSize: 13 }}>
            <span className="field-label">Prints in this order</span>
            {printOrder.join(", ")}
          </div>
        )}
        <span className="muted" style={{ fontSize: 12 }}>
          Unchecking never deletes a skill — it stays available for tagging and job-posting
          matching. Re-checking one sends it to the end of the line above, which is how a
          skill is moved.
        </span>
      </div>
    </details>
  );
}
