# PQ collaborative proof references: bounded supplementary search

Checked: 2026-09-08. This is source review, not a benchmark, security audit,
implementation acceptance, or a replacement of the user's full PQC-switch target.
The approved independent research implementation remains in progress.

## Conclusion

The additional references below provide useful FRI/PCS implementation material
and malicious-security requirements. None of the newly inspected artifacts has
yet been established as a drop-in implementation of both post-quantum proof
soundness and private, secret-shared collaborative proving. In particular,
distributing a cleartext witness among trusted workers does not meet our target.
This is a conclusion about the checked sources, not a claim that no other
compatible implementation exists.

## Candidate comparison

| Reference | What was checked | Relevance and remaining boundary |
| --- | --- | --- |
| [Code-based Scalable Collaborative SNARKs, 2026/729](https://eprint.iacr.org/2026/729) | Previously selected paper and author repository | Closest current specification for private collaborative PQ proofs. Independent research implementation authorized; original proof-core code-use permission has not been established. Main construction's honest-but-curious prover guarantee must not be reported as malicious security. |
| [FRIttata, 2025/1285](https://eprint.iacr.org/2025/1285) | Current abstract, author artifact, FRI guide, MIT license | Rust distributed FRI reference. Current paper describes a PCS, which still needs a PIOP. Author repository warns that its Winterfell base does not provide perfect ZK and is not production-ready. Neither distributed FRI nor its license proves inter-prover witness privacy. |
| [HyperFond, 2025/1349](https://eprint.iacr.org/2025/1349) | Abstract; PDF p. 10, Collaborative SNARKs paragraph; author repository and LICENSE_MIT | PQ distributed SNARK implementation, but the paper explicitly separates its acceleration goal from collaborative witness privacy. Useful as a performance/PCS reference; not a private collaborative backend as-is. Artifact is explicitly preliminary. |
| [Code-based Distributed PCS, 2025/2327](https://eprint.iacr.org/2025/2327) | Current abstract; PDF p. 2, section 1.1, and security section | Brakedown-based PCS. Explicitly assumes trusted workers and uses distribution only for parallel acceleration. Does not satisfy our adversarial, private-worker acceptance target as-is. No author code location or software license was established in this bounded pass. |
| [Veloz, 2026/714](https://eprint.iacr.org/2026/714) | Primary abstract | Reed-Solomon/Brakedown PCS distribution and proof aggregation; authors report Rust implementations. Potential efficiency reference. Inter-prover privacy, exact code bytes, software license and runnable artifact remain unverified; not selected. |
| [LigeSIS, 2026/751](https://eprint.iacr.org/2026/751) | Primary abstract | Homomorphic subset-sum hashing can aggregate partial commitments without Merkle-root homomorphism. This is relevant to the commitment bottleneck, but it does not by itself prove private collaborative SNARK security. Assumption/parameter analysis, software license and runnable artifact remain unverified; not selected. |
| [Experimenting with Collaborative zk-SNARKs, 2021/1530](https://eprint.iacr.org/2021/1530) | USENIX 2022 paper's results and implementation section; libiop README | Describes a collaborative Fractal route, with proof/verification overhead growing with prover count. The evaluated collaborative artifacts are Groth16, Marlin and Plonk, not Fractal. MIT libiop contains single-prover Fractal, but this does not establish a ready-made collaborative Fractal artifact. |
| [Malicious Security in Collaborative zk-SNARKs, 2025/1026](https://eprint.iacr.org/2025/1026) | Abstract and PDF pp. 3-5, particularly p. 4 attack model | Security dependency, not another PQ backend. Invalid witnesses and reactive proof computation can leak honest inputs; applying a generic malicious-MPC compiler without checking its conditions is insufficient. Positive results for particular SNARKs are not automatically a theorem about our new code-based implementation. |

The latest 2025/2327 revision is dated 2026-08-20 and has a different title from
older indexes, which call it "Transparent and Post-Quantum Distributed SNARK with
Linear Prover Time". The current primary source describes a PCS. Likewise,
FRIttata's current title and abstract describe a PCS, not an already integrated
private collaborative SNARK.

## Public software evidence

- [FRIttata author artifact](https://github.com/XuHuaXH/winterfell),
  [implementation guide](https://github.com/XuHuaXH/winterfell/blob/main/FRITTATA.md),
  [MIT license](https://github.com/XuHuaXH/winterfell/blob/main/LICENSE).
  The guide places the distributed FRI implementations in the `fri` crate.
  Observed main: `2f07d5a28967bab5bc8aee1acaa9073932000d32`.
- [HyperFond author artifact](https://github.com/n96409816/HyperFond),
  [MIT license](https://github.com/n96409816/HyperFond/blob/main/LICENSE_MIT).
  README identifies the artifact and says it is not ready for productive use.
  Observed main: `f0d67acfaf1a4abf1c904673750d6ed186886fb6`.
- [libiop](https://github.com/scipr-lab/libiop) documents MIT licensing,
  Fractal/Aurora/Ligero, `make_zk`, and its unaudited prototype status. This is a
  C++ reference, not permission to replace the application's Rust core.
  Observed master: `a2ed2ec2f3e85f29b6035951553b02cb737c817a`.

Paper-publication licenses and software licenses are recorded separately.
No new candidate proof implementation was executed, copied into an application,
or deployed during this supplementary search. No external author was contacted.
Published speedups are not our measurements and do not justify selecting a
backend without an end-to-end comparison under our target conditions.

## Evidence snapshots

PDFs were fetched from their exact IACR URLs, text-extracted and relevant pages
rendered and visually checked. Snapshot SHA-256 values:

| Paper | SHA-256 |
| --- | --- |
| 2025/1349 | `d9b95c9736e51d544423481242a8f0d176fdf99be2755b939c03e78621819cf6` |
| 2025/2327 | `b1a49cb85642a3ba6ec2ccc0bb6a6b8a8750398d3d889a9536b902116991c8fa` |
| 2025/1026 | `bb001689b3d615c2eb27b4d4795753e7ce47aefa116fe3cf7888d5ce95fe6de0` |

Local source snapshots are in `.artifacts/pqc-proof-switch/`. The comparison
does not claim a full reading or independent verification of every theorem.

## Effect on the current implementation

Keep the authorized private collaborative proof target and the overall on/off
switch unchanged. Use FRIttata/libiop only as appropriately licensed component
references where compatible; do not remove witness privacy to obtain a runnable
distributed benchmark. Bind witness-validity checking to the same shares used
by proof generation, and evaluate intermediate openings and abort leakage before
claiming malicious security. A successful tamper-rejection test alone does not
establish that privacy theorem.
