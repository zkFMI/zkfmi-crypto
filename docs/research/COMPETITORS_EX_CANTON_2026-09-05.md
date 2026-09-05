# ZKFMI Competitor Survey: Excluding Canton

Research and information-check date: **2026-09-05**. Scope: **17 products and platforms** addressing customer problems that overlap with ZKFMI as a whole (QOMM / OCLOB / zkPI / DeFMI / DeKYX / DeCCP). This is not a comparison for selecting cryptographic libraries.

**Supplementary research:** The same-day [comparison decisions v1.0](COMPETITIVE_DECISIONS_2026-09-05.md) and [strategy v1.0](../strategy/ZKFMI_STRATEGY_2026-09-05.md) incorporate Renegade's matching proofs and market mechanism, updated ZKFMI implementation evidence, and adoption and development priorities. This document preserves the scope confirmed in the initial survey.

## 1. Key Judgments

**Priority comparison targets are Renegade for confidential trading, Arcium for MPC infrastructure, Corda / Kinexys / Progmat for financial institutions, and Ownera for connecting trading and settlement.** These priorities are this research's judgments based on functional overlap and customer relationships, not market-share rankings.

- **“Handling secret orders with MPC and settling with ZK” alone is not a differentiator.** Renegade publicly documents that architecture. Compare the information operating nodes can read, binding to order sets and admission order, partial fills, eligibility and credit reservations, and continuity between fill and settlement proofs. [R1][r1] [R2][r2]
- **General-purpose confidential computation is both a competitor and a procurement candidate.** Arcium's current Cerberus uses a dishonest-majority, detect-and-abort model; Zama uses FHE and distributed key management. Assess ZKFMI's value in turning market rules, authorization, asset reservations, and delivery into one verifiable workflow, rather than merely owning MPC infrastructure. [A2][a2] [Z1][z1] [Z3][z3]
- **For financial institutions' adoption decisions, differences beyond cryptographic methods are substantial.** Corda's adoption for securities settlement, Kinexys's banking services, Fnality's payment arrangements, and Progmat's ST projects illustrate barriers involving customer connectivity and existing practice. ZKFMI's research implementation and these commercial paths are not at the same maturity level. [C1][c1] [K1][k1] [F1][f1] [J1][j1]
- **Cross-chain connectivity is a competitive field in its own right.** Ownera, Chainlink, and Swift address trade-instruction coordination, external data / policy / messaging integration, and interbank payment coordination, respectively. Seeking adoption of zkPI requires comparing how it connects to existing ledgers as well. [O1][o1] [L1][l1] [S1][s1]
- **A candidate differentiator is “verification of market rules while preserving confidentiality, with settlement bound to the result.” Exclusive novelty and superiority remain unestablished.** Do not conclude that competitors lack the same functionality merely because it could not be confirmed in public materials.

## 2. Scope and Reading Guide

### Selection Criteria

Targets were selected if they met at least one of the following criteria and could be checked against official specifications, operator announcements, regulatory materials, or papers by the authors.

1. Infrastructure adopted by financial institutions for ledgers, securities issuance, collateral, or settlement.
2. Systems implementing or proposing confidential-order matching, confidential balances, or third-party-verifiable transactions.
3. Alternative confidential-computation, proof, or interoperability infrastructure for implementing the above.

**Canton itself was excluded from the comparison.** For companies supporting multiple networks, compare their non-Canton functionality. For example, the original source explicitly states that the MUFG / Progmat JGB repo demonstration dated 2026-08-13 uses Canton, so it is not counted as a “non-Canton track record” in this table. [J3][j3]

This is a survey of public materials. It did not connect to competitors' paid environments, audit code, execute trades, or measure relative performance. Do not rank fees, TPS, latency, or funding amounts using figures obtained under different conditions.

### Maturity Labels

