# ADR 0011: Publish the web edition through GitHub Pages

- **Status:** Accepted
- **Date:** 2026-08-29

## Context

DGX Lab already compiles its Leptos client-rendered interface and deterministic Rust simulation runtime into a static HTML, CSS, JavaScript, and WebAssembly artifact. The application does not require an API, database, real shell, scheduler connection, or external network service after its assets load.

The Tauri shell remains useful for desktop distribution, but requiring installation creates needless friction for public, self-paced training. A separate JavaScript implementation would duplicate the simulator and invite behavioral drift.

GitHub project Pages hosts the repository beneath a repository-name path rather than at the domain root. The existing Trunk build uses root-relative asset URLs, so the public path must be supplied during the Pages build.

## Decision

1. Publish the browser edition as a public GitHub Pages project site.
2. Build the Pages artifact from `crates/web-ui` on every push to `main`; do not publish the checked-in `dist/` snapshot directly.
3. Compile with a repository-derived public base path and validate every generated asset reference before upload.
4. Run the existing source, test, and forbidden-capability checks in the deployment workflow.
5. Keep Tauri as the desktop distribution over the same Rust simulation source.
6. Display the independent-product disclaimer and browser-local persistence notice in the static entry point.
7. Treat the in-browser assessment as educational readiness evidence, not a tamper-resistant credential.

## Consequences

- The web and desktop editions retain one simulation implementation and one content model.
- Public hosting requires no runtime server or operational credentials.
- Learner progress remains origin-scoped browser storage and does not synchronize across browsers or devices.
- Moving to a custom domain creates a new storage origin unless progress is explicitly exported and imported.
- Authenticated learners, protected question banks, centralized completion records, or authoritative certificates would require a separate service and a new architectural decision.
- Repository administrators must select **GitHub Actions** as the Pages source once before the first production deployment.

## Rejected alternatives

- **Rewrite the simulator in JavaScript:** unnecessary duplication with a high drift risk.
- **Publish checked-in build output:** permits stale or untested artifacts to become production.
- **Add a backend solely for hosting:** adds state, credentials, maintenance, and failure modes without supporting a current requirement.
