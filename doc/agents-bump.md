# Agent procedure: bump all projects to a new version

Use this procedure to release one version across every repository in `Meta.toml`, update the intentionally retained lockfiles in `rust-bitvmx-client` and `rust-bitvmx-zk-proof` on the host with Cargo's native dependency cooldown, publish the non-client repositories first, then update and publish the client in a separate commit/tag round, merge the release into `dev`, and update the workspace's submodule pointers. The client retains its lockfile because it is the CLI application; the ZK repository retains its lockfile for RISC Zero reproducibility.

## Mandatory interaction and safety rules

1. Ask the user for the version (for example, `0.8.6`) and confirm that the release branch will have exactly the same name and the tag will be prefixed with `v`, producing `v0.8.6`.
2. Perform only one numbered phase at a time.
3. Before every mutating phase, show the exact commands that will run and ask for explicit confirmation.
4. After every phase, report what changed, show verification results, and ask whether to continue.
5. Stop immediately if any command fails, any repository reports an error, a merge conflict occurs, or verification differs between repositories. Explain the error and wait for instructions; never reset, discard, amend, force-push, or work around it without approval.
6. Treat output containing `Error`, `failed`, or a non-zero per-repository status as a failure even if `cargo meta` itself exits successfully. Some `cargo meta` operations print per-repository errors but still return exit code 0.
7. Never include unrelated changes in a commit. Report unrelated local changes, but they may be ignored without additional approval when they do not overlap with the targeted files or affect the requested commands. Never stage or commit them. Stop and ask for instructions if they overlap with the target or could affect the operation.
8. Never move or force-push a release tag. Tag each repository exactly once: tag and publish the non-client repositories after their commits are complete, then tag and publish `rust-bitvmx-client` after its lockfile commit is complete.
9. Run all `cargo meta` commands from the workspace root. Run lockfile updates on the host with the Rust 1.100 toolchain or newer so Cargo's native dependency cooldown is enforced.
10. Keep a list of every managed repository and verify each operation against that list. Do not assume success from the final exit code alone.
11. Treat `rust-meta` as workspace tooling, not as a managed repository. Exclude it from cross-repository operations unless the user explicitly requests otherwise.

Set and validate the requested version in the host shell:

```bash
export VERSION=0.8.6
printf 'Version: %s\nBranch: %s\nTag: v%s\n' "$VERSION" "$VERSION" "$VERSION"
```

Replace `0.8.6` with the version requested by the user. Reject an empty or malformed version before proceeding.

## Phase 1: preflight (read-only)

Inspect the workspace and every managed repository:

```bash
git status --short
git branch --show-current
git remote -v
test "$(git branch --show-current)" = main
test "$(git rev-parse HEAD)" = "$(git ls-remote origin refs/heads/main | awk '{print $1}')"
cargo meta exec 'git status --short'
cargo meta exec 'git branch --show-current'
cargo meta exec 'git remote -v'
cargo meta exec 'test "$(git branch --show-current)" = dev'
cargo meta exec 'test "$(git rev-parse HEAD)" = "$(git ls-remote origin refs/heads/dev | awk "{print \$1}")"'
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client|rust-bitvmx-zk-proof) test -f Cargo.lock ;; *) test ! -f Cargo.lock ;; esac'
rustc --version
cargo --version
```

Confirm that:

- the workspace and all managed repositories contain no unrelated changes;
- the top-level workspace is on `main`, while every managed repository is on `dev`, before the release branch is created;
- the workspace's local `main` exactly matches `origin/main`, and every managed repository's local `dev` exactly matches `origin/dev`;
- every repository has an `origin` remote;
- `Cargo.lock` exists only in `rust-bitvmx-client` (the CLI application) and `rust-bitvmx-zk-proof` (for RISC Zero reproducibility), and is absent from every other managed repository;
- the host uses the Rust 1.100 toolchain or newer, whose Cargo natively enforces the repositories' dependency cooldown configuration; and
- `VERSION`, the release branch, and `v$VERSION` are correct.

