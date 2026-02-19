# Development Workflow

- All features and bug fixes must be tracked via **GitHub Issues** before starting work
- All code changes must go through **Pull Requests** linked to the corresponding issue
- Workflow: Create Issue → Create branch from `dev` → Implement → Open PR to `dev` → Squash Merge → Delete branch (local + remote)
- Release workflow: When `dev` is ready for release → Create PR from `dev` → `main` → Create Merge Commit
- Branch naming: `feat/<short-description>`, `fix/<short-description>`, etc.
- Commit frequently: after each meaningful unit of work, commit on the current branch before moving to the next task

## Commit Splitting Guide

When a task involves a large number of changes, do not put everything into a single commit. Instead, **split commits into reviewable units**.

### Principles

- Each commit should contain **one logical unit of change**
- Reviewers should be able to follow the changes commit by commit
- Each commit should independently pass build/test

### Splitting Criteria

- **Separate structural changes from logic changes**: Structural changes such as file moves and renames should be committed separately first
- **Separate refactoring from feature additions**: When cleaning up existing code before adding a new feature, split into a refactoring commit and a feature commit
- **Separate test additions**: New test code should be included with the corresponding implementation, but test coverage improvements for existing code can be split into separate commits
- **Separate configuration/dependency changes**: Changes to `Cargo.toml`, `package.json`, config files, etc. should be committed separately

### Example

If a feature addition involves the following steps:

1. `refactor(parser): extract helper function for reuse` - Refactor existing code
2. `feat(parser): add support for dynamic imports` - Implement new feature + tests
3. `chore: add new dependency for dynamic import analysis` - Add dependency

Split each step into a separate commit so reviewers can easily follow the flow of changes.

## Issue Writing Guide

An issue is the process of **defining a problem**. It should clearly describe the problem and outline possible solutions.

- The title should be written in **natural language describing the objective**, not in conventional commit format
  - Good: `Implement async napi function support`
  - Bad: `feat(napi): add async function`
- The body should include:
  - **Problem definition**: The gap between the current state and the desired state
  - **Possible solutions**: Available approaches and their trade-offs

### Severity Labels

Every issue must be assigned a **severity label** upon creation. Choose one based on the importance and urgency of the issue:

- **`severity: low`** (green)
  - Code quality improvements, documentation, minor refactoring
  - No impact on project progress if not addressed immediately

- **`severity: medium`** (yellow)
  - Feature enhancements, general refactoring, performance optimization, CI/CD improvements
  - Important but not urgent

- **`severity: high`** (orange)
  - Bugs affecting user experience, core feature implementation, serious performance issues
  - Needs to be resolved soon

- **`severity: urgent`** (red)
  - Critical bugs, security issues
  - Requires immediate resolution

## PR Writing Guide

A PR is the process of **solving a defined problem using a chosen approach**. It describes the implementation based on the problem and solution defined in the issue.

- The title must follow **conventional commit format** (used as the commit message on Squash Merge)
  - Good: `feat(napi): add async function support`
  - Bad: `Implement async napi function support`
- The body should include:
  - **Summary**: What problem was solved and how
  - **Changes**: Key implementation details
  - A link to the corresponding issue (`Closes #N`)
