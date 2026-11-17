# Workspace Cargo audit report — 2026-11-17

## Summary

All 17 Rust repositories were audited with `cargo-audit 0.22.2` against a refreshed RustSec advisory database.

Final result: **all repositories pass `cargo audit` with zero non-ignored error-level vulnerabilities**.

- 11 lockfiles updated
- 10 repositories updated from `libz-sys 1.1.31` to `1.1.33`
- `rust-meta` updated from `rustls 0.23.37` to `0.23.46` and from `rustls-webpki 0.103.10` to `0.103.16`
- Scoped audit exceptions added where patched versions are incompatible with upstream dependency requirements
- 54 informational warnings remain across the workspace, primarily unmaintained-crate notices; these were outside this error-level remediation pass

## Updated dependencies

### RUSTSEC-2026-0544 — `libz-sys`

- Advisory published: 2026-11-11
- Advisory age when reviewed: 6 days
- Package release age reported by Cargo: 8 days
- Previous version: `1.1.31`
- Updated version: `1.1.33`
- Remediation: approved precise update with temporary publish-age allowance

Command pattern:

```sh
cargo update -p libz-sys --precise 1.1.33 \
  --config 'resolver.incompatible-publish-age="allow"'
```

Updated repositories:

- `rust-bitcoin-indexer`
- `rust-bitvmx-bitcoin`
- `rust-bitvmx-broker`
- `rust-bitvmx-client`
- `rust-bitvmx-job-dispatcher`
- `rust-bitvmx-key-manager`
- `rust-bitvmx-protocol-builder`
- `rust-bitvmx-storage-backend`
- `rust-bitvmx-transaction-monitor`
- `rust-bitvmx-wallet`

Each lockfile changed only the `libz-sys` version and checksum.

### Rust TLS stack in `rust-meta`

The advisories were older than the 14-day cooldown and were remediated directly:

- `rustls 0.23.37 -> 0.23.46`
- `rustls-webpki 0.103.10 -> 0.103.16`

This resolves:

- RUSTSEC-2026-0285
- RUSTSEC-2026-0346
- RUSTSEC-2026-0348
- RUSTSEC-2026-0349
- RUSTSEC-2026-0098
- RUSTSEC-2026-0099
- RUSTSEC-2026-0104
- RUSTSEC-2026-0382

## New scoped audit exceptions

### RUSTSEC-2026-0387 — `crypto-bigint 0.5.5`

A direct update to `crypto-bigint 0.7.10` was attempted and rejected because:

```text
age 0.12.1 -> p256 0.13.2 -> elliptic-curve 0.13.8 -> crypto-bigint ^0.5
```

`age 0.12.1` is currently the latest upstream `age` release. The fixed `crypto-bigint` 0.7 API cannot be selected by the upstream `elliptic-curve` requirement.

New exceptions were added in:

- `rust-bitcoin-indexer`
- `rust-bitvmx-bitcoin`
- `rust-bitvmx-broker`
- `rust-bitvmx-job-dispatcher`
- `rust-bitvmx-protocol-builder`
- `rust-bitvmx-settings`
- `rust-bitvmx-transaction-monitor`
- `rust-bitvmx-wallet`

Other repositories already had a scoped exception where applicable.

### RUSTSEC-2026-0382 — `rustls-webpki 0.101.7`

Repository: `rust-bitvmx-job-dispatcher`

A direct update to `0.103.16` was attempted and rejected because the AWS/Smithy legacy TLS stack requires:

```text
aws-smithy-http-client -> rustls 0.21.12 -> rustls-webpki ^0.101.7
```

A scoped exception was added pending an upstream AWS/Smithy migration away from the legacy Rustls stack.

### RUSTSEC-2026-0459 — `libsqlite3-sys 0.28.0`

Repository: `rust-bitvmx-wallet`

A direct update to `0.39.0` was attempted and rejected because:

```text
bdk_wallet 3.2.0 -> bdk_chain 0.23.4 -> rusqlite 0.31.0 -> libsqlite3-sys ^0.28
```

`bdk_wallet 3.2.0` is currently the latest upstream release. A scoped exception was added pending an upstream BDK/Rusqlite upgrade.

### `rust-bitvmx-zk-proof`

All affected packages are constrained by the current stable RISC Zero/Arkworks dependency graph. Direct precise updates were attempted and rejected by upstream semver requirements.

Scoped exceptions added:

| Advisory | Package | Upstream constraint |
|---|---|---|
| RUSTSEC-2023-0071 | `rsa 0.9.10` | `rzup 0.5.2` requires `rsa ^0.9` |
| RUSTSEC-2026-0344 | `ruzstd 0.7.3` | `object 0.36.7` requires `ruzstd ^0.7` |
| RUSTSEC-2025-0055 | `tracing-subscriber 0.2.25` | `ark-relations 0.5.1` requires `^0.2` |
| RUSTSEC-2026-0367 | `zip 2.6.1` | RISC Zero recursion crate requires `zip ^2.1` |
| RUSTSEC-2026-0368 | `zip 2.6.1` | Same constrained copy |
| RUSTSEC-2026-0369 | `zip 2.6.1` | Same constrained copy |
| RUSTSEC-2026-0429 | `zip 2.6.1` | Same constrained copy |
| RUSTSEC-2026-0545 | `zip 2.6.1` | Same constrained copy |
| RUSTSEC-2026-0548 | `zip 2.6.1` | Same constrained copy |

RISC Zero's registry latest is currently a `5.0.0-rc.1` release. A major pre-release migration was not used as an automatic security update for the current stable 3.x integration.

## Final audit status

| Repository | Non-ignored vulnerabilities | Result |
|---|---:|---|
| `BitVMX-CPU` | 0 | Pass |
| `rust-bitcoin-indexer` | 0 | Pass |
| `rust-bitcoind` | 0 | Pass |
| `rust-bitvmx-bitcoin-rpc` | 0 | Pass |
| `rust-bitvmx-bitcoin` | 0 | Pass |
| `rust-bitvmx-broker` | 0 | Pass |
| `rust-bitvmx-client` | 0 | Pass |
| `rust-bitvmx-gc` | 0 | Pass |
| `rust-bitvmx-job-dispatcher` | 0 | Pass |
| `rust-bitvmx-key-manager` | 0 | Pass |
| `rust-bitvmx-protocol-builder` | 0 | Pass |
| `rust-bitvmx-settings` | 0 | Pass |
| `rust-bitvmx-storage-backend` | 0 | Pass |
| `rust-bitvmx-transaction-monitor` | 0 | Pass |
| `rust-bitvmx-wallet` | 0 | Pass |
| `rust-bitvmx-zk-proof` | 0 | Pass |
| `rust-meta` | 0 | Pass |

## Follow-up

Re-evaluate the exceptions when any of these upstream dependency families release compatible upgrades:

- `age` / `p256` / `elliptic-curve`
- AWS SDK / Smithy legacy TLS stack
- BDK / `rusqlite`
- RISC Zero / `rzup` / recursion crates
- Arkworks

Normal CI and builds should continue using `--locked`.