Do not create or switch to the release branch unless the workspace `main`/`origin/main` check and every managed repository `dev`/`origin/dev` check pass.

Report the findings. Do not continue from a dirty or inconsistent state without explicit user instructions.

## Phase 2: create and switch to the release branch

After confirmation, create or switch to the version branch in all managed repositories:

```bash
cargo meta branch "$VERSION"
```

Verify every repository:

```bash
cargo meta exec 'git branch --show-current'
cargo meta exec 'git status --short'
```

Every managed repository must be on `$VERSION`. Report the result and ask before continuing.

## Phase 3: bump crate versions

After confirmation, bump all crate versions and internal dependency references:

```bash
cargo meta bump "$VERSION"
```

Review the resulting changes:

```bash
cargo meta exec 'git status --short'
cargo meta exec 'git diff -- Cargo.toml'
```

Confirm that the intended `Cargo.toml` files have the requested version and that no unexpected files changed. Report the result and ask before continuing.

## Phase 4: commit the manifest version bump

After confirmation, commit the TOML changes with the meta tool:

```bash
cargo meta commit --message "chore: bump version to $VERSION"
```

Verify all repositories:

```bash
cargo meta exec 'git status --short'
cargo meta exec 'git log -1 --oneline'
```

`cargo meta commit` is for the manifest-related changes; lockfile changes are committed separately later. Report all resulting commit IDs and ask before continuing.

## Phase 5: update the ZK lockfile on the host

The client lockfile cannot be finalized until the other versioned crates have been published. After confirmation, update only the ZK lockfile before the first publish round:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-zk-proof) cargo update ;; *) printf "SKIP %s: not the ZK repository\n" "$PWD" ;; esac'
cargo meta exec 'git status --short -- Cargo.lock'
```

Use the Rust 1.100 toolchain or newer so `cargo update` natively applies the repository's dependency cooldown configuration. Do not update `rust-bitvmx-client/Cargo.lock` yet, and do not create `Cargo.lock` files in any other repository. Carefully inspect the complete output from every repository. A printed per-repository error is a failure even if the aggregate command exits with status 0.

Inspect the result:

```bash
cargo meta exec 'git status --short'
cargo meta exec 'git diff -- Cargo.lock'
```

Confirm that only `rust-bitvmx-zk-proof/Cargo.lock` may have changed and summarize its dependency changes. If it did not change, report that and ask whether the user wants to continue without a ZK lockfile commit.

## Phase 6: commit the updated ZK lockfile

After confirmation, stage only the ZK lockfile:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-zk-proof) git add -- Cargo.lock ;; *) printf "SKIP %s: not the ZK repository\n" "$PWD" ;; esac'
```

Verify the staged content before committing:

```bash
cargo meta exec 'git diff --cached --name-only'
cargo meta exec 'git diff --cached -- Cargo.lock'
```

Only `Cargo.lock` in `rust-bitvmx-zk-proof` may be staged. After reporting this verification and receiving confirmation, commit only there if the lockfile changed:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-zk-proof) git diff --cached --quiet || git commit -m "chore: update Cargo.lock for v$VERSION" ;; *) printf "SKIP %s: not the ZK repository\n" "$PWD" ;; esac'
```

Verify and report the result:

```bash
cargo meta exec 'git status --short'
cargo meta exec 'git log -1 --oneline'
```

Stop if any tracked change remains or the ZK lockfile commit fails.

## Phase 7: tag and publish every repository except the client

The first publish round makes the new internal crate versions available before resolving the client lockfile. After confirmation, create `v$VERSION` in every managed repository except `rust-bitvmx-client`:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) printf "SKIP %s: client is published later\n" "$PWD" ;; *) git tag "v$VERSION" ;; esac'
```

