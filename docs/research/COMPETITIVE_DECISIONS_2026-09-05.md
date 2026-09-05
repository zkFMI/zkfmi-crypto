# Comparison Decisions for 18 Targets Including Canton v1.2

Finalized: **2026-09-05**. Scope: ZKFMI as a whole and 18 targets, comprising the 17 in the [initial survey][survey] plus the [supplementary Canton research][canton]. The local implementation baseline time is **2026-09-05 10:55:32 UTC**. The [baseline file][baseline] records HEADs for 6 repositories, SHA-256 hashes for 15 materials, and the presence of uncommitted changes. v1.2 integrates the [Claude Fable 5.1 Max review][fable]. Subsequent local evidence is kept in the [additional snapshot at 13:14:06 UTC][update]; the original baseline file has not been altered.

**The conclusions finalized here concern ZKFMI's comparative position and adoption strategy. They do not establish exclusive functionality, performance superiority, customer demand, or production safety.** Features not found in competitors' materials are not labeled “unsupported,” and commercial announcements, specifications, and our own execution records are distinguished. Future choices are connected to the [strategy document][strategy].

## 1. Finalized Decisions

| ID | Comparison decision | Evidence and scope | Implication for strategy |
| --- | --- | --- | --- |
| C01 | MPC, ZK, and proofs of fill-computation correctness do not establish uniqueness | Renegade's VALID MATCH MPC addresses validity of input orders and balances, matching correctness, and output encryption. Canton's general architecture also represents business rules, authorization, and state in Daml, with relevant participants validating Daml execution, authorization, and state for submitted transactions. The same market app as ZKFMI is unverified. Prime Match also provides a prior report of financial MPC [R4][r4] [P1][p1] [Canton Section 3][canton] | Do not market “only we can prove market rules” |
| C02 | Promising comparison axes are which secrets are hidden from whom, and which market rules and states are bound to the same transaction | Renegade's delegated relayer reads the wallets it serves. Canton hides views from unrelated parties and synchronizers, but host validators and entitled parties read the relevant data. OCLOB's new path creates shares on the corporate side, but collusion by 3 or more nodes and communication observation remain separate limitations [R1][r1] [L4][l4] [Canton Section 2][canton] | Establish the disclosure scope customers permit first, and focus on projects requiring secrecy even from computing entities |
| C03 | Differences in price formation are confirmed, but do not imply universal superiority | Renegade's public description is midpoint crossing using external prices. QOMM is an RFQ that evaluates confidential pricing policies; OCLOB uses price-time-priority matching according to finalized admission order [R5][r5] [L10][l10] [L4][l4] | Do not mix RFQ, continuous order books, and midpoint crossing into a purportedly like-for-like “speed ranking” |
| C04 | ZKFMI cannot be described as uniformly stronger in secret-sharing security | Arcium Cerberus preserves confidentiality if at least one party is honest and aborts on abnormalities. OCLOB assumes at most 2 malicious nodes out of 7. Zama KMS addresses completion of key generation and decryption with t < n/3 [A2][a2] [Z3][z3] [L4][l4] | Present confidentiality, correctness, completion, and operator independence separately |
| C05 | Existing platforms also precede ZKFMI in instruction standardization, DvP, and coordination across ledgers | Ownera describes intents, asset holds, and orchestration according to ledger capabilities. Corda verifies contract state and uniqueness. Canton standardizes reservations through Token Standard allocations and cross-app DvP in one Daml transaction on a common synchronizer [O1][o1] [C2][c2] [Canton Section 4][canton] | Propose zkPI in terms of verification that can be added to existing authorization and ledger contracts. Do not claim atomicity itself as unique |
| C06 | ZKFMI is a research MVP demonstrating some real functionality, with a maturity gap relative to finished institutional products | In addition to the initial two-fill smoke, the later snapshot confirms smoke_only evidence for two rounds of real fills, cancellation, expiry, and restart of 7 MPC nodes. Independent operation and WAN are false [Updated snapshot][update]. Even when distinguishing Mainnet, commercial DLR, TestNet pilot, a single real transaction, and launched PoCs, Canton has broader operational experience than ZKFMI [L6][l6] [Canton Section 5][canton] | Next priorities are verification of continued use, consistency under failure, and independent operation. Do not understate competitors' maturity |
| C07 | In Japan, connection to existing securities workflows and the cash leg may be an adoption condition | Progmat, ibet for Fin, Kinexys, Fnality, and others publish adoption and operational records for the target workflows; MUFG / Progmat launched a JGB repo demonstration collaboration on Canton. Integration with ZKFMI, completion of the Canton demonstration, and legal delivery are unverified [J1][j1] [B2][b2] [K1][k1] [F1][f1] [Canton Section 5][canton] | Make an integration-based adoption demonstration the first candidate. Confirm contractual / API access rights and legal roles separately |
| C08 | System-wide quantum resistance cannot currently be established as a competitive advantage | The standalone zkfmi-crypto P0 and existing commitments, proofs, signatures, communications, and stored state are separate migration targets [P0][p0] | Offer PQC as a migration plan for each boundary; do not claim whole-system readiness |
| C09 | Canton is both a priority competitor and a candidate platform for additional functionality or integration destination | The official architecture places business logic, authorization, and privacy rules in Daml and permits a choice of Global / private synchronizers. Connecting external MPC results or a zkPI verifier to Canton is a design inference; an equivalent implementation, identical confidentiality conditions, performance, costs, and legal delivery are unverified [Canton Sections 6–8][canton] | Preserve the objectives of a standalone DeFMI L1, QOMM, OCLOB, and DeCCP, and evaluate Canton integration and alternative configurations for the same workflow at G2 |

