# ZKFMI Strategy v1.2 — From an 18-Target Comparison to an Adoption Demonstration

Prepared: **2026-09-05**. Based on [comparison decisions C01–C09][comparison], the [initial survey excluding Canton][survey], the [supplementary Canton research][canton], and the [implementation baseline file][baseline]. This document is the result of the comparison and strategy work delegated by the user. No customer contact, contracting, new comparative experiments, or publication was performed as part of that formulation work. The separately instructed removal of Aethel dependencies is documented in the [implementation record][independence]. v1.2 incorporates the requested [Fable 5.1 Max review][fable] and the [additional snapshot at 13:14:06 UTC][update].

**The primary objective is to provide operators of institutional digital-securities trading with confidential order admission, advance reservations, and verifiable trade-execution instructions.** Prioritize an adoption demonstration with an organization that already has order flow and asset administration, using OCLOB's new native path as the starting point for functional verification. Preserve the overall vision, including QOMM, the dedicated DeFMI L1, DeKYX, and DeCCP, and the implementation objectives of other tasks.

Prioritizing an adoption demonstration at a financial institution is a **strategic assumption**. Customer demand, an integration destination, budget, and required SLOs were not obtained in this work. The numerical targets below are planning proposals, not a sales track record, measured performance, or commitments.

For Aethel, the direction is to **remove dependencies from the entire ZKFMI foundation**. Source and dependency-graph separation has been implemented and checked through all-feature metadata for the 6 foundations, 469 foundation tests, and 57 Aethel tests. Deployment of new binaries, migration of existing state, and the live end-to-end path require separate acceptance. Preserve implementation objectives for the standalone DeFMI L1, QOMM, OCLOB, and DeCCP.

## 1. Chosen Policies

| ID | Policy | Comparison basis | Conditions for reconsideration |
| --- | --- | --- | --- |
| S01 | Focus the initial use case on institutional secondary trading of digital securities where “orders must not be disclosed to the operator until their order is fixed” | C02/C03. A closely related OCLOB execution path exists, allowing concrete comparison of its market mechanism with midpoint crossing | No demand for this confidentiality boundary; disclosure to authorized operators is sufficient |
| S02 | Make the first sales unit an integration module for confidential trading, advance reservations, and zkPI verification | C05/C07. Seek adoption through added value to existing ledgers, instruction infrastructure, and customer relationships | Receivers cannot verify proofs or link instructions to delivery |
| S03 | Use the already-running in-house MPC / DeFMI path as the baseline, and first complete continuous trading and consistency under failure | C06. Smoke evidence exists for continuous trading, cancellation, expiry, and restart. Concurrency, unattended recovery, and independent operation have not been accepted | If the next end-to-end result under the current research contract fails, resolve its cause within that contract |
| S04 | Prioritize Canton and Renegade for market and settlement comparisons, Arcium/Zama for confidential-computation infrastructure, and Ownera for integration design. Also treat Canton as a candidate destination for implementing additional functionality or integration | C01–C05/C09. Competitors, procurement candidates, and integration destinations require different evaluation targets | An alternative meets the necessary conditions with the same inputs, outputs, and confidentiality conditions, and reduces implementation and operating burdens |
| S05 | Combine a public core and verification specifications with integration and operational support | Public-release policy, current MIT license, and C07. Combine independent verification with adopters' operational responsibility | Unable to provide the maintenance responsibilities and audit scope customers require |
| S06 | Develop PQC as migration capability at each boundary | C08. Standalone P0 differs from migration of the whole stack | Concrete customer requirements, threat models, or compatibility conditions change |

S02 is not a decision to discontinue the dedicated DeFMI L1 or relax existing production acceptance conditions. It is a sequencing choice: **retain the current DeFMI as the reference implementation path and add an integration-based form of customer delivery**.

## 2. Initial Target Customers and Workflows

### Target-Customer Hypothesis

Define “operator” concretely to include the organization's own validator, application provider, and matching engine. Treat the JSCC / Mizuho / Nomura JGB collateral PoC and Progmat/DCC WG as evidence of launches and investigation. At G0, look for portions of order submission, rate quoting, and collateral allocation that require secrecy even from computing entities, without assuming direct replacement of the entire workflow. [Fable Sections 2.6 and 5][fable]

