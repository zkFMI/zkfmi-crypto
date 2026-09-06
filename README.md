# zkfmi-crypto

An independent Rust crate for P0 of ZKFMI's post-quantum migration. It provides
shared boundaries for cryptographic suite identifiers, canonical signing inputs,
purpose-specific key management, and hybrid cryptography.

This P0 implementation is not yet wired into QOMM, zkPI, DeFMI, OCLOB, DeKYX,
DeCCP, or Aethel. It defines interfaces for future service integration without
replacing existing key management. Shared dependencies will use Git URLs pinned
to a `rev`; consolidating duplicated common crates is a prerequisite for P1
integration. P1 TLS configuration and P2 zkPI storage changes are outside this
P0 scope.

## Verification

Do not build or run tests on the local Mac.

```sh
make remote-gate
```

The gate synchronizes only to `~/work/zkfmi-crypto/` on softbank-l40s, then runs
fmt, Clippy (`-D warnings`), and release tests in the official
`rust:1.97-bookworm` Docker image. Logs are retrieved into `.artifacts/`.
Other projects and existing Docker images are not modified. Build parallelism
defaults to 4.

The complete work order is in
[docs/orders/2026-09-05-p0-handoff.md](docs/orders/2026-09-05-p0-handoff.md).
No claims are made about unmeasured performance, production adoption, FIPS
validation, or completion of an external audit.

## Deliverables

- [Cryptographic contract and responsibility boundaries](docs/CRYPTO_CONTRACT.md): registered suites, canonicalization, key rotation, and zeroization.
- [NIST / RFC known-answer vector sources](tests/vectors/SOURCES.md): pinned commits and hashes of originals and extracts.
- [Cryptographic inventory](inventory/CRYPTO_INVENTORY.md): generated from fixed snapshots of seven repositories at P0 acceptance. It does not include the subsequent Aethel separation or OCLOB updates.
- [Independent P0 acceptance report](docs/verification/P0_REPORT.md): S1–S5 commits, execution results, scope, and outstanding items.
- [Initial competitor survey excluding Canton](docs/research/COMPETITORS_EX_CANTON_2026-09-05.md): the historical 17-target survey of confidentiality boundaries, financial workflows, maturity, and comparison questions for ZKFMI.
- [Canton Network research](docs/research/CANTON_NETWORK_2026-09-05.md): Daml, validators and synchronizers, confidentiality, verification, DvP boundaries, commercial cases, and differences from ZKFMI.
- [Claude Fable 5.1 Max review](docs/research/CANTON_FABLE_5_1_MAX_REVIEW_2026-09-05.md): independent cross-checks of Canton primary sources, reservation and DvP behavior, confidentiality boundaries, and strategy.
- [Comparison decisions v1.2](docs/research/COMPETITIVE_DECISIONS_2026-09-05.md): treatment of 18 targets including Canton, established differences, limits on claims, and implementation evidence.
- [ZKFMI strategy v1.2](docs/strategy/ZKFMI_STRATEGY_2026-09-05.md): initial customers, offering scope, adoption and development sequence, metrics, and review conditions, including Canton as a competitor and possible implementation or integration platform.

The user selected public visibility on 2026-09-05. The repository is published
on `main` at [shukob/zkfmi-crypto](https://github.com/zkFMI/zkfmi-crypto).
This decision supersedes the private visibility specified in the original work order.

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