A “proof of market rules” ultimately proves correctness for chosen functions and inputs. General-purpose MPC/ZK platforms could implement similar functions. Place ZKFMI's comparative value in implementation and operational evidence connecting rules, input admission, eligibility, reservations, settlement, and recovery. Do not use the existing POSITION document's shorthand, “others prove circuits; we prove markets,” as proof of novelty on its own.

## 2. Finalized Treatment of the 18 Targets

“Priority comparison” means a target directly relevant to development and proposal decisions. “Integration candidate” does not mean an existing integration or partner. “Monitor” does not mean ignore; it means no individual deployment is being pursued at this stage. The priorities below are judgments of this research, not market-share rankings.

| Target | Role confirmed from public materials | Key confidentiality and verification boundaries | Treatment of maturity | Treatment within ZKFMI |
| --- | --- | --- | --- | --- |
| **Canton Network** | Daml apps, validators hosting parties, synchronizers coordinating ordering and confirmation, and cross-app atomic transactions | Unrelated parties and synchronizers do not read payloads. Host validators and relevant parties read the corresponding views. Daml validation differs from MPC's guarantee of hiding inputs even from computing entities | Distinguish Global Synchronizer Mainnet, commercial DLR, TestNet pilot, a single real transaction, and launched PoCs | **Highest-priority comparison and platform/integration candidate**: compare confidentiality boundaries, input sets, market rules, reservations, DvP, and operational responsibility within the same workflow [Supplementary Canton research][canton] |
| **Renegade** | Midpoint crossing through MPC and collaborative SNARKs | A delegated relayer can read the orders it handles. Matching-correctness proofs exist. Equivalence of market-wide admission-set and ordering guarantees is unverified | Mainnet launch announced | **Priority comparison**: differences from OCLOB's admission ordering and market mechanism, and corporate-side sharing [R1][r1] [R4][r4] [R5][r5] |
| **Arcium** | General-purpose MPC application infrastructure | Dishonest-majority / detect-and-abort. Applications define financial workflow semantics | Mainnet Alpha announced | **Priority comparison and infrastructure candidate**: costs and trust conditions for building the same workflow [A2][a2] [Survey][survey] |
| **Zama** | Confidential-state computation through FHE and distributed key management | Separate decryption authority, KMS, and computation-result verification. Input ZKPoK also differs from a proof of the entire workflow | Mainnet and confidential auction execution announced | **Infrastructure comparison**: assess the justification for an in-house implementation alongside Arcium [Z2][z2] [Z3][z3] [Z4][z4] |
| **R3 Corda** | Contract and asset-state management between institutions | Limits sharing. Disclosure to notaries differs from disclosure to transaction participants | Adoption for securities settlement announced | **Priority comparison and integration candidate**: does the added value outweigh the reasons to adopt the ledger? [C1][c1] [C2][c2] |
| **Kinexys** | Bank payments and asset tokenization | Bank operation, deposits, and customer relationships. Distinct from a guarantee of hiding inputs from the bank | Commercial workflows and transaction cases announced | **Institutional comparison and cash-leg candidate** [K1][k1] |
| **Fnality** | Institutional cash settlement | Payment arrangements, backing funds, and participant / operational conditions | Supervisory materials describe operations subject to limits | **Cash-leg candidate**: compare the adoption burden of a proprietary cash infrastructure [F1][f1] |
| **Partior** | Cross-border payments and FX PvP | Participating banks and payment paths. Specifications for a confidential order market are unverified | Distinguish live payment cases from DvP PoCs | **Cash-leg / PvP candidate** [T1][t1] |
| **Ownera / FinP2P** | Trade intents and execution coordination across multiple ledgers | Depends on signatures, receipts, agreement, and the underlying ledgers' hold / atomic capabilities | Concrete APIs and specifications verified | **Priority comparison and integration-design reference** [O1][o1] |
| **Swift shared ledger** | Coordination of interbank payments | Separates the shared layer from banks' asset and liquidity management. Uses existing settlement paths | The 2026-07 announcement concerns preparation for a live pilot | **Monitor for institutional connectivity**. Do not treat all functions as live [S1][s1] |
| **Chainlink** | External information, policy, and cross-chain integration | Distinguish guarantees from messages, TEEs, signatures, and similar mechanisms from business-workflow proofs | Specifications and cases by capability | **Integration candidate and comparison for zkPI** [LNK1][lnk1] |
| **Progmat** | Japanese ST issuance and administration | Separate issuance, trust, and distribution roles from the ledger | Distinguish ST cases and the provider's announcement of completed Avalanche migration from the launch of the Canton JGB repo demonstration | **Priority Japanese comparison and integration candidate**. Do not generalize the Canton demonstration to production features of existing STs [J1][j1] [J5][j5] [Canton Section 5][canton] |
| **BOOSTRY / ibet for Fin** | Japanese STs and consortium operation | Standard contracts, participating organizations, and issuance / circulation practice | Participants announced the start of operation | **Priority Japanese comparison and integration candidate** [B2][b2] |
| **Prime Match** | Confidential inventory matching between a financial institution and its clients | Bank / client MPC model defined by the authors | Live operation reported in 2023. Current operation is unverified | **Prior-research comparison**. Reject claims of being the first financial MPC system [P1][p1] |
| **Penumbra** | Shielded pool and batch DEX | Current swap input assets / amounts are public; distinguish this from confidentiality of claims and other elements | Specifications verified. Sealed-bid version is a future extension | **Monitor market design** [N1][n1] |
| **Dusk** | Ledger and selective disclosure for regulated assets | Also has a public account model; application-specific checks are needed | Mainnet connection specifications and collaborations announced | **Monitor securities workflows**. Do not generalize a partner's license to all applications [D1][d1] [D2][d2] |
| **Aztec** | Private/public application infrastructure | Client-side proving differs from computation over multiple parties' secret inputs. Watch version status | Alpha V5 vulnerability notice; completion of the V6 fix was not verified in this research | **Monitor infrastructure** [X2][x2] |
| **Hyperledger Fabric** | Ledger and private data among authorized organizations | Authorized peers read actual data. Hashes are shared with others | Specifications verified | **In-house alternative**: a comparison target when limiting which organizations can read is sufficient [H1][h1] |