The first candidate is a Japanese securities firm or digital-securities trading-platform operator that already has institutional clients, order flow, and asset-administration providers. Candidate users are dealers and institutional investors. Entities responsible for trusts, asset administration, and settlement are important parties to integration and acceptance.

Progmat / ibet for Fin are platforms to compare, not current customers or collaborators. Canton is likewise a highest-priority comparison and integration candidate; neither adoption nor access rights have been established. The MUFG / Progmat JGB repo project using Canton is at the start of a demonstration collaboration, not evidence of completion or commercialization. [Canton Section 5][canton] Kinexys / Fnality / Partior and others have not been established as available integration destinations either.

| Role | Problem to investigate | Basis for an adoption decision |
| --- | --- | --- |
| Budget owner: head of digital-securities / trading business | Is disclosing orders to the operator a barrier to participation? | Target transactions, participants, implementation owner, and evaluation budget can be defined |
| Users: dealers and institutional investors | From whom do they want to hide limit prices, quantities, and pricing policies? | Can trade with confidentiality maintained within their organization and prior authorization, and explain the outcome afterward |
| Market operations / risk department | Consistency of admission order, eligibility, credit, remaining reservation capacity, and cancellations | Can reject invalid instructions and excessive fills, and account for all items |
| IT / security department | Key management, node operation, shutdown/recovery, and audit | Fits the organization's trust conditions and connection method, with measurable operating burden |
| Settlement / asset-administration entity | Authoritative reservations, acceptance of instructions, and delivery finality | Can actually verify asset state and proofs, with clear responsibility for failures |

### Initial Use Case

1. A corporation prepares orders and eligibility in its own environment and reserves the funds, securities, or guarantee capacity needed for trading.
2. Finalize admission order for secret-shared orders and match them under specified market rules.
3. Create zkPI binding all fills to reservations and authorization, and execute without requiring the trading parties to reauthorize after matching.
4. Each node and corporation reads back settlement results, restores remaining reservations and receipts, and uses them for the next order.
5. Auditors receive defined verification materials and check the required propositions and disclosure scope.

The initial evaluation does not require real-asset transfers; execute the full path with approved data and evaluation assets. Conditions for moving to production asset and cash legs are separated into later acceptance gates.

### Why Start with This Use Case

Differences from Renegade can be explained within the same transaction through price-time priority, confidential admission, and continuity of eligibility, reservations, and settlement. With Canton, compare sub-transaction privacy that discloses to host validators against MPC that withholds complete inputs even from each computing node; verification of submitted Daml transactions against zkPI including the admission set; and Canton-native DvP against external legs. [Comparison C02/C03/C05/C09][comparison] Current OCLOB has a two-fill end-to-end smoke, allowing existing results to inform adoption decisions before building a different market from scratch. [Comparison C06][comparison]

At the same time, there may be too few participants, order disclosure may not be a problem, acceptable latency may be too short, or advance reservations may worsen capital efficiency. This is an unverified demand hypothesis. The feasibility of confidential computation alone does not establish market demand.

### Priorities for Adjacent Use Cases

| Use case | Policy | Rationale |
| --- | --- | --- |
| OCLOB confidential orders, reservations, and settlement | Initial demonstration path | Shortest path from current implementation and execution evidence to verifying continued use |
| QOMM multi-dealer RFQ | Next workflow candidate. Existing research continues | Adopt if it matches demand for handling confidential pricing policies. Economic effects need confirmation under the existing contract |
| Confidential computation and instructions for collateral / guarantee capacity | Candidate expansion within the same customer | Design hypothesis that eligibility, reservations, and ledger readback can readily be reused |
| A new general-purpose public DEX | Not an initial sales target | This research has not confirmed liquidity acquisition or general-user demand |
| A fully new FMI/CCP service | Long-term vision | Requires separate acceptance covering the business entity, institutional framework, capital, and operational responsibility |

## 3. Offerings and Integration Conditions

### Three Offering Units