| Label | Meaning in this document |
| --- | --- |
| Commercial case announced | An operator or adopter has announced execution of the target workflow. This does not mean an independent operational audit |
| Mainnet / Alpha announced | Announcement of network stage. Distinct from commercial delivery in a regulated market |
| Adoption / demonstration / planned | Preserve the stage stated in the announcement. Passage of a planned date alone does not establish live operation |
| Specification confirmed | Operating model checked in specifications. Operational status of a particular deployment is unverified |
| Unverified | Not confirmed in the materials read. Different from “unimplemented” or “unsupported” |

### Our Own Comparison Baseline

For ZKFMI, the current `defmi/README.md`, `defmi/POSITION.md`, Japanese qomm materials, and this repository's P0 acceptance report were reread on 2026-09-05. DeFMI is research software; it is not treated as having completed acceptance for independent operators, production security, or legal finality. On the native note path, **asset IDs and settlement metadata are public**; comparisons must not imply that every field is confidential.

At the time of reference, DeFMI HEAD was `153fe671e523ec573a6c6261f341423a49371f5d` and qomm HEAD was `61596e523a4249031ae2471c78bd2249fea8f49b`. This survey did not rerun the existing stack's E2E path. Discrepancies between an old incomplete-items table on the website and the current README were not used to select convenient feature claims.

For PQC, what has been verified is hybrid signatures/KEM in standalone `zkfmi-crypto` and 45 tests, including known-answer tests. Wiring into existing services has not been implemented; this does not establish quantum resistance for ZKFMI as a whole. [P0 acceptance report](../verification/P0_REPORT.md)

## 3. Comparison of Institutional Ledgers, Settlement, and Connectivity

| Target | Main competing task | Confidentiality and trust boundary | Confirmed stage and implication for comparison |
| --- | --- | --- | --- |
| **R3 Corda** | Securities / asset-state management and interinstitutional workflows | Data sharing among relevant parties and double-spend prevention by notaries. The model does not necessarily have notaries read all transaction contents | Adoption for CSD Prague's DLT settlement announced. Network governance and integration into financial workflows are strong comparison axes [C1][c1] [C2][c2] |
| **Kinexys / J.P. Morgan** | Bank payments, asset tokenization, and fund-related processing | Bank-operated services, private permissioned infrastructure, and products on public chains. Distinct from hiding inputs from the bank | Announced JPM Coin on Base, the first Fund Flow transaction, and other milestones on 2026-04-28. Competes through cash legs and customer relationships [K1][k1] [K2][k2] |
| **Fnality** | Institutional digital-cash settlement | A system backed by funds held in central-bank accounts. Price formation for secret orders is not its primary focus | Sterling FnPS confirmed in the latest BoE supervisory materials. Phased-operation conditions apply; do not treat all currencies and capabilities as complete [F1][f1] [F2][f2] |
| **Partior** | Cross-border payments and FX PvP | Settlement network of participating financial institutions. Specifications for confidential market computation are unverified in these materials | Separately announces live payment-network cases and a DvP **PoC** with OpenAssets. Alternative / integration candidate for cash legs and PvP [T1][t1] |
| **Ownera / FinP2P** | Asset distribution across ledgers and coordination of settlement instructions | Each institution's Router, adapters, and agreement among participating institutions. Depends on underlying ledgers and payment infrastructure | Publishes implementation APIs and orchestration plans. Close to the integration position of zkPI/SDK [O1][o1] [O2][o2] |
| **Swift shared ledger** | Interbank payment coordination and tokenised-deposit connectivity | Shared layer operated by Swift and bank-side ledgers. Describes paths using existing systems for final settlement | As of 2026-07-09, ready for initial use and preparing for bank live pilots. Does not state a full commercial transition [S1][s1] [S2][s2] |
| **Chainlink CCIP / CRE / ACE** | Integration of external information, policies, and cross-chain processing | DONs, TEEs, DKG, and related mechanisms. Combines access to private APIs with confidential computation | The 2026-05 explanation includes privacy capabilities and development cases. This does not mean all are production cases of regulated financial FMIs [L1][l1] [L2][l2] |
| **Progmat** | Japanese digital-securities issuance and administration, and integration with financial-product practice | Institutional roles such as issuers, trusts, and distributors are important. Hiding inputs from all relevant parties is unverified in these materials | ST project list and provider announcement of completed migration to Avalanche L1 confirmed. Directly competes in adoption within Japanese practice. Canton JGB projects excluded [J1][j1] [J4][j4] [J5][j5] |
| **BOOSTRY / ibet for Fin** | Japanese ST issuance and circulation, and consortium operation | Member-operated network and standardized ST handling. Detailed confidentiality scope requires deployment-specific confirmation | Official consortium explanation and participants' announcement of operation launch confirmed. A comparison target when implementing Japanese securities workflows from scratch [B1][b1] [B2][b2] |