## 3. Decisive Points in the Closest Comparisons

### 3.1 Differences from Canton

Reject comparisons that characterize Canton as “an institutional ledger without confidential computation or atomic settlement.” Official docs describe Daml transactions split into views, only relevant participants decrypting them and checking re-execution, authorization, and the Active Contract Set, and synchronizers coordinating encrypted-message ordering and commit / abort. On a common synchronizer, DvP across multiple applications and participants can execute atomically in one transaction. [Canton Sections 1–4][canton]

The confirmed difference is from whom secrets are hidden. In Canton's standard path, a host validator holds its parties' data, and validators involved in a transaction validate their own views in plaintext. OCLOB's corporate-side share-generation path does not provide complete inputs to individual MPC nodes within the tolerated collusion threshold. Designs that pass external MPC results into Daml on Canton, or express eligibility, credit, reservations, and matching rules in Daml, can be inferred as candidates, but an equivalent app implementation has not been verified. This possibility prevents treating the difference as exclusive ZKFMI novelty.

Canton's Proof of Stakeholder has parties validate submitted Daml transactions and the relevant contracts. Whether orders were omitted from the whole market, admission ordering or censorship before submission, and consistency with external credit and custody master records lie at individual application boundaries. ZKFMI does not automatically eliminate pre-admission censorship either. **Without a comparison that defines the same admission set and external legs, do not claim that only one system proves the market.**

