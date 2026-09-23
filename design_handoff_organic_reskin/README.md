# Handoff: Resume Fixer — Organic reskin

## Overview
A visual reskin of the Resume Fixer desktop app (Tauri + React + TS, `src/`) in the **Organic** style: a warm cream-and-sand ground, terracotta accent, sage second accent, Caprasimo display headings over Figtree body text, pill-shaped controls and large rounded containers.

**This is a reskin. Layout, flows, copy, state and IPC stay as they are.** Every tab (Generate, Library, Vault, Base) keeps its structure; only styling and a few wrapper elements change.

## About the design files
`reference/Resume Fixer.html` is a **design reference built in HTML**. It shows the intended look and behavior. It is not production code. Recreate it inside the existing React codebase using its current patterns (plain CSS in `src/styles/tokens.css` + `src/App.css`, class names in the TSX).

To view it: open `reference/Resume Fixer.html` in a browser (it needs the `reference/_ds/` and `reference/support.js` files next to it). Use ⌘1–⌘4 to switch tabs. Clicking a Library row opens the detail panel. Clicking Generate runs a fake 2.6-second generation into the draft view.

## Fidelity
**High fidelity.** Colors, type, radii, spacing and states are final. Match them exactly.

## How to implement (recommended order)
1. **Replace `src/styles/tokens.css`** with `src/styles/tokens.css` from this folder. All existing variable names (`--bg`, `--surface`, `--border`, `--text`, `--muted`, `--accent`, `--accent-soft`, `--danger`, `--applied`…`--rejected`, `--radius`, `--gap`, `--font`, `--mono`) are kept and re-pointed, so inline styles in TSX keep working.
2. **Bundle the fonts locally** (the app is local-first): `npm i @fontsource/caprasimo @fontsource/figtree`, import Caprasimo 400 and Figtree 400/600/700 in `main.tsx`, then delete the Google Fonts `@import` at the top of tokens.css.
3. **Replace `src/App.css`** with `src/App.css` from this folder. Every existing class is kept. Classes marked `NEW` in the file need small TSX changes (listed below).
4. Make the TSX changes in "Markup changes" below.
5. Remove leftover hard-coded colors in TSX inline styles (`#fff`, `#fdf6f5`, etc.) and replace them with tokens.
6. Compare each tab against the reference side by side.

## Design tokens

### Color
| Token | Hex | Use |
|---|---|---|
| `--color-bg` | #f5ead8 | page ground |
| `--color-surface` | #ebddc5 | cards, panels, disclosures, default input fill |
| `--color-text` | #201e1d | text |
| `--color-accent` | #c67139 | primary buttons, focus ring, error dot, "Offer" bar |
| `--color-accent-2` | #7a8a5e | sage: success/ready, "reworded", printed skills |
| `--color-divider` | #201e1d @ 16% | borders on secondary buttons and inputs |

Ramps (100 → 900):
- neutral: #f9f4ed #eee7db #dcd3c4 #c0b6a5 #a19786 #82796a #645c50 #474238 #2e2b25
- accent: #fff2eb #ffe1d0 #ffc6a5 #f6a06b #d67f48 #b2622d #8c491a #643312 #402310
- accent-2: #f0fae1 #e1eecc #ccdbb2 #aebf92 #8fa073 #728157 #56633f #3d472b #272e1b