| Offering unit | Contents | Initial position |
| --- | --- | --- |
| Reference implementation | Reproducible path through corporate input, OCLOB, DeKYX, advance reservations, zkPI, and DeFMI | Baseline for technical evaluation. Retain the current dedicated non-EVM Avalanche L1 architecture |
| Integration modules | Input / eligibility / reservation adapters, zkPI verifier, reconciliation with ledger receipts, retries, and recovery | Product unit to evaluate in the first adoption demonstration |
| Operational support | Integration, configuration, key / node operation, audit materials, failure recovery, and version upgrades | Candidate for a contract after confirming demand and delivery responsibilities |

The public core consists of verification specifications, wire / use-case contracts, verifiers, sources, and reproducible tests. Do not unilaterally change the current MIT license. Exclude customer secrets, orders, credentials, keys, and operational data from publication.

### Contract Required of an Adapter

Choose **one** ledger adapter used by the evaluation partner, rather than building many from the outset. Ownera's intent/hold/receipt and Canton's Daml contract / common synchronizer / reassignment designs are comparison references, not adoption decisions. [Comparison C05/C09][comparison]

- The receiver can verify target assets, eligibility, purpose / domain / version / deadline, and the correspondence between orders and reservations.
- Check the authoritative reservation record, remaining quantity, sequence number, and revocation again when executing the instruction.
- Retries of the same instruction do not apply it twice; tampering, partial extraction, and deadline violations are rejected.
- Make explicit which proofs the receiver verifies and where trust in a signer remains.
- The meanings of hold, commit, abort/release, finalized-state readback, and recovery can be defined.
- When using multiple ledgers, define states and responsibilities if one leg fails. Do not assume DeFMI's internal atomicity has been transferred unchanged.

If choosing Canton, use **Daml app + Token Standard + synchronizer + validator operation** as the comparison and implementation unit. Map reservations to allocations and check `allocateBefore` / `settleBefore`, sender withdrawal, jointly authorized cancellation, and target-registry behavior. State whether zkPI is verified inside Daml or whether the system trusts an off-ledger verifier's signature. Include required sponsors, traffic costs, Global / private synchronizers, and Foundation governance / upgrade conditions. Cost amounts and access rights have not been obtained. [Canton Section 4][canton] [Fable Section 5][fable]

**Do not sell a destination that fails this contract as a DvP integration with the same guarantees as ZKFMI.** Reconsider the destination or retain the evaluation configuration using DeFMI. Sending signed JSON to an arbitrary API alone does not complete integration.

## 4. Priorities for Development, Procurement, and Collaboration

| Area | Next decision | Completion evidence |
| --- | --- | --- |
| Binding order admission, rules, and all fills | Implement and verify as a ZKFMI core capability | Receipt demonstrating correspondence among the set, ordering, eligibility, reservations, and all fills |
| Continued use and recovery | Highest priority under the current end-to-end gate | Receipt → next real fill → settlement; cancellation, expiry, contention, and stop/resume |
| zkPI / SDK | Clarify the delivery boundary | Concrete examples of independent verification, authoritative-state readback, retries, and partial failure |
| General-purpose MPC/FHE | Use current infrastructure as the baseline and compare Arcium/Zama when needed | Results from executing the same complete candidate. Do not replace infrastructure based solely on the cryptographic method |
| Canton comparison / adapter | Evaluate adding MPC results, zkPI verification, eligibility, and reservations to Daml asset / cash applications, or settlement integration, as G2 candidates | Instruction verification, atomic settlement, readback, and failure recovery through real APIs with the same confidentiality scope and inputs. Do not adopt based only on company names in commercial cases |
| Dedicated DeFMI L1 | Continue as part of the overall vision and as reference settlement infrastructure | Separate the current VM's end-to-end path from acceptance of independent operation |
| DeKYX | Clarify eligibility issuer, scope, revocation, and linkability | Corporate-workflow authentication entities accept responsibility for issuance, revocation, and audit |
| DeCCP | Preserve core development objectives while limiting what is mandatory for initial sales | Separate acceptance of the workflow and delivery if selling clearing / guarantees |
| Aethel dependency | Application side owns the 4 integration crates; source and Cargo dependencies have been removed from the foundation | Metadata for the 6 foundations without Aethel present, 469 relevant foundation tests, and 57 application tests. Operational acceptance separately verifies the new binary's end-to-end path and old-state migration [Dependency separation][independence] |
| PQC | Move from standalone P0 to migration by use case and boundary | Common-crate organization, version pinning, interop, revocation, migration, and reverification. Specific P1/P2 implementations are separate work |
| External cryptographic audit, HSMs, and operations | Confirm responsibilities and estimates required for production adoption | Audit of the target version, including experimental dependencies, plus evidence of key management and failure domains |