Verify that each non-client tag points to `HEAD` and that no client tag exists:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) test -z "$(git tag --list "v$VERSION")" ;; *) test "$(git rev-parse HEAD)" = "$(git rev-list -n 1 "v$VERSION")" ;; esac'
```

After reporting verification and receiving another confirmation, push the release branches and tags for every non-client repository:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) printf "SKIP %s: client is published later\n" "$PWD" ;; *) git push -u origin "$VERSION" && git push origin "v$VERSION" ;; esac'
```

Verify remote branch and tag state for every non-client repository, and verify that the client tag is still absent remotely:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) test -z "$(git ls-remote origin "refs/tags/v$VERSION")" ;; *) git fetch origin --quiet && test "$(git rev-parse HEAD)" = "$(git rev-parse "origin/$VERSION")" && test "$(git rev-parse HEAD)" = "$(git ls-remote origin "refs/tags/v$VERSION" | awk "{print \$1}")" ;; esac'
```

Report the result. Do not continue until the newly published internal crates are available to Cargo and satisfy the configured native cooldown.

## Phase 8: update the client lockfile

After confirmation that the non-client packages are available and eligible under Cargo's native cooldown, update only the client lockfile:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) cargo update ;; *) printf "SKIP %s: not the client repository\n" "$PWD" ;; esac'
cargo meta exec 'git status --short -- Cargo.lock'
```

Inspect the client lockfile change:

```bash
cargo meta exec 'git status --short'
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) git diff -- Cargo.lock ;; *) : ;; esac'
```

Confirm that the client lockfile resolves the intended newly published internal crate versions. If those versions are not selected, stop; do not tag or publish the client. If the client lockfile did not change, report that and ask whether the user wants to continue without a client lockfile commit.

## Phase 9: commit the updated client lockfile

After confirmation, stage and verify only the client lockfile:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) git add -- Cargo.lock ;; *) printf "SKIP %s: not the client repository\n" "$PWD" ;; esac'
cargo meta exec 'git diff --cached --name-only'
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) git diff --cached -- Cargo.lock ;; *) : ;; esac'
```

After reporting verification and receiving confirmation, commit only in the client if its lockfile changed:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) git diff --cached --quiet || git commit -m "chore: update Cargo.lock for v$VERSION" ;; *) printf "SKIP %s: not the client repository\n" "$PWD" ;; esac'
```

Verify the client commit and clean state:

```bash
cargo meta exec 'git status --short'
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) git log -1 --oneline ;; *) : ;; esac'
```

Stop if any client change remains or the commit fails.

## Phase 10: tag and publish the client

After confirmation, create the client's release tag at its final lockfile commit:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) git tag "v$VERSION" ;; *) printf "SKIP %s: already published\n" "$PWD" ;; esac'
```

Verify the client tag points to `HEAD`. After reporting and receiving another confirmation, push the client release branch and tag:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) test "$(git rev-parse HEAD)" = "$(git rev-list -n 1 "v$VERSION")" ;; *) : ;; esac'
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) git push -u origin "$VERSION" && git push origin "v$VERSION" ;; *) printf "SKIP %s: already published\n" "$PWD" ;; esac'
```

Verify the client's remote branch and tag:

```bash
cargo meta exec 'case "$(basename "$PWD")" in rust-bitvmx-client) git fetch origin --quiet && test "$(git rev-parse HEAD)" = "$(git rev-parse "origin/$VERSION")" && test "$(git rev-parse HEAD)" = "$(git ls-remote origin "refs/tags/v$VERSION" | awk "{print \$1}")" ;; *) : ;; esac'
```

Report the result and ask before merging into `dev`.

## Phase 11: merge the release branch into `dev`

After confirmation, check out `dev` everywhere:

```bash
cargo meta checkout dev
```

Verify every repository is on `dev` and clean:

```bash
cargo meta exec 'git branch --show-current'
cargo meta exec 'git status --short'
```

After reporting and receiving confirmation, merge the release branch:

