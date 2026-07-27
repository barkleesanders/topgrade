# Fork triage — `barkleesanders/topgrade` vs `topgrade-rs/topgrade`

Snapshot: **2026-07-26**. Fork at `15d29a0`, upstream/main 27 commits ahead (v17.8.0 tagged 2026-07-15).
Divergence: **141 local non-merge commits**; fork has **184 steps** vs upstream's **174**.

Purpose: make each future sync cheaper by (a) upstreaming what upstream would take,
(b) dropping what upstream has since implemented itself, (c) naming what must be carried forever.

## Authorship reality

| Author | Commits | Note |
|---|---:|---|
| Barklees Sanders / Barklee Sanders | 114 | mix of original work + re-implementations of upstream *issues* |
| GideonBear | 16 | upstream contributor's work carried here |
| Rafael Scalet | 3 | |
| renovate[bot] | 2 | |
| David Knaack, cloudy | 2 each | |
| Thorsten Vitt, Prajwal K | 1 each | |

This fork is a deliberate **mass-integration of pending upstream PRs/issues**, per the
documented fork-integration strategy — not 141 pieces of original work. Triage must respect that.

---

## A. Upstreamable — Barklee-original, absent upstream (PR candidates)

### A1. Implementations of upstream issues that are STILL OPEN (highest value — upstream asked for these)

| Commit | Feature | Upstream issue | Issue state | In upstream? |
|---|---|---|---|---|
| `e957127` | differentiate exit codes, failures vs fatal | [#398](https://github.com/topgrade-rs/topgrade/issues/398) | **open** | no |
| `0fcb76c` | pipe nix commands through `nom` when available | [#719](https://github.com/topgrade-rs/topgrade/issues/719) | **open** | no (verified: no `nix-output-monitor` in upstream `src/`) |
| `35810cc` | `--show-step-ids` flag | [#1467](https://github.com/topgrade-rs/topgrade/issues/1467) | **open** | no |
| `e1a7b9e` | GitHub token for self-update (rate limits) | [#1474](https://github.com/topgrade-rs/topgrade/issues/1474) | **open** | no |
| `28c4897` | expose env vars to custom commands | [#1013](https://github.com/topgrade-rs/topgrade/issues/1013) | closed | no (upstream only has `TOPGRADE_SKIP_BRKC_NOTIFY`) |

### A2. New steps, Barklee-authored, absent upstream

| Commit | Step(s) | Note |
|---|---|---|
| `0760f8d` | `HardwareIds` | |
| `3db4a20` | `Ldcup` | ldc2 version manager |
| `31aaca3` | `Micromamba`, `MicrosoftAutoUpdate`, `Msys2`, `NixFlake` | **split into 4 PRs** — upstream reviews one step per PR |
| `8eb3375` | `NhClean` | bundled with CLI/UX changes; needs splitting |
| `1ff4488` | `Sdio` | Windows driver updates |
| `a3e8171` | `YtDlp` | |

### A3. Standalone fixes/features worth upstreaming

| Commit | Change |
|---|---|
| `91a0dfb` | `[npm] exclude` option to skip packages in global update |
| `15d29a0` | resolve OpenCode target version via `gh api` to avoid 403 rate limit |
| `bb873c7` | skip `uv cache prune` when another uv process holds the cache lock |

---

## B. NOT ours to upstream — other contributors' work carried here

| Step / area | Author |
|---|---|
| `Adless` | Prajwal K |
| `Plasmoids`, `PlasmoidsSystem` | cloudy |
| `UvPython` | GideonBear (16 commits total in this fork) |

Their PRs are upstream's to merge. Carrying them is a deliberate cost, not a contribution backlog.

---

## C. Converged — upstream implemented it independently → DROP local version on merge

Verified present in `upstream/main`:

| Feature | Local ref | Upstream evidence |
|---|---|---|
| Soar package manager | `a84e31a` (#1377) | upstream `feat(step): add Soar package manager (#2212)` |
| install-release step | `dfc7d64` (#1789) | in upstream `src/step.rs` |
| Colima step | `a73b042` (#1798) | in upstream `src/step.rs` |
| Sparkle fix | `54e39e2` (#1517) | upstream `fix(sparkle): correct CLI argument usage (#1912)` |
| `gup_exclude` | `6d681a3` (#604) | upstream `src/config.rs:224` |
| doas `--setenv` | `4b93a70` (#1435) | upstream `src/sudo.rs:486` |
| `merge` → `merge2` | `95ccb6d` (Mar) | upstream `refactor: migrate from merge to merge2 (#2211)` |

Each of these is a **guaranteed conflict** until dropped. Taking upstream's side on merge is correct.

Also note: fork **removed** `NixHelper` (`de4ff54`) while upstream still ships it → divergence in `step.rs`.

---

## D. Permanent dead weight — fork-only merge repair, never upstreamable

`76c7c19`, `1db26fe`, `eba4652`, `d3211a3`, `c43ea3f`, `4585381`, `671d424`, `be6ce6e`
("Fix merge" ×3, restore-fork-APIs-after-merge ×3, regenerate Cargo.lock, resolve config.rs conflict)

These exist only because the fork diverged. Every sync manufactures more. **This is the cost curve
that upstreaming A1–A3 is meant to bend.**

---

## Ranked PR plan

Upstream `CONTRIBUTING.md` requires: conventional-commit PR **titles**, and `cargo fmt` +
`cargo clippy` + `cargo test` green. It also explicitly **forbids LLM-generated PR descriptions**
and "vibe-coded" whole PRs — so descriptions must be written by hand.

| # | PR | Commits | Why it lands |
|---|---|---|---|
| 1 | `feat: support GitHub token for self-update to avoid rate limiting` | `e1a7b9e` | open issue #1474, self-contained, no new step |
| 2 | `feat: add --show-step-ids flag` | `35810cc` | open issue #1467, small surface |
| 3 | `feat(nix): pipe nix commands through nom when available` | `0fcb76c` | open issue #719 |
| 4 | `feat: differentiate exit codes for failures vs fatal errors` | `e957127` | open issue #398; behaviour change — discuss on the issue first |
| 5 | `feat(npm): add exclude option for global updates` | `91a0dfb` | small, matches existing `gup_exclude` precedent |
| 6 | `fix(uv): skip cache prune when another uv process holds the cache` | `bb873c7` | pure bug fix |
| 7 | `fix(opencode): resolve target version via gh api to avoid 403` | `15d29a0` | pure bug fix |
| 8+ | one PR per new step | `a3e8171`, `3db4a20`, `1ff4488`, `0760f8d`, split `31aaca3` | follow the "Adding a new step" checklist in CONTRIBUTING |

Start with 1–2: smallest diff, open issues, no new `Step` variant (so no `default_steps` /
i18n-locale CI churn). Each accepted PR permanently removes a conflict source.

## Sync procedure that keeps working

1. `git fetch upstream && git merge-tree --write-tree HEAD upstream/main` → conflict preview, no working-tree change.
2. On conflicts in Category C, **take upstream's side**.
3. After merge: `cargo fmt && cargo clippy && cargo test`, then rebuild the installed binary
   (`cargo install --path .` — the local `topgrade` on PATH is built from this repo, not a release).
4. Re-check the `Step` enum diff (fork-only vs upstream-only) and update this file.