### Corda: A Ledger with Limited Sharing Versus Confidential Computation

Corda 5.2's non-validating notary handles references to input states and similar information and need not receive the full contents of commands or signatures. Transaction participant validation and history retrieval, however, have separate disclosure boundaries. It would be incorrect to say either that all Corda nodes see the same information or that no participant can see transactions. [C2][c2]

**Comparison judgment:** Compare ZKFMI's target use case, “verifying the result of applying price rules or other parties' orders without revealing them,” with Corda's handling of rights and contract state within the same project. Distinguish R3's “Corda protocol” for Solana from the established Corda ledger product. The 2025-12 announcement planned a launch in the first half of 2026; that announcement alone does not establish actual operation as of September. [C3][c3]

### Kinexys / Fnality / Partior: Strong Cash Legs and Customer Access

Kinexys services combine banking deposits, customer relationships, and operational responsibility. Fnality's supervisory position and Partior's payment paths likewise cannot be replaced simply by implementing cryptographic crates. [K1][k1] [F1][f1] [T1][t1]

**Comparison judgment:** For near-term projects, compare proposals to verify external computation with zkPI and connect to existing cash legs, as well as proposals to replace these systems entirely. Such connectivity is a proposal from this research; no ZKFMI integration track record exists.

Fnality's own 2023 announcement refers to its first live payment, while the BoE 2025–26 report describes the start of operations subject to limits in December 2024. Because the events' descriptions and dates differ, they have not been merged into a single “full production launch date.” [F1][f1] [F2][f2]

### Ownera / Swift / Chainlink: Instruction Formats Alone Are Not Enough to Compete

FinP2P has concrete mechanisms that convert trade intents into agreement among multiple Routers and commands for each ledger. Swift is also building a shared layer that coordinates commitments between banks. **Do not treat “connecting existing ledgers” or “standardizing instructions” alone as unique zkPI value.**[O1][o1] [S1][s1]

Chainlink describes confidential HTTP processing using TEEs and distributed key generation. ZK proofs, joint computation over secret inputs through MPC, and plaintext processing inside a TEE place trust differently. Do not infer equivalent third-party verification capabilities from a vendor's use of the single word “verifiable.” [L1][l1]

**Comparison judgment:** Show adopters which propositions are bound into an instruction, what they can reverify beyond signatures, and where responsibility lies for duplicate execution or failure of one leg. Distinguish agreement across ledgers from completed legal delivery.

### Progmat / ibet for Fin: Compare Connections to Japanese Institutions and Operations

Connectivity to organizations responsible for ST issuance, rights transfers, distribution, and record administration is a competitive axis. Progmat's ST project list and ibet for Fin's descriptions of its network and standard contracts show that this field involves workflows beyond cryptographic implementation. [J1][j1] [B1][b1]

For the non-Canton ledger path, Avalanche's announcement dated 2026-02-25 describes a plan to migrate from Corda to a dedicated Avalanche L1, and AvaCloud's official post subsequently announces completion. The latter displayed the relative date “1mo” when checked, so no exact completion date is inferred. Migration completion is treated as a provider announcement, not independently verified operating status or performance. [J4][j4] [J5][j5]

**Comparison judgment:** Prioritize these comparisons in Japanese projects. Specify how confidential eligibility / guarantee-capacity checks and collateral computations could be incorporated into existing registration, trust, and settlement workflows. The investment-trust release original is dated 2026-08-28 and was posted on the Web on 09-03; it explicitly describes a demonstration without solicitation or sales to outside investors. Do not reinterpret a “demonstration in a live operating environment” as the launch of commercial sales. This original alone does not identify the chain used, so it is not used as evidence of a non-Canton ledger track record. [J2][j2]

