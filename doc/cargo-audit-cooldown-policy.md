# Cargo audit and cooldown remediation policy

Apply this process to every Rust repository in this workspace:

1. Run `cargo audit` and identify error-level RUSTSEC advisories.
2. Check the publication date of each applicable advisory against the 14-day dependency cooldown.
3. If the advisory is older than 14 days, update the affected dependency directly under the normal Cargo publish-age policy.
4. If the advisory is between 7 and 14 days old, attempt a narrowly targeted precise update while temporarily allowing recent versions:

   ```sh
   cargo update -p <crate> --precise <version> --config 'resolver.incompatible-publish-age="allow"'
   ```

   Review the complete `Cargo.lock` diff because the temporary allowance applies to the entire resolution operation, including transitive dependencies.
5. If the advisory is newer than 7 days, pause and request manual review before updating it.
6. If remediation is impossible because the vulnerable dependency is constrained by an upstream dependency outside our control, add a narrowly scoped advisory exception to the repository's Cargo audit configuration.
7. Create a report covering every repository, advisory, update, exception, failure, and remaining finding.
8. Keep normal builds and CI locked with `--locked`.