Canton can be both a competitor to ZKFMI and an implementation destination for adding MPC order processing, a zkPI verifier, and eligibility / reservation adapters to Daml asset / cash contracts, or a settlement integration destination. Customer demand, performance and costs under the same conditions, API access rights, and legal authorization are unverified, so do not discontinue or replace DeFMI at this stage.

The concrete comparison unit is **Daml app + Token Standard + synchronizer + validator operation**. Do not claim uniqueness from the existence of reservations or DvP. At G2, compare withdrawal and jointly authorized cancellation conditions; the guarantee difference between verifying zkPI inside Daml and accepting an off-ledger verifier's signature; participation sponsors; traffic costs; and finalized-state readback through the Ledger API. Distinguish the CIP body from the Splice interface, and check the target registry implementation through a real API. [Canton Section 4][canton] [Fable Section 5][fable]

### 3.2 Differences from Renegade

Reject comparisons describing Renegade as “trusting matching results based only on signatures.” Its official repository explicitly specifies collaborative proofs covering correct matching and valid inputs. [R4][r4]

Confirmed differences concern the publicly described midpoint price, the assigned relayer's visibility, and ZKFMI's workflow choice of RFQ / continuous order book. [R5][r5] However, customers operating their own relayers can avoid disclosing plaintext to external providers. ZKFMI's “hidden from the operator” proposition therefore requires a comparison of operating burdens and collusion conditions that also includes self-operated relayers.

OCLOB's admission-order proof does not guarantee the time an order was first sent across the entire network or the absence of pre-admission censorship. QOMM's minimum price is likewise a proposition within a specified participant and input set, distinct from the whole market or legally defined best execution. **Claim a difference only after specifying the comparison set and admission boundary.**

### 3.3 Differences from Arcium / Zama

Do not assign scores to MPC method names, node counts, or the FHE label. Compare trust conditions and operating costs when implementing the same inputs, rules, outputs, and decryption authority. Arcium's abort behavior and Zama KMS's key-processing completion conditions guarantee different things. [A2][a2] [Z3][z3]

Zama's coprocessor description distinguishes input ZKPoK, FHE computation, commitments, and signatures. Do not reinterpret this as “the entire business result can be verified with a single ZK proof.” Equally, do not assert that only ZKFMI has external verification. [Z4][z4]

