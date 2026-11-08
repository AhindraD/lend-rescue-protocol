# LendRescue Agent Operating Instructions (`AGENTS.md`)

> **Notice to all coding agents:** You are an engineering agent assisting in the implementation of **LendRescue**. You must adhere to the phased execution protocol, git hygiene rules, and verification standards defined herein.

---

## 1. Mandatory Phase-Gate Discipline

1. **One Phase at a Time**: Implement only the single phase explicitly authorized by the user. Do not begin subsequent phases automatically.
2. **Phase Plan First**: Before making edits, state a concise phase plan and enumerate the files to be modified or created.
3. **No Unfinished Stubs**: Do not add stubbed, empty, or placeholder code for future phases just to make the current phase appear complete.
4. **Mandatory Verification**: Run every verification command specified in the current phase gate. Never claim a test or build passed without observing an actual zero exit code and expected output.
5. **Clean Commit & Stop**:
   - Inspect `git diff --check`, `git diff --stat`, and staged changes.
   - Stage only intended files.
   - Commit with a standard conventional commit message.
   - Print the exact required phase report and **stop execution**.

---

## 2. Git Hygiene Rules

- **Zero Destructive Actions**: Never run `git reset --hard`, `git clean -fd`, `git checkout -- .`, or force-push (`git push --force`).
- **Never Amend Unless Requested**: Never amend a phase commit unless explicitly instructed by the user.
- **Never Commit Secrets**: Keep `.env`, private keys (`id.json`, `*.key`), RPC credentials, and wallet tokens out of version control.
- **Keep Build Artifacts Out**: Ensure `node_modules`, `target`, `.anchor`, `.surfpool`, and build caches remain ignored.
- **Commit Toolchain & Lockfiles**: Maintain `Cargo.lock`, `pnpm-lock.yaml`, and `skills-lock.json` in sync with manifests.

---

## 3. Mandatory Pre-Implementation Reading

Before implementing any phase, read:
1. `SPEC.md` — Product scope, functional requirements, and acceptance criteria.
2. `ARCHITECTURE.md` — Trust boundaries, component roles, and data flows.
3. `USER_STORIES.md` — User expectations, happy paths, and adversarial failure criteria.
4. `TOOLCHAIN.md` — Pinned compiler and package versions, compatibility rules, and client boundaries.

---

## 4. Design Skill Directive for Frontend Work

When implementing frontend screens (Phase 6):
- Follow the instructions in `.agents/skills/frontend-design/SKILL.md`.
- Act as an editorial design lead: create distinctive typography, deliberate palettes (porcelain, graphite, safety amber, deep teal), and clear numerical hierarchy.
- Avoid generic AI-generated aesthetics (purple gradients, generic SaaS cards, unnecessary uppercase badges).
- Ensure the Three.js 3D hero is client-only, lazy-loaded, DPR-capped (1.5 max), paused on scroll, and paired with an accessible reduced-motion fallback.

---

## 5. Required Phase Report Format

At the completion of each phase, output this exact block:

```text
PHASE: <number and title>
STATUS: COMPLETE | BLOCKED

IMPLEMENTED:
- <bulleted summary of delivered functionality>

FILES CHANGED:
- <list of modified/created files>

VERIFICATION:
- <command> — PASS/FAIL (<summary of real observed output>)

COMMIT:
- hash: <short SHA or NONE>
- message: <commit message or NONE>
- working tree: CLEAN | NOT CLEAN

KNOWN LIMITATIONS:
- <explicit list of limitations or deferred items>

NEXT PHASE:
- <name of next phase — DO NOT START AUTOMATICALLY>
```

If a verification gate fails, set `STATUS: BLOCKED`, do not commit broken work, and explain the exact failure and required remediation.

