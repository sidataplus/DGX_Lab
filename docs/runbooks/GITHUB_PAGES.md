# GitHub Pages public web distribution

## Scope

The public web edition is the release build from `crates/web-ui`. GitHub Pages serves the generated HTML, CSS, JavaScript, and WebAssembly files. The deterministic simulator, virtual shell and filesystem, course state, grading, and assessment all execute in the browser.

There is no production API, database, real scheduler connection, telemetry service, or host command execution.

## One-time repository setup

After this change is merged, a repository administrator must complete the GitHub Pages setup once:

1. Open **Settings → Pages**.
2. Under **Build and deployment**, select **GitHub Actions** as the source.
3. Confirm that the `github-pages` environment permits deployment from `main`.
4. Treat the deployed site as public even if the source repository remains private.

The workflow cannot safely assume that repository visibility restricts the published site.

## Build and deployment behavior

`.github/workflows/pages.yml` has two paths:

- Pull requests targeting `main` run the complete static build and artifact validation but do not upload or deploy.
- Pushes to `main` and manual runs on `main` build, validate, upload, and deploy the Pages artifact.

The workflow derives the default project-site path from `GITHUB_REPOSITORY`. For this repository, the expected base path is `/DGX_Lab/`. `scripts/validate_pages_dist.py` rejects root-hosted asset URLs, missing files, external entry-point URLs, symlinks, oversized artifacts, and missing public-release notices.

## Local reproduction

Install the repository's normal Rust/WASM prerequisites and pinned Trunk version, then run:

```bash
make web-pages PAGES_BASE=/DGX_Lab/
```

For ordinary root-hosted local inspection:

```bash
make web-release
cd crates/web-ui/dist
python3 -m http.server 1421 --bind 127.0.0.1
```

A project-path build can be exercised locally by serving it beneath the same path:

```bash
rm -rf /tmp/dgxlab-pages
mkdir -p /tmp/dgxlab-pages/DGX_Lab
cp -R crates/web-ui/dist/. /tmp/dgxlab-pages/DGX_Lab/
python3 -m http.server 1421 --bind 127.0.0.1 --directory /tmp/dgxlab-pages
```

Then open `/DGX_Lab/` on the local server.

## Trust and persistence boundary

- Simulation commands never leave the local runtime and never reach a host shell or real scheduler.
- Web-edition progress is stored in browser-local storage for the current origin.
- Clearing site data, changing browsers, changing devices, or moving to a custom domain produces a separate learner state.
- Course content, answer keys, scoring logic, and thresholds are delivered to the browser and are therefore inspectable. The assessment is suitable for learning feedback and local readiness evidence, not secure certification.

## Custom-domain migration

The initial workflow deliberately targets the default GitHub project-site path. Before enabling a custom domain, change the build base to `/`, validate the resulting artifact, and communicate the browser-storage origin change. Do not serve one build artifact interchangeably at both base paths.

## Rollback

Revert the offending commit on `main`. The next successful workflow run replaces the deployed artifact. If a new build fails validation, the deploy job does not run and the last successful Pages deployment remains in place.
