# Branch Strategy

- **`main`**: Production release branch. Must always remain in a stable state.
- **`dev`**: Development branch. Serves as the base branch for all PRs.
- All feature/bugfix branches are created from `dev`, and PRs are opened against `dev`.
- Merging `dev` → `main` is only performed when a release is ready, using the **Create Merge Commit** strategy.
- All PR merges other than `dev` → `main` use **Squash Merge**.