## 4. Comparison of Confidential Orders and Confidential Asset Trading

| Target | Market and verification mechanism | Confidentiality and trust boundary | Confirmed stage and implication for comparison |
| --- | --- | --- | --- |
| **Renegade** | Pairwise MPC generates a collaborative SNARK and updates balances on-chain | The connected relayer can read orders and balances of the wallets it serves in plaintext. These are hidden from other relayers | Official site announces Arbitrum One mainnet launch. A close comparison for OCLOB / confidential trading [R1][r1] [R2][r2] [R3][r3] |
| **Prime Match** | MPC inventory matching between a financial institution and its clients | Participant / bank threat model defined by the authors. Not the same node configuration as ZKFMI | A 2023 paper reports live operation at J.P. Morgan. A prior implementation refuting claims of being the first financial MPC system [P1][p1] |
| **Penumbra** | Shielded pool, per-block batch DEX, and transaction / claim proofs | Current specifications disclose swap input assets and amounts. Different from the confidentiality scope of shielded transfers | Implementation specifications checked. Sealed-bid batch swaps are explicitly a future feature in the reference specification [N1][n1] [N2][n2] |
| **Dusk** | Ledger, confidential transfers, and selective disclosure designed with regulated assets in mind | Depends on transaction model and application. A public account model also exists | Mainnet connection specifications and collaborations with NPEX and others confirmed. Actual operation of each financial market and applicability of licenses need individual checks [D1][d1] [D2][d2] |

### Renegade: The First Confidential-Trading Implementation to Compare

An architecture producing ZK proofs as MPC outputs and linking them to settlement of confidential state already exists. However, under the official relayer specifications, a customer's delegated relayer reads orders and wallet balances in plaintext. Customers also have the option of operating their own relayers. [R1][r1] [R2][r2]

**Comparison judgment:** If ZKFMI actually maintains secret sharing from the customer to the computing nodes, there is a difference at this delegation boundary. It must nevertheless show the node group's collusion conditions, share delivery from corporate clients, and whether any gateway can decrypt. Further distinguish proofs of order validity from proofs that price-time priority was applied without omitting relevant orders. No audit sufficient to conclude that Renegade does not support the latter has been conducted.

### Prime Match: A Prior Case of Commercial MPC

The authors' paper abstract reports privacy-preserving inventory matching and live operation at J.P. Morgan. This confirms execution at the time of publication, not a fresh verification of current operation in 2026, volumes, or the scope of contracted services. [P1][p1]

**Comparison judgment:** Compare computation targets, tolerated collusion, behavior under faults, propositions external auditors can check, and integration scope through settlement. Do not compare speed without reruns under identical conditions.

### Penumbra: The “Private DEX” Label Does Not Establish Confidentiality Scope

Official specifications distinguish disclosure for ordinary shielded transfers from swap inputs. Swaps reveal assets and amounts; subsequent claims produce confidential outputs. The sealed-bid version is described as a future extension. [N1][n1] [N2][n2]

**Comparison judgment:** In comparisons with OCLOB/QOMM, place “whether third parties can link wallets” and “whether order prices and quantities can be read before execution” in separate rows. Protection of execution ordering in batch settlement is also a different market design from price-time priority in a continuous order book.

### Dusk: Overlap in Workflow Design for Regulated Assets

Dusk describes an architecture oriented toward confidential transfers, selective disclosure, asset lifecycles, and DvP. Its exchange integration guide, however, specifies the public account model Moonlight, so do not treat all Dusk transactions as uniformly confidential. [D1][d1] [D2][d2]

**Comparison judgment:** Do not use collaboration with NPEX or a particular operator's license as evidence that authorization extends to arbitrary applications on Dusk. Apply the same standard to ZKFMI: the existence of DeCCP code alone does not establish a legal CCP or novation.

## 5. Comparison of Confidential-Computation and Application Infrastructure

