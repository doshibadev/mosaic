# Mosaic v1.0.0 Public Release Roadmap

## Phase 1: Stability & Security (The "Don't Break" Phase)

- [x] **Secure Authentication (CLI)**
  - **Goal:** Stop storing tokens in plain text `auth.toml`.
  - **Action:** Integrate the `keyring` crate to store tokens in the OS secure credential manager.
  - **Why:** Essential security practice for CLI tools.

- [x] **Smart Packaging (.mosaicignore)**
  - **Goal:** Allow users to exclude files from their package (e.g., secrets, large assets).
  - **Action:** Implement a parser for `.mosaicignore` (similar to `.gitignore`) in the `publish` command.
  - **Why:** Prevents accidental leakage of sensitive data and reduces package size.

- [x] **Registry Database & Storage Hardening**
  - **Goal:** Ensure data persistence and security.
  - **Action:** Audit `Dockerfile` for production volumes. Verify S3 bucket permissions (Public Read / Private Write).
  - **Status:** Migrated from SurrealDB to Neon (PostgreSQL) and optimized Dockerfile.
  - **Why:** Prevents data loss and unauthorized file tampering.

- [x] **User-Friendly Error Handling**
  - **Goal:** Replace raw rust panics/stack traces with readable error messages.
  - **Action:** Refactor CLI error handling to catch specific errors (Network, Auth, Config) and display clean `Logger::error` messages.
  - **Why:** Improves perceived quality and usability.

## Phase 2: User Experience (The "Ease of Use" Phase)

- [x] **Website README Rendering**
  - **Goal:** Display project documentation on the package page.
  - **Action:**
    1. Modify Registry: Extract `README.md` from the uploaded ZIP during `publish`.
    2. Modify DB: Store the README content in the `package` or `package_version` table.
    3. Modify Website: Fetch and render this Markdown on the package detail page.
  - **Why:** Critical for users to understand how to use a package.

- [ ] **Search & Discovery Polish**
  - **Goal:** Make it easy to find packages.
  - **Action:** Add "Recently Updated" and "Most Downloaded" sorting to the API. Display these lists on the website homepage.
  - **Why:** Encourages exploration and ecosystem growth.

- [ ] **SemVer Resolution (Optional for v1.0)**
  - **Goal:** Support version ranges (e.g., `^1.0.0`).
  - **Action:** Implement semver matching logic in the CLI `install` command.
  - **Why:** Standard expectation for package managers, though exact matching is acceptable for MVP.

## Phase 3: Documentation (The "Adoption" Phase)

- [x] **The "Mosaic Book" (Documentation Site)**
  - **Goal:** Centralized knowledge base.
  - **Action:** Create a sub-site (e.g., `docs.getmosaic.run`) with:
    - Getting Started / Installation
    - "Create your first library" tutorial
    - CLI Command Reference
    - `mosaic.toml` Configuration Reference
  - **Status:** Implemented Nextra docs in `/docs`.
  - **Why:** Users cannot use what they don't understand.

- [x] **One-Line Install Script**
  - **Goal:** Frictionless installation.
  - **Action:** Create `install.ps1` (Windows) and `install.sh` (Mac/Linux) to download the latest binary from GitHub Releases and add to PATH.
  - **Why:** Lowers the barrier to entry significantly.

## Phase 4: Release Logistics (The "Launch" Phase)

- [x] **CI/CD Pipeline (GitHub Actions)**
  - **Goal:** Automated building and releasing.
  - **Action:** Configure workflow to build binaries for Windows/Linux/macOS on tag push and create a GitHub Release.
  - **Status:** Created `.github/workflows/release.yml` with draft releases and multi-platform build matrix.
  - **Why:** Ensures consistent, clean builds for users.

- [ ] **Legal & Social**
  - **Goal:** Basic compliance and community presence.
  - **Action:** Add `TERMS.md` and `PRIVACY.md`. Set up Discord/Twitter.
  - **Why:** Professionalism and community building.
