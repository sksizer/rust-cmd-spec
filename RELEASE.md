# Releasing

Releases are **fully automated** by [`release-plz`](https://release-plz.dev/)
running in CI. You do **not** run any release command locally — you merge a PR
that a bot prepares for you.

## The flow, end to end

```
  merge feature PRs to main
            │
            ▼
  ┌───────────────────────────────────────────────┐
  │  release-plz CI (on every push to main)         │
  │  opens/updates a "chore: release" PR:           │
  │    • bumps the crate version                    │
  │    • regenerates CHANGELOG.md (git-cliff)        │
  └───────────────────────────────────────────────┘
            │
            ▼
  you review + merge the "chore: release" PR
            │
            ▼
  ┌───────────────────────────────────────────────┐
  │  release-plz CI (on that merge):                │
  │    • git tag (v{version})                       │
  │    • publish to crates.io                        │
  │    • create a GitHub Release                     │
  └───────────────────────────────────────────────┘
```

So the only human steps are: **merge feature work**, then **merge the release
PR** when you're ready to cut a release. Everything else is the bot.

## Where it's configured

| File | What it controls |
|------|------------------|
| [`.github/workflows/release-plz.yml`](./.github/workflows/release-plz.yml) | The CI job: triggers on push to `main`, runs the `release-plz` action. |
| [`release-plz.toml`](./release-plz.toml) | Release behavior: publish on/off, semver gate, GitHub releases. |
| [`cliff.toml`](./cliff.toml) | Changelog format and how commits are grouped. |

Key `release-plz.toml` settings:

- `publish = true` — publishes to crates.io when the release PR merges.
- `git_release_enable = true` — creates a GitHub Release per release.
- `semver_check = true` — runs `cargo-semver-checks` before publishing.
- `changelog_config = "cliff.toml"` — reuse the git-cliff config for changelogs.
- `pr_name = "chore: release"` — the title of the automated release PR.

If the project should **not** be published to crates.io, set `publish = false`
in `release-plz.toml` (and `publish = false` in `Cargo.toml` to match) — you
keep the automated version bumps, changelog, tags, and GitHub Releases.

## How the version is decided

release-plz reads **Conventional Commits** since the last release tag and picks
the bump automatically. Pre-1.0 crates follow 0.x SemVer — the **minor** slot is
the breaking tier:

| Commit type | Bump (0.x) |
|-------------|-----------|
| `fix:` | patch |
| `feat:` | minor |
| `feat!:` / `fix!:` / `BREAKING CHANGE:` | minor (the 0.x breaking tier) |

You influence the release by how you write commit messages — there is no manual
version bump. `cargo-semver-checks` runs as a gate before publishing and will
block a bump that's too small for the API delta.

> Merge commits (`Merge pull request #NNN …`) are non-conventional and are
> filtered out of the changelog by `cliff.toml`; the underlying `feat:`/`fix:`
> commits on the branches are what appear.

## Prerequisites (one-time setup per project)

1. **`CARGO_REGISTRY_TOKEN` repo secret** — a crates.io API token with publish
   rights. Needed only at the *publish* step (when a release PR merges), not to
   open the PR.
   - Set via: `gh secret set CARGO_REGISTRY_TOKEN` (paste a crates.io token), or
   - Prefer **crates.io Trusted Publishing (OIDC)** — no long-lived token; see
     the release-plz docs.
2. **"Allow GitHub Actions to create and approve pull requests"** must be enabled
   (Settings → Actions → General → Workflow permissions), or via CLI:

   ```sh
   gh api -X PUT repos/<owner>/<repo>/actions/permissions/workflow \
     -f default_workflow_permissions=read -F can_approve_pull_request_reviews=true
   ```

   Without it, release-plz cannot open the release PR. The workflow already
   grants the job `contents: write` and `pull-requests: write`.

> Optional but recommended: have release-plz open the PR with a **PAT or GitHub
> App token** instead of the default `GITHUB_TOKEN`. PRs opened by the default
> token do **not** trigger other workflows (like `ci.yml`), so the release PR
> won't get CI runs otherwise.

## Common gotcha: "working directory has uncommitted changes"

release-plz refuses to run if any file is **both git-tracked and matched by a
committed `.gitignore`** — it sees them as uncommitted noise. Symptom in the CI
log:

```
ERROR failed to update packages
  1: the working directory of this project has uncommitted changes. If these
     files are both committed and in .gitignore, either delete them or remove
     them from .gitignore.
```

Find offenders and untrack them (they stay on disk):

```sh
git ls-files --cached --ignored --exclude-standard
git rm --cached <each file>
```

## Manual escape hatch

There is no local release recipe — release-plz owns releases. If you ever must
publish by hand (registry outage, emergency), do it deliberately and outside
this pipeline:

```sh
cargo semver-checks --baseline-rev "$(git describe --tags --abbrev=0)"  # just semver-check
cargo publish                                                            # then a real publish
```

...and reconcile the tag/changelog afterward so release-plz's next run agrees
with reality.