| Target | Capabilities provided | Confidentiality and trust boundary | Confirmed stage and implication for comparison |
| --- | --- | --- | --- |
| **Arcium** | General-purpose MPC, MXEs, and confidential apps coordinated with Solana | Current Cerberus preserves confidentiality assuming at least 1 member is honest and aborts when abnormalities are detected. Availability is a separate condition | Official site displays Mainnet Alpha. Competitor / procurement candidate for MPC development infrastructure and confidential trading apps [A1][a1] [A2][a2] |
| **Zama Protocol** | Computation over encrypted state through FHE, confidential tokens, and access control | Consider FHE computation separately from decryption authority and distributed key management. KMS assumes a strong honest majority | Officially announced mainnet launch on 2025-12-31 and a confidential auction in 2026-01. Competitor / procurement candidate for general-purpose confidential financial apps [Z1][z1] [Z2][z2] [Z3][z3] |
| **Aztec** | Private/public applications on Ethereum L2 and client-side proving | Private witnesses remain on clients. Distinct from joint computation over secret inputs from multiple companies | Alpha V5. A critical proving-system vulnerability was announced on 2026-08-07, with a fix planned for V6 [X1][x1] [X2][x2] |
| **Hyperledger Fabric** | Permissioned ledger, contract execution, and private data between organizations | Actual data goes to authorized organizations' peers; hashes go to the whole channel. Orderers do not receive private data | Official specifications checked. A comparison target for financial institutions building in-house on existing infrastructure [H1][h1] |

### Arcium: Do Not Assume Its MPC Has Weaker Security Assumptions Than Ours

Current docs explicitly describe Cerberus as a dishonest-majority, detect-and-abort model. The comparison “we are secure because we have multiple nodes; competitors are centrally controlled” does not hold. Separate confidentiality protected by at least one honest party from availability that allows processing to complete. [A2][a2]

**Comparison judgment:** Base the rationale for ZKFMI development on how reserved assets, eligibility, credit, order sequencing, and settlement authority are bound into one workflow evidence trail, rather than reimplementing MPC infrastructure itself. Building the same workflow on Arcium is also an alternative to evaluate.

### Zama: Confidential Computation and Auctions Have Reached Public Deployment

Official announcements describe mainnet launch and execution of a sealed-bid auction using FHE. A 2025 testnet article alone cannot support saying “FHE is not yet practical.” [Z2][z2]

The public key-management specification assumes a strong honest majority with `t < n/3`, where `n` is the number of participants and `t` the number of tolerated faulty or malicious participants, to guarantee completion of key generation and decryption. This differs in collusion conditions and completion guarantees from Arcium's detect-and-abort model, which preserves confidentiality with at least one honest party. [Z3][z3] [A2][a2]

**Comparison judgment:** When comparing FHE-based order / collateral computation with MPC + proofs, align decryption authority, supported computation types, completion waits, failures and retries, and information auditors can trace. This research did not measure performance differences between them.

### Aztec: Separate Its Value as an Application Platform from the State of a Particular Version

Executing and proving private functions on clients provides an alternative for applications using confidential holdings state. [X1][x1] However, the official notice dated 2026-08-07 describes a critical proving-system vulnerability in V5 Alpha and a fix planned for V6. The notice consulted during this research did not confirm that the fix had been completed. [X2][x2]

This is not a conclusion that every Aztec version is permanently unsafe. Adoption evaluation should recheck the target version, completion of fixes, and migration evidence. ZKFMI also lacks a completed independent audit; do not position it as production-quality merely because a competitor is at the Alpha stage.

### Fabric: Separate Confidential Sharing Between Organizations from Computation Hidden Even from the Computing Entities

Private Data Collections send actual data to authorized peer groups and leave hashes for other peers. This suits limiting which organizations see data, but does not by itself describe MPC that hides computation inputs even from authorized peers. [H1][h1]

**Comparison judgment:** If a workflow does not require secrecy from everyone, existing interorganizational governance and Fabric may sometimes provide a clearer adoption rationale. Justify adopting ZKFMI through confidentiality boundaries and external verification requirements that access control alone cannot satisfy.

