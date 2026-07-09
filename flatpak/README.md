# Flatpak / Flathub packaging

This directory contains everything needed to build Readfence as a Flatpak and
submit it to [Flathub](https://flathub.org).

| File | Purpose |
|---|---|
| `com.readfence.Readfence.yaml` | Flatpak build manifest |
| `com.readfence.Readfence.metainfo.xml` | AppStream metadata (store listing) |
| `com.readfence.Readfence.desktop` | Desktop entry + Markdown file association |
| `cargo-sources.json` | Every crate from `Cargo.lock`, vendored for the **offline** Flathub build |

The application ID is **`com.readfence.Readfence`** (based on the
`readfence.com` domain you control). The icon is installed from `assets/icon.png`.

---

## 1. Regenerating `cargo-sources.json`

Flathub builds have **no network access**, so all dependencies are vendored.
Regenerate this file whenever `Cargo.lock` changes:

```sh
# one-time setup
python3 -m venv /tmp/fcg && /tmp/fcg/bin/pip install aiohttp tomlkit
curl -sSLO https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py

# from the repo root
/tmp/fcg/bin/python flatpak-cargo-generator.py Cargo.lock -o flatpak/cargo-sources.json
```

---

## 2. Test the build locally

Install the builder and (optionally) lint, then build and run:

```sh
flatpak install flathub org.flatpak.Builder

# build + install into the user scope, then run
flatpak run org.flatpak.Builder --force-clean --user --install \
    build-dir flatpak/com.readfence.Readfence.yaml
flatpak run com.readfence.Readfence

# Flathub's linters (must pass before submitting)
flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
    manifest flatpak/com.readfence.Readfence.yaml
flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
    appstream flatpak/com.readfence.Readfence.metainfo.xml
```

> **Fast local iteration (before tagging a release):** the manifest points at
> the `0.3.4` git tag on GitHub. To build from your local working tree instead,
> temporarily replace the `type: git` source block with:
> ```yaml
>       - type: dir
>         path: ..
> ```
> Revert it before submitting (Flathub requires a pinned `git` source).

---

## 3. Cut the 0.3.4 release

The manifest builds from tag `0.3.4`, which must contain these packaging files
and the argv/file-open changes. Commit everything, then:

```sh
git tag 0.3.4
git push origin main --tags        # (or your default branch)

git rev-parse 0.3.4                 # copy this commit SHA...
```

Paste that SHA into `com.readfence.Readfence.yaml` in place of
`REPLACE_WITH_COMMIT_SHA_OF_TAG_0.3.4`.

---

## 4. Submit to Flathub

1. **Fork** [`github.com/flathub/flathub`](https://github.com/flathub/flathub).
   Leave *"Copy the master branch only"* **unchecked**.
2. Clone the `new-pr` branch of your fork and create a submission branch:
   ```sh
   git clone --branch=new-pr git@github.com:<your-user>/flathub.git
   cd flathub
   git checkout -b com.readfence.Readfence new-pr
   ```
3. Copy the manifest **and** `cargo-sources.json` to the **top level** of the
   flathub repo (Flathub expects the manifest at the repo root):
   ```sh
   cp /home/adam/projects/readfence/flatpak/com.readfence.Readfence.yaml .
   cp /home/adam/projects/readfence/flatpak/cargo-sources.json .
   git add com.readfence.Readfence.yaml cargo-sources.json
   git commit -m "Add com.readfence.Readfence"
   git push -u origin com.readfence.Readfence
   ```
   (The `.desktop`, `.metainfo.xml`, and icon are installed from your app repo
   by the manifest, so they don't need to be copied here.)
4. Open a **pull request against the `new-pr` branch** of `flathub/flathub`,
   titled `Add com.readfence.Readfence`.
5. Comment **`bot, build`** on the PR to trigger a test build, and check the
   result. A reviewer will look it over — respond to any requested changes.
6. When merged, Flathub creates a `flathub/com.readfence.Readfence` repo and
   invites you as a maintainer. **Enable 2FA** on GitHub and accept within a
   week. The app publishes ~1–2 hours after merge.

---

## 5. After it's live

- **Updating for new releases:** in the `flathub/com.readfence.Readfence` repo,
  bump the `tag`/`commit` in the manifest, regenerate & commit
  `cargo-sources.json`, add a `<release>` entry to the metainfo, and open a PR.
- **Verified badge:** because you control `readfence.com`, you can verify
  ownership at <https://flathub.org/apps/com.readfence.Readfence> to get the
  blue "verified" checkmark.
