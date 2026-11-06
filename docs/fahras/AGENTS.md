# Fahras fork instructions

Fahras is a fork of [Strata](https://github.com/lgse/strata) that adds Windows
support, Arabic, and its own features. These rules override `AGENTS.md` where
they conflict; everything else in `AGENTS.md` still applies.

## Owner decisions

- Owner: iBar8. Commits are authored as
  `iBar8 <340081494+iBar8@users.noreply.github.com>`.
- Never add a `Claude-Session:` link to commit messages. A `Co-Authored-By:`
  trailer is fine.
- Push finished work directly to `main`. No issue-first workflow or pull
  requests unless the owner asks for one.
- No automatic upstream sync. Sync only when the owner asks.
- Supported targets: Linux (as upstream) and Windows 10/11 x64. Linux must keep
  building; ARM64 Windows comes later.
- CodeScene (`scripts/code-health.sh`) is not available; skip it.

## Keep the diff against upstream small

Upstream changes weekly, and every line we change in an upstream file is a
future merge conflict.

- Put Fahras code in new files: Windows code under `src/platform/windows/`,
  Fahras features in their own modules, and Fahras assets under `branding/`.
- Touch upstream files only with small hooks, such as `#[cfg(windows)]` calls
  into our modules.
- Rebrand only what users see: display name, icons, app ID, update source,
  installer and links. Keep internal names (`strata` crate, `strata-*` CSS and
  assets, `@strata_*` colors) unchanged.
- Never reuse upstream's update signing key or release infrastructure.
- Dependabot is removed (`.github/dependabot.yml`); dependency updates arrive
  through upstream syncs. Keep it deleted when a sync touches it.
- Upstream's GitHub workflows are removed: they depend on Strata's secrets,
  servers and containers. Keep them deleted when a sync touches them. Fahras
  workflows use the `fahras-` filename prefix.

## Languages

Fahras supports fourteen languages: upstream's thirteen plus Arabic (`ar`).
Every new user-facing string needs all fourteen. Arabic is plain Modern
Standard Arabic (Fusha) with simple, common words and no dialect, and the
interface must work right to left.

## Updates and file types

- Only Windows updates itself, through the signed installer. Linux builds
  leave the in-app updater off and their Updates page points to the Fahras
  Releases page (`src/fahras/updates.rs`).
- File type names come from Windows itself, in the system language, as File
  Explorer shows them. Fahras only names types Windows has no description for
  (`src/fahras/file_types.rs`); do not translate Windows' own names.

## Releases

- Fahras uses Strata's version from `Cargo.toml`; release after an upstream
  sync brings a new version. Switch to Fahras's own numbers only when the owner
  wants to ship between syncs.
- To release, the owner creates the release on GitHub with tag `v<version>`
  from `main`. `fahras-windows.yml` builds, tests and signs the installer with
  the `FAHRAS_UPDATE_SIGNING_KEY` secret and attaches the files.
- `data/update-keys.json` holds the matching public key. Never replace it
  without the owner; installed copies only accept updates signed by it.

## Syncing with upstream

The `upstream` remote points at `https://github.com/lgse/strata` with pushes
disabled. When the owner asks for a sync:

```bash
git fetch upstream main
git merge upstream/main
```

Resolve conflicts by keeping upstream's change and re-applying ours on top.
For `README.md` and `LICENSE`, keep the Fahras version; for deleted upstream
workflows and Dependabot, keep them deleted. Build and test both
Linux and Windows before pushing, and summarise what upstream changed.