Rules:
- Light steps (100–300) are for tinted fills. Dark steps (700–900) are for text on those fills.
- Paragraph-size text in the accent must use `--color-accent-700` (#8c491a), never the base accent (contrast).
- Muted text is `--color-neutral-700` (#645c50).
- Do not use greys or pure white anywhere. The resume "paper" preview uses `--color-neutral-100`.

### Type
- Display: **Caprasimo 400**. Used for headings, the brand, company titles, big numbers (29%), disclosure titles ("9 bullets printed"), and the label of every real action button (`.btn`: Generate, Download, Save vault…).
- Body: **Figtree** 400/600/700. Used for body text, tab labels, inputs, quiet/link-style buttons, and tags.
- Sizes: body 14px/1.55. Brand 21px (top bar) / 22px (sidebar). Generate empty-state h2 36px. Draft/panel company 28px / 26px. Section headings (Vault, Base) 22px. Disclosure title 17px. Meta 13px. Hints and labels 12px. Eyebrow 11px uppercase with .08em tracking. Tags 11px.
- Headings use letter-spacing −0.015em and line-height 1.12.

### Spacing
The scale has 1.1× density baked in: `--space-1` 4.4, `-2` 8.8, `-3` 13.2, `-4` 17.6, `-6` 26.4, `-8` 35.2 px.

### Radius
- Buttons, inputs, selects, tags and chips: **999px** (pill).
- Textareas: 16px (`--radius-md`).
- Cards, disclosures, experience cards, role blocks, notes: 28px (`--radius-lg`).
- Large hero containers (PDF preview frame, funnel card, detail panel): 32px (`--radius-xl`).
- The paper preview itself: 6px.

### Shadow
- `--shadow-md` `0 3px 10px #2e2b25 @16%`: resume paper.
- `--shadow-lg` `0 12px 32px #2e2b25 @22%`: detail panel.
- Nothing else has a shadow. Containers separate by fill color, not borders.

### Icons
Use Lucide at stroke-width **2.75**. Icons in use: sparkles (Generate), library (Library), vault (Vault), file-text (Base), refresh-cw (re-check services), x (close/remove link), info (notes), plus, chevron-down, check. `lucide-react` works: `<RefreshCw size={15} strokeWidth={2.75} />`.

## Components and states
- **Primary button** (`button.primary`): accent fill, text in `--color-bg`. Hover accent-600, active accent-700.
- **Secondary** (default `button`): transparent with a 1px divider border. Hover is text @7%, active is text @14%.
- **Quiet** (`button.quiet`): Figtree 13px in accent-700, no border. Hover is accent @10%, active is accent @18%. `button.quiet.subtle` uses neutral-700 text and is for Remove/Delete-type actions.
- **Disabled**: opacity .45, `not-allowed` cursor.
- **Focus**: `:focus-visible` 2px accent outline with 2px offset, on every control. Inputs change their border to accent instead.
- **Input**: surface fill, divider border, pill shape, 14px left/right padding, min-height 36px. Hover border is text @45%. Inside a surface container the fill becomes neutral-100. Inside a role block it becomes `--color-bg` (the CSS handles this automatically).
- **Tabs**: transparent. Hover accent-100. Active (`aria-current="true"`) is accent-200 fill with accent-900 text.
- **Health pills**: ready is accent-2-100 / accent-2-800 with a sage-500 dot. Missing is accent-100 / accent-800 with an accent dot.
- **Status tags** (Library): interview*/offer are sage (`.tag.good`). applied/OA are terracotta (`.tag.active`). rejected/saved/withdrawn are neutral (`.tag.neutral`).
- **Disclosures**: `<details>` with the native marker hidden and a CSS chevron that rotates 180° when open (.15s ease). Two looks: filled (surface card) and `.quiet` (no fill, 13px muted summary, chevron on the left).

## Screens

### App shell / Navigation
There are two variants. **Top bar is the default.** The sidebar is optional; build it only if you want it.
- **Top bar (`.tabs`)**: no border, no background, padding 17.6px × 26.4px. Order: brand "Resume Fixer" → 4 tab buttons → spacer → ↻ refresh icon button (32px, ghost) → health pills (Postgres, Ollama, <model>, Tectonic).
- **Sidebar (`.app.sidebar` + `.sidenav`)**: 232px wide, full height. Brand at the top. Vertical nav buttons with an icon, a label, and a right-aligned `⌘1`–`⌘4` hint (11px neutral-600). Pinned to the bottom is a "Local services" card (surface, 28px radius) with a list of services: a dot, the name, and a right-aligned "ready"/"missing" label.
- **Page**: `.page` padding 26.4px 26.4px 72px. Content max-width 940px, centered.

### Generate: empty
- Column `.generate-empty`, max-width 620px, 12vh top margin, **left-aligned** (it was centered before).
- h2 "Paste a job link" in Caprasimo 36px.
- Row with a URL input (46px tall, 15px text, 20px padding) and a primary **Generate** button (26px side padding, 15px).
- Quiet button "or paste the description" toggles a textarea (28px radius, min-height 240px).
- While busy: `.status-line` becomes a surface card with a 10px sage dot and the status copy. The Tectonic warning is appended to the same line.
- Idle hint below: "⌘↵ to generate · everything stays on this machine" (12px muted). This is new copy. Drop it if unwanted.

### Generate: draft ready
- Header row (wraps): `.title-edit` (company in Caprasimo 28px, role 14px muted, both borderless pills that show a divider border on hover). Then: Download (secondary), Regenerate (secondary, toggles a feedback input + primary "Go" row), **Applied — save** (primary), Save only (secondary), Discard (quiet).
- `.note`: a sage info note with an info icon ("Left off to reach one page: …"). `.note.warn` (terracotta) is for the pending-edits warning.
- `.preview-frame`: surface, 32px radius, 26.4px padding, centered. The iframe inside has neutral-100 fill, 6px radius and `--shadow-md`. Below it, a quiet button "Open in the system viewer".
- DraftBullets inside a filled `.disclosure` (open by default). Summary: title "N bullets printed" (Caprasimo 17px) plus meta "3 reworded, 1 dropped to fit, 2 more available" (13px muted).
  - Each row (`.draft-bullet`): org label 12px muted + an optional sage "reworded" tag, then a textarea, then a right-hand quiet.subtle "Remove".
  - Spare bullets (`.draft-bullet.spare`, opacity .62) show text in a dashed neutral-400 box with a quiet "Add" button.
  - Footer: primary "Apply changes" (disabled until dirty) + 12px muted note.
- Rejected rewrites use `.disclosure.quiet`. Each item shows the attempted text with the reason below it (12px muted).

### Library
- `.funnel-card` (surface, 32px radius, 26.4px padding) wraps `<Funnel>`:
  - headline "29%" (Caprasimo 40px) + "response rate · 14 sent" (muted).
  - Rows as a grid `88px | 1fr | 72px`: label, a track (18px pill, neutral-200) with a bar filled in the stage color, then a count ("7 · 50%", tabular numbers, count in 600 weight).
  - Stage colors: Applied neutral-400, OA accent-2-400, Interview accent-2-600, Offer accent.
  - Footer: "5 rejected · 1 withdrawn · 2 saved, not sent" (12px muted).
- "Track an application I already sent": a filled `.disclosure`. The summary has a 26px accent-100 circle with a plus icon. The form uses `.grid-4` (auto-fit, min 160px), a file input, and a Notes textarea, with the "Track it" button in secondary style.
- `table.apps`: sits directly on the page ground. The header is 11px uppercase and tracked. Rows have 12px vertical padding, the company is bold, the Status column uses tags, and PDF/link are accent-700 13px quiet buttons. Hover is text @4%.

### Library: detail panel
- `.panel` floats with a 12px inset from the top/right/bottom, width min(520px, 90vw), surface fill, 32px radius, `--shadow-lg`, 26.4px padding, laid out as a column with 17.6px gaps.
- Top: company (Caprasimo 26px) + role (muted), with a ghost X icon button on the right.
- A status select (max 220px) + an "Open PDF" secondary button.
- "History" eyebrow, then `.history-row`s (sage dot, status, timestamp right-aligned muted).
- Meta line "1 page · <model> · prompts vN" (12px muted).
- "Job description" eyebrow + `.job-text` (neutral-100, 16px radius, no border).
- Bottom (margin-top auto): quiet "Delete this application".
- Optional: add a backdrop of neutral-900 @18% behind the panel that closes it on click.

### Vault
- Top row: the muted helper sentence on the left and a primary **Save vault** on the right. After saving, the confirmation shows as 13px accent-2-800 text.
- `.vault-section` headings are now Caprasimo 22px sentence-case (not uppercase), with no rule and 26.4px top margin. The count pill sits beside them in neutral-100.
- Skills: a filled disclosure. Chips are `.pill.skill-toggle`. Off: transparent with a divider border and a 14px empty ring. On: accent-2-200 fill, accent-2-900 text, and the ring filled accent-2-600 with a check. The add-skill row is an input + quiet "+ add" (nowrap). "Prints in this order" is a 12px label above a 13px list, followed by the 12px muted explainer.
- Bullet standard: `.disclosure.quiet` "How to write a strong bullet". `.pair` grids: Weak (neutral-200, eyebrow neutral-700) | Strong (accent-2-100, eyebrow accent-2-700). 16px radius.
- Header info: a filled disclosure, summary "Jordan Avery · email · N links". Fields sit in an auto-fit grid. Link rows use a grid `160px | 1fr | 36px` (label, URL, ghost X). Quiet "+ link".
- GPA and Interests: plain `.card`s.
- `.experience` card: surface, 28px radius, no border.
  - Summary: org (600), a kind tag (neutral), meta (13px muted), an optional "always printed" (sage) or "hidden" (terracotta) tag, and a chevron on the right.
  - Body: field grids (auto-fit). Use `.field-hint` for the explanations under Link / Link text.
  - Each `.role` is a neutral-100 block (28px radius, 17.6px padding) holding the title/date grid, the date-override hint, and bullets.
  - Each `.bullet` has a textarea (bg = page ground) with a borderless sage `.tag-input` under it. On the right is a stacked column of quiet "Improve" (accent) and quiet.subtle "Delete".
  - Role actions: quiet "+ bullet", quiet.subtle "delete role".
  - Card footer: secondary "+ role", "Hide from resumes", "Always print this", then a spacer and quiet "Delete experience".

### Base
- Top row: a Template select (min-width 260px) with a label, then a spacer, primary "Build base resume" (label becomes "Compiling…" while busy), and secondary "Use for generated resumes".
- "Edit or paste a template": `.disclosure.quiet` → `.template-editor` (name input + primary Save template, then a monospace 12px textarea on neutral-100 with 28px radius, then a 12px muted note about the Tera variables).
- After a build: a meta line + "Name this base resume" input + primary Save. Then a filled disclosure "N bullets printed" (same DraftBullets editor as Generate). Then `.preview-frame`.
- "Saved base resumes" (22px heading): each `.saved-base` row is a surface **pill** (padding 8 10 8 22) with the name (600), meta (12px muted, flex 1), and quiet actions Open / Reopen / Download plus quiet.subtle Delete.

## Markup changes (TSX)
Only the class names below are new. Everything else is CSS-only.
- `App.tsx`: give the refresh button `className="refresh"`. For the sidebar variant, render `.sidenav` instead of `.tabs` and add `sidebar` to `.app`.
- Generate: wrap the preview iframe in `<div className="preview-frame">`. Use `.note` / `.note.warn` for the info and pending banners. Use `.hint` for the ⌘↵ line. Add `.title` / `.meta` spans in disclosure summaries. Add `className="quiet"` to the rejected-rewrites disclosure.
- `DraftBullets.tsx`: `.draft-bullet` rows (`.body`, `.org`, `.spare`, `.text`).
- Library: wrap `<Funnel>` in `.funnel-card` and use the `.funnel-*` classes. Use `.history-row` in the panel and `.eyebrow` on section labels. Status tags use `.tag.good|active|neutral`.
- Vault: the skill chip checkbox lives inside `.skill-toggle` (it is restyled as a ring). Use `.pair` / `.weak` / `.strong` in `BulletStandard.tsx`, `.tag-input` and `.actions` in `BulletRow.tsx`, `.field-hint` for helper text, and `.quiet` on the bullet guide disclosure.
- Base: `.saved-base` rows, `.template-editor` wrapper, `.disclosure.quiet` on the template editor.
- Use `button.quiet.subtle` for Remove / Delete bullet / delete role / Delete (saved base).

## Interactions and behavior
All behavior stays as it is today: the ⌘1–⌘4 tab shortcuts, ⌘↵ to generate, save-on-blur fields, the busy/disabled states and IPC calls. The only motion added is the chevron rotation (transform, .15s ease). No other animation.

## Files in this bundle
- `src/styles/tokens.css`: drop-in replacement.
- `src/App.css`: drop-in replacement.
- `reference/Resume Fixer.html`: the interactive HTML design reference (all 4 tabs, both nav variants via the `nav` prop, the draft state via `draftReady`).
- `reference/_ds/…/styles.css`: the source Organic stylesheet the reference uses (for exact values).