Do not add a new cryptographic core or proprietary token as a prerequisite for initial adoption. This strategy document alone does not establish that infrastructure changes, license changes, commissioning external audits, or changes to keys / operational authority have been performed or approved.

## 5. Execution Order and Acceptance Gates

The 0–90 days below are a **planning window starting when work begins and the required resources are secured**. They do not guarantee customer contracts, external audits, or independent operators within 90 days. Elapsed time without evidence does not justify promotion to the next stage.

### Advance Commercial Evaluation and Product Verification Separately

| Gate | Planning window and responsible role | Deliverables and pass conditions | If it fails |
| --- | --- | --- | --- |
| G0 Customer problem | 0–30 days, business lead. Independent of G1 | Plan approximately 5 discussions across 3 organizations. At least 1 organization documents a requirement to hide inputs even from its own validator, app, and matching entity, and defines transactions, data, owner, SLO, and evaluation budget. Also check existing Canton PoC/WG participation and the cash leg | Revisit S01 if limited visibility is sufficient. Meeting counts alone do not establish demand |
| G1 Continuous end-to-end path | First technical work, protocol lead | Additional snapshot is smoke_only through 2 trading rounds, cancellation, expiry, and restart of 7 MPC nodes. Accept the next concurrency, unattended-recovery, and UI paths under the existing contract. Check the post-separation DeFMI live path separately | Diagnose observed end-to-end failures. Do not replace experiments in other tasks |
| G2 One integration destination | After G0/G1, SDK lead and destination owner | Execute reservation → rules → instruction verification → DvP → readback through one real API / evaluation ledger. For Canton, record sponsor / costs, allocations, verification location, common synchronizer, finalized-state reads through the Ledger API, deadlines / cancellation / retries, and information visible to each entity | Change candidates if API / hold / verification authority is insufficient. Mocks and other companies' cases are not integration evidence |
| G3 Conditions for limited adoption | Approximately 30–90 days, operations / security / business leads | Accept independent administration, keys, and failure domains; audit and migration of the target version; authenticated APIs / real screens; workflow responsibilities and failure procedures | Remain at the research / evaluation stage and identify incomplete conditions |
| G4 Expansion decision | After evidence for G0–G3, business and risk owners | Evaluation results meeting agreed SLOs, costs, and operational quality, and a decision on the next contract / use case | Do not expand the adoption demonstration. Revise S01/S02 pricing, scope, or the customer hypothesis |

G1 retains accepted smoke evidence and addresses remaining concurrency, unattended recovery, and related work through **RUN_ROUGH_END_TO_END_AND_OBSERVE_FINAL_METRIC** under the existing contract. Do not place an alternative comparison platform, performance tuning, abstractions, or additional audits as independent milestones before that rough end-to-end result. Follow the existing contract and manifest for necessary fixes.

At G3, do not promote evidence of running 7 MPC processes or 5 verification processes on one machine into evidence of independent operation. Because one administrator controlling multiple nodes changes collusion conditions, check the actual keys, administrators, recovery authority, and failure domains. If adopting Canton, acceptance items also include host-validator visibility into its own parties' data and dependencies on SVs, Foundation, traffic, and upgrades. At the real-asset stage, the gate cannot pass while the current experimental Triptych dependency, old-state migration, and business delivery conditions remain unaccepted.

### Connection to Existing Research Contracts