Retain in-house MPC as the currently executable baseline. A decision to change infrastructure requires results from the same end-to-end path; no infrastructure was introduced or replaced in this research.

### 3.4 Differences from Ownera / Corda

Ownera already describes signing intents, agreement among multiple institutions, checking receipts, and DvP according to ledger capabilities. [O1][o1] Corda also handles contract conditions, state transitions, and prevention of double spending. [C2][c2]

The potential added value of zkPI is binding specific rules evaluated over confidential inputs and asset reservations to an instruction, so that the receiver can verify defined propositions. However, if an adapter's receiver accepts only signatures or digests without verifying proofs, the guarantee falls back to trust in that adapter / signer. Do not describe atomicity within DeFMI as extending to atomicity across arbitrary banking systems.

## 4. Scope Confirmed for Our Own Implementation

| Item | Judgment in this review | Evidence and limitations |
| --- | --- | --- |
| Information separation from orders to MPC | Design and implementation descriptions of the new CLI/Docker path confirmed | Corporate-side sharing, 7 nodes, and original orders withheld from the coordinator. A path holding plaintext remains in the browser-compatible demo [L4][l4] |
| Settlement of multiple fills | **Confirmed by reading existing execution artifacts** | 2 fills, 1 transaction, 14 node/fill confirmations, matching roots and restart across all 5 validators. Artifact SHA-256 is fixed in the baseline file. Not rerun in this task [L5][l5] [L6][l6] |
| Continuous trading | Additional post-baseline evidence is **smoke_only** | cycle-final-006 records 2 rounds and 3 fills. lifecycle-final-004 records cancellation, actual expiry, reuse of returned assets, restart of 7 MPC nodes, and agreement across 5 ledgers. rough-001's rejected record is retained. This task did not rerun these paths or independently inspect remote logs [Updated snapshot][update] |
| Trust in ledger finality | Trust in the read service remains | Each node performs its own readback, but does not directly verify an independent consensus proof [L4][l4] |
| Settlement confidentiality scope | Not all fields are confidential | Asset IDs and settlement metadata on the native rail are public [L1][l1] |
| Identity and eligibility | Pseudonymity within a scope | DeKYX selectively discloses eligibility, but is not fully unlinkable from issuer records. It is not a KYC/KYB-certified product [L14][l14] |
| Clearing | Research clearing and risk state machine | DeCCP alone is neither an asset custodian nor an authorized clearing institution [L15][l15] |
| Cryptographic security | Distinguish fixed defects from unaccepted production guarantees | A record exists of fixing a defect in the old note proof. The current Triptych dependency is experimental; independent audit and old-state migration are separate conditions [L3][l3] |
| QOMM economic effects | Treat confirmation experiments and external validation as not yet passed | Existing contract is at the smoke stage. Do not promote synthetic-data smoke evidence into price superiority or customer benefits [L11][l11] |
| Aethel dependency | Source and dependency-graph separation implemented | Moved 4 app-specific integration crates into Aethel. All-feature metadata for the 6 foundations contains 0 Aethel references. Passed 469 foundation tests without mounting Aethel, and 57 application tests. Deployment, old-state migration, and the post-change live end-to-end path require separate acceptance [Dependency separation][independence] |
| PQC | Standalone P0 implementation | Wiring into all existing services and system-wide quantum resistance have not been demonstrated [P0][p0] |

OCLOB is being changed in a separate task. The additional snapshot incorporates only contract, manifest, artifact, and ledger hashes and recorded execution verdicts; it did not stop or modify that work. lifecycle-final-004 ran on the previously distributed DeFMI revision and must not be reused as live evidence for the binary after this separation. Keep concurrency, unattended recovery, UI, and independent operation as unaccepted conditions.

## 5. Acceptable and Rejected Wording