```bash
cargo meta merge "$VERSION"
```

Stop on any conflict. Otherwise inspect and verify the merge result:

```bash
cargo meta exec 'git status --short'
cargo meta exec 'git log --oneline --decorate -3'
cargo meta exec 'git merge-base --is-ancestor "$VERSION" HEAD'
```

After reporting and receiving confirmation, push `dev` in all repositories:

```bash
cargo meta push
```

Verify that local and remote `dev` match:

```bash
cargo meta exec 'git fetch origin --quiet && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/dev)"'
```

Report the result and ask before updating the top-level workspace.

## Phase 12: commit updated submodule pointers in the workspace

This phase uses plain Git in the top-level workspace, not `cargo meta`.

Inspect the workspace changes:

```bash
git status --short
git diff --submodule=short
```

Confirm that the changes consist only of the expected managed submodule pointers moving to the newly pushed `dev` commits. Do not stage unrelated files or untracked files.

Show the exact list of submodule paths that will be staged and ask for confirmation. Then stage those paths explicitly (replace the placeholder with the reviewed list):

```bash
git add <CHANGED_SUBMODULE_PATHS>
git diff --cached --submodule=short
```

Report the staged diff and ask for confirmation before committing:

```bash
git commit -m "chore: update workspace to v$VERSION"
```

Verify the workspace status and commit:

```bash
git status --short
git log -1 --oneline
```

After reporting and receiving final push confirmation, push the current workspace branch:

```bash
git push origin HEAD
```

Verify the push succeeded and report the final workspace commit, all release branches, and tag `v$VERSION`.

## Final report

Provide a concise release summary containing:

- version, release branch, and tag;
- manifest bump commits by repository;
- the ZK lockfile commit and first non-client publish round;
- the client lockfile commit and separate client publish round, including either repository if its lockfile did not change;
- confirmation that every release branch and final tag was pushed exactly once;
- confirmation that the release branch was merged and pushed to `dev` everywhere;
- the top-level workspace commit and pushed branch; and
- any warnings or deviations explicitly approved by the user.

## Utility workflow: targeted changes across repositories

Use this independent workflow when the same targeted operation must be performed in all managed repositories, such as modifying one known file, running a command, staging that file, committing it, and pushing the commits. Do not use the release phases above unless the operation is part of a release.

### Scope and inspection rules

- Prefer targeted inspection. Read the requested file, relevant nearby lines, and directly related configuration only; do not scan or read the complete source tree unless the task cannot be completed safely without doing so.
- Use `cargo meta exec` without `--crate-dir` when the command must run once at the root of each unique repository. This is the normal choice for a file with the same repository-relative path.
- Use `cargo meta exec --crate-dir` only when the command must run separately in every crate listed in `Meta.toml`. A repository containing multiple crates will receive the command multiple times.
- Use exact relative paths and stage exact files. Never use `git add .`, `git add -A`, or a broad search-and-replace.
- First determine whether the target exists everywhere and whether its current contents are compatible with the requested edit. Repositories may legitimately differ; stop and report differences rather than forcing a uniform edit blindly.
- Ignore `rust-meta` unless the user explicitly includes it in the requested scope.
- Unrelated local changes that do not overlap with the target or affect the command may remain in place, but must not be staged or committed.
- Continue to follow the mandatory confirmation and error-handling rules at the start of this document. In particular, inspect all output because `cargo meta exec` can print an error for one repository while returning an overall successful exit code.

### Utility phase A: establish scope (read-only)

Ask for and repeat back:

- the exact repository-relative file path;
- the requested transformation;
- whether the command should run once per repository or once per crate;
- the command to run after editing, if any;
- the commit message; and
- whether the resulting commits should be pushed and to which branch.

Set exported variables when useful so that the child shells started by `cargo meta exec` inherit them:

```bash
export FILE='path/to/file'
export COMMIT_MESSAGE='chore: describe the targeted change'
```