| Target | Fixed contract and current position | Evidence needed next |
| --- | --- | --- |
| OCLOB | Cycle contract SHA-256 **b79ab992fb5fd2d5abf1c0cfe9eea6707f3188b57debdd97c555107160ea9d9a** has smoke evidence for 2 trading rounds. Subsequent lifecycle contract SHA-256 **233e378ace5575d3950a3a355d38126041374c13fc0616d4c6603e7c25ea8d0f**; final-004 is smoke_only for cancellation, expiry, and actual restart [Updated snapshot][update] | Concurrency, unattended recovery, UI, independent operation, WAN, and production conditions. Preserve original rejected records and do not count unpassed conditions as successes |
| QOMM | QOMM-PRODUCT-2026-08-24, SHA-256 **8c783133f6f737784497d700d9738a6ab9623c524fe36cae85635cb9f4153333**. current_stage = smoke | Paired comparison for the same market and requirements, prescribed statistical gates, and external-data validation. Do not claim economic superiority from smoke evidence |

This strategy has not changed research contracts, truth providers, or experiment ledgers. Before new comparative experiments, read the latest contract and hash at execution time, then follow preregistration, preflight, rough execution of the complete candidate, results, and postflight in that order. Do not interrupt ongoing product implementation or research objectives because of these sales priorities.

## 6. Success Metrics

The primary adoption-demonstration metric is **completion of one customer's entire workflow while preserving the agreed confidentiality scope, correctness, SLO, and operational responsibilities, enabling the customer to make its next adoption decision**. Do not substitute standalone proof speed or demo counts.

| Metric | Definition | Current evidence | How to set the target |
| --- | --- | --- | --- |
| Customer evaluation established | A project with agreed target workflow, owner, data, and evaluation criteria | No customer confirmation in this work | Specify conditions for 1 evaluation at G0. Do not count it as a fictional order won |
| End-to-end completion | All required states agree from admission through settlement, receipt, and the next trade | Not achieved at the original baseline time. Additional snapshot is smoke_only for 2 trading rounds, cancellation, expiry, and restart | Meet every condition of the existing contract |
| False acceptance / inconsistency | State changes caused by tampering, double use, partial extraction, deadline violations, or invalid finality | Specified smoke / regression records exist. They do not establish resistance to all attacks | 0 cases in mandatory rejection scenarios for the target version; audit conditions are independent |
| Confidentiality scope | What each entity can read of originals, shares, reservations, outputs, and metadata | Differs between the new CLI path and compatibility UI | Match the customer-agreed list of entities with visibility |
| Latency / processing capacity | Measure admission → MPC → proof → finality → receipt for the same request | Acceptance elapsed_ms is not normal latency | Fix load, geography, and confidentiality conditions; agree SLOs with the customer before measurement |
| Cost per completed transaction | Includes preprocessing, proving, communication, storage, retries, and operations | Comparable costs were not measured in this work | Build up costs under the partner's operating conditions and assess sustainable delivery |
| Funds tied up / failure rate | Reservation duration, unfilled orders, expiry, re-execution, and time to release | No commercial data | Compare with the same inputs as the existing workflow and show the cost of confidentiality |
| Operations / recovery | Recovery involving keys, node shutdown, stored state, retries, and failures at the integration destination | Limited single-host restart records | Agree required RTO/RPO and separation of roles with the partner in advance |

Comparisons claiming superior performance or economic effects must align workflow, confidentiality scope, inputs, and failure conditions. Predeclare a representative paired cohort and a power calculation or sequential stopping rule for the smallest meaningful difference. Preserve QOMM's prescribed alpha 0.05, power 0.8, and required multiple-comparison correction. Do not create numerical rankings that mix different market mechanisms, stronger guarantees, or lower loads.

## 7. Sales, Publication, and Research Approach

### Core Proposal Wording

> ZKFMI is a research implementation for verifying the correspondence between agreed trading rules and settlement results while limiting who learns what about institutional participants' orders, eligibility, and reservation information. We have currently verified a path on a single host that matches confidential orders and settles multiple fills as a batch. We evaluate integration into existing trading and asset-administration infrastructure according to the target workflow and operating conditions.

