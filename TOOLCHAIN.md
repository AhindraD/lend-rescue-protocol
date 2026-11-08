# LendRescue Toolchain & Dependency Policy (`TOOLCHAIN.md`)

*Documented as of: November 8, 2026*

This document defines the exact, verified toolchain, operating system requirements, package manager policies, and compatibility constraints for building, testing, and deploying LendRescue.

---

## 1. Verified Toolchain Baseline

Every component in the LendRescue mono-repo is pinned to explicit versions verified in this environment:

| Tool / Package | Verified Version | Channel / Source | Purpose |
|---|---|---|---|
| **Operating System** | `Ubuntu 26.04.1 LTS` | Resolute Raccoon (WSL2 Linux 6.18) | Development and CI runtime |
| **System GLIBC** | `2.43` | Ubuntu system package | Satisfies Agave/Solana $\ge 2.39$ prerequisite |
| **Node.js** | `26.10.0` | Active LTS line (managed by nvm) | Frontend & TypeScript runtime |
| **pnpm** | `12.18.0` | npm package (`packageManager` pinned) | Monorepo package manager |
| **Rust toolchain** | `1.99.1` | Stable (channel `stable` / `1.99.1`) | Pure Rust engine, API, contracts |
| **Cargo** | `1.99.1` | Cargo release 2026-08-27 | Rust build and dependency manager |
| **Anchor CLI** | `1.2.2` | AVM (`avm install 1.2.2`) | Solana smart contract framework |
| **Anchor Crates** | `1.2.2` | crates.io (`anchor-lang`, `anchor-spl`) | On-chain Anchor dependencies |
| **Anchor TS Client** | `1.2.2` | npm (`@anchor-lang/core`) | Anchor TypeScript integration client |
| **Solana / Agave CLI**| `2.1.0` | Agave client release | Solana CLI, keygen, test-validator |
| **Platform Tools** | `1.57` | Solana SBPF toolchain | BPF/SBPF compilation tools |
| **SBPF Architecture**| `v3` | Solana target specification | SBPF v3 instruction format |
| **Surfpool** | `1.7.0` | `/home/kibi/.local/bin/surfpool` | Local Mainnet fork & instruction simulation |
| **Next.js** | `16.4.5` | npm package | Frontend web console (`apps/web`) |
| **React / React DOM**| `19.3.1` | npm package | React UI framework |
| **TypeScript** | `5.9.3` | npm package | Anchor v1-compatible TypeScript line |
| **Tailwind CSS** | `4.3.5` | npm package | Styling & design token system |
| **`@solana/web3.js`**| `1.99.0` | npm package (Legacy v1 line) | Compatibility bridge for `@anchor-lang/core` |
| **`@solana/spl-token`**| `0.4.15` | npm package | SPL Token client library |
| **Agent Skill** | `frontend-design` | GitHub: `anthropics/skills` | Design lead skill in `.agents/skills/` |

---

## 2. Operating System & GLIBC Verification

Agave and Solana platform tools compile and link against modern Linux system libraries. A GLIBC version $\ge 2.39$ is strictly required to avoid dynamic linker symbol mismatches:

```bash
$ cat /etc/os-release | grep PRETTY_NAME
PRETTY_NAME="Ubuntu 26.04.1 LTS"

$ ldd --version | head -n 1
ldd (Ubuntu GLIBC 2.43-2ubuntu2.4) 2.43
```

Ubuntu 26.04 LTS ships GLIBC 2.43, fully satisfying the requirement without unsafe manual system library patching.

---

## 3. Solana & Anchor Compatibility Architecture

### 3.1 Anchor 1.2.2 and Solana CLI 2.1.0 (Agave)
Anchor 1.2.2 is the stable baseline for production Anchor v1 programs. It relies on:
- Solana / Agave CLI: `2.1.0`
- Platform Tools: `v1.57` (`1.95.0-sbpf-solana-v1.57`)
- SBPF architecture: `v3`
- Anchor CLI installed via AVM:
  ```bash
  avm install 1.2.2
  avm use 1.2.2
  ```