## 6. Evidence to Show Before Claiming Differentiation

The following are verification tasks for ZKFMI derived from the comparison, not a list of competitors' weaknesses. This survey does not authorize or begin new implementations or experiments.

| Candidate proposition | Required evidence | Main comparison targets |
| --- | --- | --- |
| Orders and pricing rules hidden even from operators | List of entities with visibility from corporate-side share generation through MPC and settlement; collusion conditions; plaintext decryption points; key administrators | Renegade, Arcium, Prime Match |
| Trade-execution rules can be verified afterward | Propositions binding the target order set, admission ordering, eligibility, price/time priority, and partial fills, plus independent verification procedures | Renegade, Penumbra, market apps on Zama |
| Computation results match delivery | Evidence binding the same commitment to computation, instructions, asset reservations, and consumption of both legs, with state unchanged by retries or double-spend attempts | Ownera, Corda, Chainlink |
| Financial institutions can adopt the system | Allocation of issuance / custody / registration / cash-leg responsibilities, disaster recovery, supervisory disclosure, participant entry and exit, and legal settlement completion | Kinexys, Fnality, Partior, Swift, Progmat, ibet for Fin, Dusk |
| Long-term confidentiality and verifiability | Threat model distinguishing signatures, KEM, commitments, proofs, TLS, and stored data, plus migration, revocation, and reverification paths | Entire adopted configuration, including Arcium, Zama, and Aztec |

Labels such as “encrypted,” “distributed,” and “verifiable” alone do not satisfy the table above. In particular, a proof of correct circuit evaluation, the best execution within a target set, and the statutory best-execution obligation are not the same proposition.

### What Remains Open in the PQC Comparison

The materials checked here do not comprehensively establish each product's PQC status across signatures, key exchange, commitments, proofs, and ledger consensus. **Do not score unverified items as unsupported.** Neither the FHE label nor adoption of a PQC signature library makes a whole system quantum-resistant. For our own system too, distinguish completion of standalone P0 from migration of the entire existing settlement stack.

## 7. Proposed Competitive Actions

1. **Make Renegade and Arcium the first technical comparison targets.** Specify differences in which entities can read orders and which market rules output proofs guarantee. Do not begin with cryptographic names.
2. **Always include Progmat and ibet for Fin in Japanese proposals.** Explain eligibility, asset registration, delivery, disclosure, and existing workflow responsibilities alongside anonymity.
3. **Consider integration proposals that acknowledge overlap with Ownera, Chainlink, and Swift.** Show which aspects of existing instructions become independently verifiable by adding zkPI.
4. **View Kinexys, Fnality, and Partior as potential integration destinations as well as cash-leg competitors.** Available connection contracts / APIs and legal delivery conditions are unverified; do not present them as implemented.
5. **Limit external wording to candidate configurations and the current acceptance stage.** This research does not support claims such as “the world's first financial MPC,” “the only confidential trading,” “faster than competitors,” or “the entire system is quantum-resistant.”

This research compares 17 targets and does not claim complete market coverage. It prioritizes targets directly relevant to deciding what to buy, build, or connect at each ZKFMI layer, rather than listing individual existing exchanges, custodians, issuer SaaS products, and every FHE / TEE / MPC vendor.

## 8. Primary Sources and Scope Checked

All links were checked on 2026-09-05. Publication dates are given only where explicitly stated in the body or link title. Undated specifications use the version and check date as the reference. Corporate announcements are treated as facts reported by those companies, not equated with regulatory materials, adopter evidence, or independent operational audits.