Attach the evaluated version and limitations to this wording as well. Do not sell by understating Canton's party validation, cross-app atomicity, or commercial DLR; Renegade's matching proofs; Arcium's collusion conditions; or the practice of existing financial infrastructure.

### Commercial-Process Hypothesis

1. Document the target workflow, confidentiality boundaries, integration conditions, and evaluation criteria.
2. Consider a paid PoC with clearly defined deliverables and a partner able to meet the conditions.
3. After acceptance, proceed to contracts for integration, maintenance, and operational support.
4. Extend the same verification contract to other use cases and ledgers.

Do not derive prices, market size, or expected orders from this research. Estimate after confirming required effort, external audits, operational responsibilities, and the partner's budget. Do not base current business viability on token demand or transaction-volume-linked revenue.

### Publication Policy

zkfmi-crypto was published publicly on [GitHub main](https://github.com/zkFMI/zkfmi-crypto) on 2026-09-05. Public comparison tables retain sources and versions; do not label competitors' unverified items as “absent.” Agree conditions with the partner for individual customers' confidential materials and reproducibility materials that may be published.

### Research Claims

Do not claim invention of a new cryptographic primitive or exclusive proof of market rules. Assume that the same workflow might be built on Canton with Daml and external MPC / ZK; study the architecture that consistently connects admission, eligibility, reservations, confidential computation, settlement, and recovery, together with its capabilities, costs, and failure conditions. Match claim strength to the fixed implementation and the scope of artifacts third parties can reverify.

## 8. Conditions for Changing Direction

| Observation | Strategic decision |
| --- | --- |
| Multiple target customers are unconcerned about disclosure to operators and find existing authorization controls sufficient | Reassess demand for RFQ or eligibility / collateral computation instead of fixing a confidential continuous order book as the initial use case |
| A self-operated relayer meets the required confidentiality conditions and a Renegade-style pricing mechanism suffices | Reassess the added value and operating burden of market-wide admission ordering, eligibility, and reservations |
| The same workflow on Arcium/Zama meets required guarantees and SLOs and sustainably lowers the burden relative to in-house infrastructure | Develop a concrete backend-change proposal. Do not replace without operating results |
| Canton + external MPC / zkPI, or Canton alone, meets the confidentiality scope, verification, SLOs, and costs for the same workflow | Specify additional functionality on Canton or an integration, and reassess the adoption role of a standalone DeFMI L1. Do not cancel existing objectives without execution results |
| The ledger cannot support proof verification, hold, or finalized-state readback | Do not adopt it as a destination with the same guarantees. Explicitly change the candidate or offering scope |
| Independent operation cannot meet acceptable latency, cost, or recovery requirements | Do not promote to a production proposal; reconsider the method and use case based on observed constraints |
| Independent cryptographic audit, old-state migration, or workflow responsibilities are not accepted | Do not promote to a service handling real assets. Continue with research / evaluation configurations |
| End-to-end gates for continuous trading, cancellation, and contention fail | Resolve failures under the existing contract. Do not count the objective as achieved by changing product names or comparison targets |

## 9. What This Formulation Work Completed

- Competitive treatment of 18 targets including Canton, and comparison decisions C01–C09.
- Initial customers and use cases, strategies S01–S06, offering units, and contractual integration conditions.
- Adoption and development sequence G0–G4, metrics, mappings to research contracts, and reconsideration conditions.
- A comparison baseline with fixed evidence and current unmet objectives.

What remains necessary is completion evidence for existing product work, customer evaluation conditions, a contract for one integration destination, and acceptance of production operations. Do not confuse carrying out those steps with completion of this strategy document.

[comparison]: ../research/COMPETITIVE_DECISIONS_2026-09-05.md
[survey]: ../research/COMPETITORS_EX_CANTON_2026-09-05.md
[canton]: ../research/CANTON_NETWORK_2026-09-05.md
[baseline]: ../research/STRATEGY_BASELINE_2026-09-05.json

[fable]: ../research/CANTON_FABLE_5_1_MAX_REVIEW_2026-09-05.md
[update]: ../research/STRATEGY_UPDATE_2026-09-05.json
[independence]: ../../../aethel/docs/FOUNDATION_INDEPENDENCE_JA.md