| Wording | Judgment | Conditions for use / what to show instead |
| --- | --- | --- |
| “A research implementation connecting confidential order admission, price-time-priority matching, advance reservation, and proof-carrying settlement” | Acceptable | State the new CLI/Docker path and the scope executed on a single host |
| “Atomically settled 2 fills in 1 transaction” | Acceptable | Attach the conditions and artifact of the relevant smoke run. Do not describe it as general processing capacity |
| “Aims to independently verify consistency between a specified order set and rules and the settlement results” | Conditional | Specify the propositions actually checked by the verifier and trust in signers / readback |
| “The world's first financial MPC” / “The only MPC + ZK settlement” | Reject | Prime Match and Renegade precede it |
| “Competitors only prove circuits; we prove market rules” | Reject as evidence of novelty | Renegade also has matching-correctness proofs; in Canton, relevant participants validate Daml rules. Show specific differences in functions, sets, authorization, and settlement |
| “Canton cannot implement orders, eligibility, or reservations” / “Canton has no atomic DvP” | Reject | Acknowledge the possibility of building the same workflow in Daml and cross-app atomic transactions, and compare only the scope of publicly documented individual applications |
| “With 7 nodes, confidentiality is stronger than Arcium's” | Reject | Assumptions about honest participants differ |
| “A confidential ledger hides everything, including asset types, communications, and real identities” | Reject | Distinguish public asset IDs, communication observation, and DeKYX's relationship with issuers |
| “Production FMI/CCP” / “The entire system is quantum-resistant” / “Faster than competitors” | Not currently acceptable | Requires, respectively, institutional and operational evidence, evidence covering all cryptographic paths, and performance evidence under identical conditions |

## 6. Conditions for Updating the Comparison

1. **C02/C06:** New acceptance receipts for continuous trading, cancellation, expiry, contention, recovery, and independent operation.
2. **C01/C03/C09:** Explicit mappings to the same admission set, ordering, eligibility, reservations, and settlement conditions in Canton or Renegade. Do not unilaterally turn unverified items into weaknesses.
3. **C04:** Execution of the same end-to-end path on Arcium/Zama, with measured differences in outputs, confidentiality scope, costs, and failure behavior.
4. **C05/C07:** The ledger integration destination confirms APIs, hold/commit/abort, readback, and allocation of legal workflow responsibilities.
5. **C08:** Migration receipts covering components beyond signatures/KEM.
6. Changes in vendors' official versions, operating stages, or fix notices. Recheck the relevant sources before reuse in external proposals.

## 7. Sources and Retrieval Limitations

v1.2 addition: Fable retrieved 14 of the 15 existing Canton-related URLs again, replaced the original Tradeweb URL with its official republication, and retrieved 23 new URLs. Of 38 referenced URLs with IDs, 37 could be checked. Do not double-count an original URL and its republication as independent adoption cases. See [Fable Sections 1 and 9][fable] for retrieval times and the breakdown, and the [source index][source_index] for the reference URL set in the integrated documents. The 54-source breakdown below is as of v1.1.

The external foundational materials retain the 37 sources from the [initial survey][survey] plus three later additions: Renegade's official repository and P2P explanation, and Zama's coprocessor documentation. The [supplementary Canton research][canton] checked 14 new sources, including specification and operational materials from Canton / Digital Asset / Global Synchronizer Foundation and adopter announcements from Broadridge / Tradeweb, for 54 distinct sources in total. MUFG's JGB repo material reuses J3 from the initial survey.

The Canton pilot PDF could not be opened directly through Web retrieval because of a size limit, so the successfully retrieved official pilot-completion announcement was used as evidence; the PDF is excluded from materials read in full. The Renegade whitepaper's 502, Zama body content-type error, and Kinexys 403 likewise remain excluded from the fully read materials.

L1–L15 correspond to the originals in the [baseline file][baseline]. Because linked working trees may change in the future, use the SHA-256 hashes in that file as the reference for the bytes at comparison time. This document is not a record of a source audit, benchmark, or customer interview.