| ID | Primary source and publication date / version | Main points checked |
| --- | --- | --- |
| C1 | [R3: Corda adoption by CSD Prague, 2024-11-07][c1] | Announcement of adoption for securities settlement |
| C2 | [Corda 5.2: non-validating notary][c2] | UTXO uniqueness and disclosure differences between participant nodes and notaries |
| C3 | [R3: Corda protocol announcement, 2025-12-12][c3] | Distinguishing a separate Solana product and its planned status |
| K1 | [Kinexys milestones, 2026-04-28][k1] | JPM Coin on Base, first Fund Flow transaction, and other milestones |
| K2 | [Kinexys product page][k2] | Bank payments and private permissioned asset infrastructure |
| F1 | [BoE: FMI Annual Report 2025–26][f1] | Fnality's supervised status and operations subject to limits |
| F2 | [Fnality: First Sterling payment announcement, 2023-12-14][f2] | Status of the initial live payment |
| T1 | [Partior official site][t1] | Distinguishing payments / PvP, participating banks' live cases, and a DvP PoC |
| O1 | [Ownera: Intent-Based Orchestration][o1] | Proposal/approval among Routers and coordination across ledgers |
| O2 | [Ownera: Integration Use Case Guides][o2] | API configuration for trading, issuer, and payment connectors |
| S1 | [Swift: Shared ledger ready for initial use, 2026-07-09][s1] | Live-pilot preparation and final settlement through existing systems |
| S2 | [Swift: March 2026 newsletter][s2] | Besu foundation, Swift operation, and bank-side asset / liquidity management |
| L1 | [Chainlink: Privacy architecture, 2026-05-21][l1] | TEEs, DKG, confidential HTTP, private tokens, and development cases |
| L2 | [Chainlink: CCIP, 2026-04-22][l2] | Boundary between interoperability capabilities and the messaging protocol |
| J1 | [Progmat ST project track record][j1] | Ongoing list of Japanese ST projects |
| J2 | [Progmat: Demonstration of a Japanese-domiciled tokenized investment trust, original 2026-08-28 / posted 09-03][j2] | Demonstration without external solicitation or sales. The original alone does not identify the chain used |
| J3 | [MUFG: JGB repo demonstration, 2026-08-13][j3] | Excluded from the non-Canton track record because it uses Canton |
| J4 | [Avalanche: Progmat migration plan, 2026-02-25][j4] | Plan to migrate from Corda to a dedicated Avalanche L1 |
| J5 | [AvaCloud official post: Progmat migration completion][j5] | Provider announcement of completed migration. Do not infer an exact date from a relative date |
| B1 | [BOOSTRY: ibet for Fin consortium explanation][b1] | Network and ST standards checked through the official search-index summary. Body extraction was unsuccessful |
| B2 | [SBI and others: ibet for Fin operation launch, 2021-06-15][b2] | Participants' announcement of operation launch. Do not infer current scale from this |
| R1 | [Renegade: Role of relayers][r1] | Plaintext visibility into connected wallets, pairwise MPC, and self-operated relayers |
| R2 | [Renegade: collaborative zkSNARK][r2] | Proofs as MPC outputs and on-chain settlement |
| R3 | [Renegade official site][r3] | Announcement of Arbitrum One mainnet launch |
| P1 | [Prime Match paper abstract and bibliographic record, 2023][p1] | Authors' report of confidential inventory matching and live operation at the time |
| N1 | [Penumbra: Batch Swaps specification][n1] | Distinguishing V1 from the future sealed-bid version |
| N2 | [Penumbra: Privacy Features][n2] | Disclosure scope for transfers, swaps, claims, and LPs |
| D1 | [Dusk official site][d1] | Asset workflows, selective disclosure, and relationships with NPEX and others |
| D2 | [Dusk: Exchange integration][d2] | Mainnet endpoint and specification of Moonlight public accounts |
| A1 | [Arcium official site][a1] | Mainnet Alpha label |
| A2 | [Arcium: MPC protocols][a2] | Cerberus threat model, abort, and availability |
| Z1 | [Zama official site][z1] | Architectures and cases of confidential financial apps using FHE |
| Z2 | [Zama: Mainnet Season 1, 2026-02-11][z2] | Mainnet launch date and confidential auction execution |
| Z3 | [Zama KMS: Threshold cryptography concepts][z3] | Distributed key management, strong honest majority, and completion guarantees for key generation / decryption |
| X1 | [Aztec official site][x1] | Client-side proving and private/public apps |
| X2 | [Aztec: Alpha V5 vulnerability notice, 2026-08-07][x2] | Status of that version and planned V6 fix. Completion has not been confirmed |
| H1 | [Hyperledger Fabric: Private data][h1] | Actual data to authorized peers, hashes to the whole group, and the orderer boundary |