### 3.2 The `@solana/web3.js` v1 Compatibility Boundary
Anchor 1.2.2's official TypeScript client is `@anchor-lang/core` (published under npm `@anchor-lang/core`, replacing the legacy `@coral-xyz/anchor` namespace).

**Critical Constraint:**
`@anchor-lang/core` 1.2.2 is tightly coupled to `@solana/web3.js` **v1** (specifically `1.99.0`) and `@solana/spl-token` **0.4.x**. It is **incompatible** with `@solana/web3.js` v2/v3 / Solana Kit.

To preserve stability and avoid broken peer dependency graphs:
1. All Anchor integration test suites (`tests/anchor/*`) and IDL client interactions use:
   - `@anchor-lang/core`: `1.2.2`
   - `@solana/web3.js`: `1.99.0`
   - `@solana/spl-token`: `0.4.15`
   - `typescript`: `5.9.3`
2. Never float web3.js dependencies to `^2.0.0` or `^3.0.0` in the Anchor execution path.
3. If future non-Anchor client modules explore Solana Kit / web3.js v2, they must be isolated in independent sub-packages behind an explicit adapter boundary.

---

## 4. Surfpool Configuration

**Surfpool** provides local Mainnet fork simulation for testing read-only protocol decoders and instruction handlers without broadcasting transactions to the live network.

- **Binary Location**: `/home/kibi/.local/bin/surfpool` (symlinked into `/home/kibi/.cargo/bin/surfpool`)
- **Verified Version**: `surfpool 1.7.0`
- **Local RPC URL**: `http://127.0.0.1:8899`
- **Local WebSocket URL**: `ws://127.0.0.1:8900`
- **Command to Start Local Fork**:
  ```bash
  surfpool --url https://api.mainnet-beta.solana.com
  ```
- **Rules of Engagement**:
  - Results derived from Surfpool must be explicitly tagged `Surfpool Simulated`.
  - Never broadcast transactions simulated on Surfpool to Solana Mainnet.

---

## 5. Dependency Pinning & Locking Rules

1. **Exact Semver Only**:
   - `package.json` dependencies must be exact versions (e.g. `"@anchor-lang/core": "1.2.2"`, not `"^1.2.2"` or `"~1.2.2"`).
   - No wildcard (`*`), `latest`, or `next` ranges in production packages.
2. **Deterministic Lockfiles**:
   - Node packages: `pnpm-lock.yaml` (generated with `pnpm install`, enforced with `--frozen-lockfile` in CI).
   - Rust crates: `Cargo.lock` (enforced with `cargo build --locked`).
   - Agent skills: `skills-lock.json` (tracks computed SHA256 hashes of installed agent skills).
3. **Rust Workspace Isolation**:
   - Root `Cargo.toml` is a virtual workspace (`[workspace]`).
   - Workspace members are added only as their physical crates are created.
   - Pinned Rust compiler: `1.99.1` declared in `rust-toolchain.toml`.
4. **Node & Engine Strictness**:
   - Node version: `26.10.0` pinned in `.nvmrc` and `.node-version`.
   - `package.json` specifies `"packageManager": "pnpm@12.18.0"` and `"engines"`.
   - `.npmrc` enforces `engine-strict=true`.

---

## 6. Official Version and Compatibility References

- **Node.js Active LTS**: <https://nodejs.org/en/about/previous-releases>
- **Rust Stable Releases**: <https://blog.rust-lang.org/releases/latest/>
- **Anchor 1.2.2 Release Notes**: <https://www.anchor-lang.com/docs/updates/release-notes/1-2-2>
- **Anchor TypeScript Client**: <https://www.anchor-lang.com/docs/clients/typescript>
- **Agave / Solana Releases**: <https://github.com/anza-xyz/agave/releases>
- **Surfpool Documentation**: <https://surfpool.run>
- **Anthropic Agent Skills**: <https://github.com/anthropics/skills/tree/main/skills/frontend-design>