[survey]: COMPETITORS_EX_CANTON_2026-09-05.md
[canton]: CANTON_NETWORK_2026-09-05.md
[baseline]: STRATEGY_BASELINE_2026-09-05.json
[strategy]: ../strategy/ZKFMI_STRATEGY_2026-09-05.md
[p0]: ../verification/P0_REPORT.md
[r1]: https://help.renegade.fi/hc/en-us/articles/32530262853395-What-are-relayers-on-Renegade
[r4]: https://github.com/renegade-fi/renegade
[r5]: https://help.renegade.fi/hc/en-us/articles/32530439391763-What-is-the-Renegade-peer-to-peer-network
[p1]: https://arxiv.org/abs/2310.09621
[a2]: https://docs.arcium.com/multi-party-execution-environments-mxes/mpc-protocols
[z2]: https://www.zama.org/post/zama-developer-program-mainnet-season1-building-for-the-long-game
[z3]: https://github.com/zama-ai/kms/blob/main/docs/getting-started/concepts.md
[z4]: https://docs.zama.org/protocol/protocol/overview/coprocessor
[c1]: https://r3.com/r3s-corda-selected-as-first-authorized-dlt-platform-for-european-dlt-pilot-regime/
[c2]: https://docs.r3.com/en/platform/corda/5.2/developing-applications/ledger/notaries/non-validating-notary.html
[o1]: https://finp2p-docs.ownera.io/docs/orchestration-plan-capabilities
[j1]: https://progmat.co.jp/concept/st_list/
[j5]: https://www.linkedin.com/posts/avacloud_progmat-inc-japans-largest-security-token-activity-7482536047919136768-zSy5
[b2]: https://www.sbigroup.co.jp/news/pdf/group/2021/0615_a.pdf
[k1]: https://www.jpmorgan.com/payments/newsroom/kinexys-milestones-2026
[f1]: https://www.bankofengland.co.uk/financial-stability/financial-market-infrastructure-supervision/report/fmi-annual-report-2025-26
[t1]: https://partior.com/
[s1]: https://www.swift.com/news-events/press-releases/swifts-blockchain-ledger-ready-use-17-banks-set-pioneer-tokenised-cross-border-payments-trusted-global-infrastructure
[lnk1]: https://chain.link/blog/how-chainlink-is-bringing-privacy-to-blockchains
[n1]: https://protocol.penumbra.zone/main/dex/swap.html
[d1]: https://dusk.network/
[d2]: https://docs.dusk.network/developer/integrations/exchanges/
[x2]: https://aztec.network/blog/alpha-v5-proving-system-vulnerability
[h1]: https://hyperledger-fabric.readthedocs.io/en/latest/private-data/private-data.html
[l1]: /Users/shukob/Research/DeFMI/defmi/README.md
[l3]: /Users/shukob/Research/DeFMI/defmi/docs/NOTE_PROOF_SECURITY_REVIEW_20260905.md
[l4]: /Users/shukob/Research/DeFMI/oclob/README.md
[l5]: /Users/shukob/Research/DeFMI/oclob/docs/NATIVE_MULTIFILL_JA.md
[l6]: /Users/shukob/Research/DeFMI/oclob/artifacts/oclob_native_multifill.json
[l7]: /Users/shukob/Research/DeFMI/oclob/research/experiment-ledger.jsonl
[l8]: /Users/shukob/Research/DeFMI/oclob/research/oclob_native_cycle_contract.json
[l10]: /Users/shukob/Research/DeFMI/qomm/README.md
[l11]: /Users/shukob/Research/DeFMI/qomm/research/contract.json
[l14]: /Users/shukob/Research/DeFMI/dekyx/README.md
[l15]: /Users/shukob/Research/DeFMI/deccp/README.md

[fable]: CANTON_FABLE_5_1_MAX_REVIEW_2026-09-05.md
[update]: STRATEGY_UPDATE_2026-09-05.json
[independence]: ../../../aethel/docs/FOUNDATION_INDEPENDENCE_JA.md
[source_index]: COMPARISON_SOURCE_INDEX_2026-09-05.json