Retrieval of the Renegade whitepaper body returned 502, and parts of the Zama litepaper produced a content-type error in the browsing tool. The table summaries rely on official FAQs, specifications, and announcements that were successfully retrieved; unavailable originals are not treated as read in full. BOOSTRY's body-extraction limitation is also stated in B1. Items requiring further implementation audits or regulatory confirmation remain as the unverified items in each section.

[c1]: https://r3.com/r3s-corda-selected-as-first-authorized-dlt-platform-for-european-dlt-pilot-regime/
[c2]: https://docs.r3.com/en/platform/corda/5.2/developing-applications/ledger/notaries/non-validating-notary.html
[c3]: https://r3.com/r3-announces-launch-of-corda-protocol-on-behalf-of-r3-foundation-to-bring-institutional-grade-curated-yield-to-solana/
[k1]: https://www.jpmorgan.com/payments/newsroom/kinexys-milestones-2026
[k2]: https://www.jpmorgan.com/kinexys/index
[f1]: https://www.bankofengland.co.uk/financial-stability/financial-market-infrastructure-supervision/report/fmi-annual-report-2025-26
[f2]: https://fnality.com/news/fnality-commences-initial-phase-of-sterling-payment-operations-in-a-world-first?showiframe=true
[t1]: https://partior.com/
[o1]: https://finp2p-docs.ownera.io/docs/orchestration-plan-capabilities
[o2]: https://finp2p-docs.ownera.io/docs/integration-to-finp2p-api
[s1]: https://www.swift.com/news-events/press-releases/swifts-blockchain-ledger-ready-use-17-banks-set-pioneer-tokenised-cross-border-payments-trusted-global-infrastructure
[s2]: https://www.swift.com/news-events/newsletters/faster-payments-smarter-standards-and-swifts-blockchain-based-shared-ledger-road-ahead-global-finance
[l1]: https://chain.link/blog/how-chainlink-is-bringing-privacy-to-blockchains
[l2]: https://chain.link/blog/ccip-cross-chain-standard
[j1]: https://progmat.co.jp/concept/st_list/
[j2]: https://progmat.co.jp/news/2026-09-03-press/
[j3]: https://www.mufg.jp/dam/pressrelease/2026/pdf/news-20260813-002_ja.pdf
[j4]: https://www.avax.network/about/blog/progmat-migrates-2b-tokenized-securities-to-avalanche
[j5]: https://www.linkedin.com/posts/avacloud_progmat-inc-japans-largest-security-token-activity-7482536047919136768-zSy5
[b1]: https://boostry.co.jp/blog/ibet-for-Fin
[b2]: https://www.sbigroup.co.jp/news/pdf/group/2021/0615_a.pdf
[r1]: https://help.renegade.fi/hc/en-us/articles/32530262853395-What-are-relayers-on-Renegade
[r2]: https://help.renegade.fi/hc/en-us/articles/32529961385363-What-is-a-collaborative-zkSNARK
[r3]: https://renegade.fi/
[p1]: https://arxiv.org/abs/2310.09621
[n1]: https://protocol.penumbra.zone/main/dex/swap.html
[n2]: https://guide.penumbra.zone/overview/privacy
[d1]: https://dusk.network/
[d2]: https://docs.dusk.network/developer/integrations/exchanges/
[a1]: https://www.arcium.com/
[a2]: https://docs.arcium.com/multi-party-execution-environments-mxes/mpc-protocols
[z1]: https://www.zama.org/
[z2]: https://www.zama.org/post/zama-developer-program-mainnet-season1-building-for-the-long-game
[z3]: https://github.com/zama-ai/kms/blob/main/docs/getting-started/concepts.md
[x1]: https://aztec.network/
[x2]: https://aztec.network/blog/alpha-v5-proving-system-vulnerability
[h1]: https://hyperledger-fabric.readthedocs.io/en/latest/private-data/private-data.html
