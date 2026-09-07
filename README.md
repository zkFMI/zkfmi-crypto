# zkfmi-crypto

The shared cryptographic foundation for ZKFMI's post-quantum migration. It
defines cryptographic suite identifiers, canonical signing inputs,
purpose-specific key management, and hybrid signature and KEM compositions.

The foundation is now consumed by QOMM, zkPI, DeFMI, OCLOB, DeKYX, DeCCP, and
Aethel. The synchronized source snapshot passed the scoped P1-P6 integration
and native acceptance described in
[the integration acceptance report](docs/verification/P1_INTEGRATION_REPORT.md).
This is single-host engineering evidence. Final immutable revisions and remote
HEAD reconciliation remain pending until the user-managed GitHub organization
re-push is complete.

## Verification

Do not build or run tests on the local Mac.

### Foundation crate gate

```sh
make remote-gate
```

This gate checks only the `zkfmi-crypto` crate. It synchronizes to
`~/work/zkfmi-crypto/` on softbank-l40s, then runs fmt, Clippy (`-D warnings`),
and release tests in the official
`rust:1.97-bookworm` Docker image. Logs are retrieved into `.artifacts/`.
Other projects and existing Docker images are not modified. Build parallelism
defaults to 4.

### Eight-workspace portable compile gate

From this isolated `zkfmi-crypto` checkout, run:

```sh
PQC_NATIVE_ENGINE=0 scripts/pqc-remote.sh zkfmi-crypto \
  'bash scripts/pqc-workspace-check.sh'
```

`pqc-remote.sh` verifies the fixed integration checkout, synchronizes all eight
sibling repositories to the task-owned directory on softbank-l40s, and invokes
`pqc-workspace-check.sh` with `/integration/src/zkfmi-crypto` as its container
working directory. The inner script runs locked Cargo metadata, fmt, and locked
all-feature, all-target checks for the eight workspaces, the actual QOMM demo
binaries, and the nested `qomm-batch-audit` workspace. It also fails if any non-Aethel
workspace resolves an Aethel package.

This is a cross-workspace compile and dependency-boundary gate. It does not run
the workspace test suites, strict Clippy, MP-SPDZ execution, validator restart,
or lifecycle scenarios. It can compile every workspace and may therefore take
substantial remote CPU time even though it uses the non-native Rust image.

### Full native acceptance

The complete market, finality, lifecycle, and P6 run is:

```sh
scripts/pqc-native-acceptance.sh
```

The script takes no arguments and is intentionally pinned to this isolated
checkout, the authorized softbank-l40s task root, the verified native base image,
and the predeclared v2 contracts/manifests. It acquires the shared
`remote-run.lock`, synchronizes all eight repositories, builds two unique images
from the synchronized sources, executes the three seven-node native scenarios
and the P6 recovery tests, and verifies source, protocol, compiled MP-SPDZ, image,
and cleanup receipts. It writes run artifacts below `.artifacts/` and prepares
three postflight ledger records for guarded append after a successful run.

This is the high-cost acceptance path: it builds Rust release binaries and the
MP-SPDZ C++ artifacts, starts the isolated service/validator topologies, and
exercises crash/restart paths. The accepted warm-cache run took 15 minutes and
7 seconds; a clean build can take longer. Do not run it concurrently with any
other command using `remote-run.lock`. Its hard-coded checkout, base image and
dependency pin must be updated and revalidated after final release pins are set.

The complete work order is in
[docs/orders/2026-09-05-p0-handoff.md](docs/orders/2026-09-05-p0-handoff.md).
That document and the P0 report are historical records of the independent P0
stage. No claims are made about unmeasured performance, production adoption,
FIPS validation, independent operators, WAN operation, physical HSM custody, or
completion of an external audit.

## Deliverables

- [Cryptographic contract and responsibility boundaries](docs/CRYPTO_CONTRACT.md): registered suites, canonicalization, key rotation, and zeroization.
- [NIST / RFC known-answer vector sources](tests/vectors/SOURCES.md): pinned commits and hashes of originals and extracts.
- [Cryptographic inventory](inventory/CRYPTO_INVENTORY.md): generated from fixed snapshots of seven repositories at P0 acceptance. It does not include the subsequent Aethel separation or OCLOB updates.
- [Independent P0 acceptance report](docs/verification/P0_REPORT.md): S1–S5 commits, execution results, scope, and outstanding items.
- [P1-P6 integration acceptance report](docs/verification/P1_INTEGRATION_REPORT.md): current implemented boundaries, deterministic gates, native scenarios, and explicit evidence limits.
- [Consolidated integration receipt](docs/verification/PQC_INTEGRATION_ACCEPTANCE_2026-09-06.json): source, protocol, image, scenario, and P6 evidence for the accepted synchronized snapshot; final release pins remain pending.
- [Initial competitor survey excluding Canton](docs/research/COMPETITORS_EX_CANTON_2026-09-05.md): the historical 17-target survey of confidentiality boundaries, financial workflows, maturity, and comparison questions for ZKFMI.
- [Canton Network research](docs/research/CANTON_NETWORK_2026-09-05.md): Daml, validators and synchronizers, confidentiality, verification, DvP boundaries, commercial cases, and differences from ZKFMI.
- [Claude Fable 5.1 Max review](docs/research/CANTON_FABLE_5_1_MAX_REVIEW_2026-09-05.md): independent cross-checks of Canton primary sources, reservation and DvP behavior, confidentiality boundaries, and strategy.
- [Comparison decisions v1.2](docs/research/COMPETITIVE_DECISIONS_2026-09-05.md): treatment of 18 targets including Canton, established differences, limits on claims, and implementation evidence.
- [ZKFMI strategy v1.2](docs/strategy/ZKFMI_STRATEGY_2026-09-05.md): initial customers, offering scope, adoption and development sequence, metrics, and review conditions, including Canton as a competitor and possible implementation or integration platform.

The user selected public visibility on 2026-09-05. The repository is published
on `main` at [zkFMI/zkfmi-crypto](https://github.com/zkFMI/zkfmi-crypto) in the
[`zkFMI`](https://github.com/zkFMI) organisation; the integration lane is the
`codex/pqc-astra-high` branch of each repository. This decision supersedes the
private visibility specified in the original work order.

## Reproducing the inventory

Source repositories are read-only inputs. The Mac is used only to copy them.

```sh
scripts/capture-inventory.sh /Users/shukob/Research/DeFMI "$PWD/.cache/inventory-source"
```

Synchronize the captured snapshots to `.cache/inventory-source/` inside the
dedicated directory on softbank-l40s, then run the following in the same Rust
Docker environment.

```sh
cargo run --release --bin crypto-inventory -- generate .cache/inventory-source inventory
scripts/inventory_check.sh .cache/inventory-source
```

Existing snapshots cannot be overwritten. Use a new name when capturing again.
The detected set is independently compared using GNU grep. Missing or extra
files, missing primitives, source-hash mismatches, and differences between the
JSON and Markdown inventories cause a nonzero exit.
With the fixed snapshots already available in the current environment, rerun
all gates with:

```sh
RUN_INVENTORY_CHECK=1 make remote-gate
```