Check workspace and repository state without reading unrelated code:

```bash
git status --short
cargo meta exec 'git status --short'
cargo meta exec 'printf "%s: " "$PWD"; git branch --show-current'
cargo meta exec 'if test -f "$FILE"; then printf "FOUND %s/%s\n" "$PWD" "$FILE"; else printf "MISSING %s/%s\n" "$PWD" "$FILE"; fi'
```

If this is a per-crate operation, use the following existence check instead:

```bash
cargo meta exec --crate-dir 'if test -f "$FILE"; then printf "FOUND %s/%s\n" "$PWD" "$FILE"; else printf "MISSING %s/%s\n" "$PWD" "$FILE"; fi'
```

Use targeted commands such as `grep`, `rg`, or a bounded file read to inspect only the required file and matching lines. Report missing files, dirty repositories, branch differences, and content differences. Agree with the user whether missing files should be skipped, created, or treated as an error before modifying anything.

### Utility phase B: apply the targeted edit

Show the exact edit command and ask for confirmation. Run it once per repository by default:

```bash
cargo meta exec '<TARGETED_EDIT_COMMAND_USING_THE_EXACT_FILE_PATH>'
```

For a true per-crate operation only:

```bash
cargo meta exec --crate-dir '<TARGETED_EDIT_COMMAND_USING_THE_EXACT_FILE_PATH>'
```

The edit command must fail if its expected old content is absent or ambiguous. It must not silently append duplicate content or rewrite unrelated formatting. If shell quoting would be fragile, create a small, reviewable temporary script, show it to the user, and invoke that script with the exact target path instead of constructing a complex one-liner.

Immediately inspect only the target and repository status:

```bash
cargo meta exec 'git status --short'
cargo meta exec 'git diff -- "$FILE"'
```

Confirm that only intended files changed and report repositories where no change occurred. Stop on unexpected changes or partial application.

### Utility phase C: run targeted validation or another command

Show the exact command and ask for confirmation. Choose the correct execution mode:

```bash
# Once per unique repository root
cargo meta exec '<VALIDATION_OR_OTHER_COMMAND>'

# OR, only when required, once per configured crate
cargo meta exec --crate-dir '<VALIDATION_OR_OTHER_COMMAND>'
```

Do not substitute a workspace-wide build or broad code inspection when a focused check is sufficient. Capture and report the result for every repository or crate. Stop if any output reports an error, even when the top-level `cargo meta` process exits successfully.

If the command itself may modify files, run `cargo meta exec 'git status --short'` afterward and obtain approval for every additional change before staging it.

### Utility phase D: stage and commit only the target

Show the staging command and ask for confirmation:

```bash
cargo meta exec 'git add -- "$FILE"'
cargo meta exec 'git diff --cached --name-only'
cargo meta exec 'git diff --cached -- "$FILE"'
```

Verify that each staged path is exactly the approved target. Repositories with no change should have no staged content. After reporting the staged diffs and receiving confirmation, commit only where something is staged:

```bash
cargo meta exec 'git diff --cached --quiet || git commit -m "$COMMIT_MESSAGE"'
```

Verify every result:

```bash
cargo meta exec 'git status --short'
cargo meta exec 'git log -1 --oneline'
```

Report commit IDs per repository and explicitly list repositories that required no commit. Stop if any intended change remains uncommitted or any unrelated change is present.

### Utility phase E: push when requested

Confirm the current branch and upstream state first:

```bash
cargo meta exec 'printf "%s: " "$PWD"; git branch --show-current'
cargo meta exec 'git status --short'
```

Show the push command and ask for explicit confirmation:

```bash
cargo meta push
```

Do not force-push. Verify local `HEAD` against the corresponding remote branch in every repository, then report the pushed branches and commits. If top-level submodule pointers changed, do not commit them automatically; ask whether the user wants a separate workspace pointer commit and follow the targeted staging rules above.
