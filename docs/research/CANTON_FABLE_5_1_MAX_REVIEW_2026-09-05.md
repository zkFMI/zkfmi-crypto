# Reassessment of the 18-Target Comparison Including Canton — Claude Fable 5.1 / effort max

Translator's note: This is a faithful English translation of the existing review, not a fresh review of its sources.

Review date: **2026-09-05** (actual times verifiable in this session: first Web retrieval saved at 11:56:32 UTC, curl retrieval at 12:02–12:03 UTC, local checks at 12:07:04 UTC, and additional retrieval during revision at 12:30 UTC. These exclude the previous assignee's working times). Author: Claude Fable 5.1 (model id `claude-fable-5-1`, maximum effort, no subagents or substitute models). Delegated by the parent Codex task, the author consulted the [initial survey excluding Canton][survey], [comparison decisions v1.1][decisions], [strategy v1.1][strategy], [baseline file][baseline], and [supplementary Canton research v1.0][canton] on a read-only basis and reassessed the comparison decisions and strategy across 18 targets with Canton included. **This file is the only write target for this review.** No code, Cargo files, lock files, other repositories, README files, or existing documents were changed. No commits, pushes, external communications, experiments, builds, tests, or authentication configuration changes were made.

Underlying user decision: **Remove Aethel dependencies from the entire ZKFMI foundation.** Align the dependency direction so that Aethel is a business application that uses the foundation. Because the parent task is separating the code, this review takes that policy as a premise for the comparison and strategy, but **does not state that the separation has been implemented completely** (at 12:07 UTC, uncommitted deletions of `*-aethel` crates were observed in the defmi / dekyx / deccp working trees. This is not evidence of completion).

Parent integration note (2026-09-05): Fable's technical judgments have been retained. Long source quotations were replaced with summaries, and the generalization from the single Cantex example that all applications are AMM-centered was removed. Subsequent verification of foundation separation and OCLOB will be updated in the parent task's comparison v1.2 and a separate snapshot.

Revision v1.1 (12:30 UTC): Incorporated three points raised by the parent task. (1) Corrected the description of allocation withdrawal to match the original CIP-0056 text and Splice reference interface (Sections 2.4, 3.3, 5.1, and 9). (2) Recounted primary sources, distinguishing the number referenced from the number successfully retrieved (Sections 1 and 9). (3) Restricted retrieval times to actual times verifiable in this session (opening paragraph and Section 1). The conclusions are unchanged.

## 0. Conclusions

1. **Canton is the highest-priority comparison target and also a candidate integration destination and implementation platform.** Retain this position (the preceding research's conclusion, comparison decision C09, and strategy S04). Official Canton materials describe visibility in which only the necessary parties read plaintext, re-execution verification by the parties' validators, and multi-application DvP in one transaction on the same synchronizer. Primary sources document Mainnet operation, commercial applications, and Japanese PoCs. [CN1][cn1] [CN3][cn3] [CN7][cn7] [CN10][cn10] [J3][j3] [J6][j6]
2. **Canton's own design documents, rather than an absence of documentation, confirm that confidential computation that withholds inputs even from the computing entities is outside the Canton protocol's scope.** The 2020 Canton whitepaper explicitly states that advanced cryptography, including MPC, is computationally expensive and that Canton chooses to limit visibility under stronger trust assumptions. The official Canton blog in 2025 likewise characterizes ZKP for general-purpose privacy as experimental. [CN17][cn17] [CN18][cn18] This supports an axis on which ZKFMI could demonstrate a difference. However, no statement prohibits applications on Canton from integrating external MPC or ZK, so this cannot be described as “impossible on Canton.”
3. **Canton also has a standard for “advance reservation → atomic settlement.”** The Canton Network Token Standard (CIP-0056) defines allocations that lock assets until a deadline and all-or-nothing settlement in which a settlement app executes all transfers in one transaction. Reservation and DvP, as feature names, do not establish ZKFMI's uniqueness. The difference is whether the correspondence between reservations and orders is hidden from matching nodes or read in plaintext by the registry, app, and validator. [CN26][cn26]
4. **In Japan, three Canton PoCs are proceeding in parallel in areas adjacent to ZKFMI's target workflows.** These are JSCC / Mizuho / Nomura (2026-04-20, JGB collateral management), MUFG / Progmat (2026-08-13, JGB repo), and the Progmat/DCC WG (launched 2026-05, report targeted for 2026-10). All are at the launch or investigation stage and do not establish commercialization or completed legal delivery. [J6][j6] [J3][j3] [J7][j7] The strategy should avoid proposing a direct replacement for the same workflow and focus on the portions requiring inputs to remain hidden even from the computing entities.
5. **Retain all of C01–C09. Changes are limited to additional evidence and more precise wording.** Retain the direction of S01–S06 and G0–G4 as well, adding Token Standard mappings to the G0 decision questions, G2 Canton acceptance path, and S02 adapter contract.
6. **Local evidence has advanced beyond the baseline time but remains smoke_only.** Under the continuous-trading contract `oclob-native-cycle-v1`, post-baseline runs 005 and 006 completed two rounds of matching and settlement. The single-host setup, absence of independent operators and WAN, and trust in the configured DeFMI read service are unchanged. [L16][l16] [L7][l7]
7. **The preceding research contains no fatal error.** Corrections concern the Global Synchronizer Foundation's renaming to Canton Foundation on 2025-09-22, the source for “network of networks,” the counting of pilot participants, and a retrievable URL for the Tradeweb announcement (Section 6).

## 1. Verification Method

- Of the 15 primary sources cited by the preceding research (CN1–CN14 and J3), 14 were opened and checked against the original text. CN14 (investors.tradeweb.com) timed out and could not be retrieved; its content was checked using Canton's official republication of the same announcement (CN29). In addition, 23 new sources were opened: the privacy chapter of the Daml ledger model, the Canton whitepaper, official Canton blogs and FAQ, Super Validator components, the Token Standard (the CIP text and Splice reference interface), Canton Foundation, DTCC / Visa / Tradeweb announcements, Japanese announcements concerning JSCC / Mizuho / Nomura and Progmat, and the FSA page establishing PIP (20 sources, CN15–CN34, and three, J6–J8; Section 9).
- Count breakdown (no duplicate URLs):

| Category | Count | Breakdown |
| --- | --- | --- |
| Primary sources referenced with IDs | 38 | 15 existing (CN1–CN14, J3) + 23 new (CN15–CN34, J6–J8). All URLs are distinct. CN14 and CN29 are different URLs for the same announcement, so there are 37 documents |
| Successfully opened and checked | 37 | All except CN14. Body text for CN15 and CN16 was extracted with curl; J3 and CN17 were downloaded as PDFs and extracted locally (CN17: pages 1–3 only); J8 and CN34 were opened with curl during revision (12:30 UTC) |
| Could not be retrieved | 1 | CN14 (timeout). Replaced by CN29 |
| Pages opened without assigning IDs | 8 | canton.network (home page, global-synchronizer), docs.cantex.io (home page), docs.sync.global / docs.dev.sync.global (token standard), progmat.co.jp/news, fsa.go.jp (2026-02-27 support decision under the “FinTech Proof-of-Concept Hub”), docs.daml.com/daml/intro/7_Composing.html (table of contents only). Not used as evidence for the body text |
| URLs with failed retrieval attempts and no IDs | 7 | tradeweb.com (403), businesswire.com (403), jpx.co.jp JSCC (403), nomuraholdings.com (403), ledger-privacy in docs.digitalasset.com 3.3 and 3.4 (404), MUFG English PDF (unread) |

- Retrieval times (only those verifiable in this session's records): saved file for the first Web retrieval batch, 11:56:32 UTC (batch start time unrecorded); second batch, 11:59:00 UTC; third batch, 12:00:03 UTC; Daml docs retrieval with curl, 12:02:59–12:03:02 UTC; local git and ledger checks (`date -u`), 12:07:04 UTC; CIP-0056, Splice, and FSA page retrieval during revision, 12:30:14–12:30:15 UTC.
- Retrieval methods: Web retrieval tool, HTML body retrieval with `curl` (the Daml docs returned only a table of contents, so their body text was extracted with curl), and local PDF extraction (MUFG announcement and Canton whitepaper).
- Unavailable sources: `investors.tradeweb.com` (timeout) and `tradeweb.com` (403) were replaced by Canton's official republication of the same text [CN29][cn29]. The JSCC (jpx.co.jp) and Nomura announcement pages returned 403, so the same announcement on Digital Asset's official blog [J6][j6] was used instead. The ledger model privacy page in `docs.digitalasset.com` 3.x returned 404; the equivalent chapter in `docs.daml.com` 2.10.6 [CN15][cn15] was used. The FSA page announcing the February 2026 support decision for the Payment Innovation Project could not be identified, so the review is limited to the statements in the MUFG and DA announcements.
- This review checks public materials. It did not run Canton nodes, use competing applications, measure performance, audit code, or contact customers. **No speed, cost, or superiority figures were measured, so none are claimed.** Companies' self-reported figures are cited only with their sources and dates and are not used for rankings.

## 2. Canton's Position (Within the Scope Confirmed by Primary Sources)

### 2.1 A Layered Structure, Not a Single Product

| Layer | What primary sources confirm | Implication for comparison |
| --- | --- | --- |
| Daml (contract language and ledger model) | Privacy is provided “on a need-to-know basis, at the subtransaction level.” A party learns only the portions affecting contracts in which it has a stake and their consequences. Disclosure is defined through informee / witness / projection / divulgence [CN15][cn15] | Business logic (signatory / observer / controller) determines visibility; the protocol enforces it |
| Canton protocol | Splits transactions into views and encrypts each view for a restricted set of recipients. The sequencer has “only two functions: ordering and delivery to recipients” and cannot read the encrypted contents. The mediator likewise learns no contents, receiving only each view's informee list and approvals/rejections [CN16][cn16] [CN4][cn4] | Separates ordering and confirmation coordination from content validation |
| validator / participant node | Hosts parties and stores only contracts in which its hosted parties are stakeholders. Daml re-execution takes place here [CN1][cn1] | A “validator” is not a public-chain node that validates the entire ledger |
| synchronizer (sequencer + mediator) | Sees only encrypted envelopes and metadata for ordering and routing. Does not independently validate Daml logic [CN3][cn3] [CN33][cn33] | Correctness of the contents depends on confirmation by the relevant participants |
| Global Synchronizer | A public synchronizer operated in a distributed manner by Super Validators (SVs). Each SV runs a sequencer, mediator, and CometBFT orderer; block production requires “agreement from more than 2/3 of SV nodes.” Fault tolerance is `f = floor((n-1)/3)` [CN22][cn22] [CN8][cn8] | BFT consensus on ordering, not validation of business contents |
| Canton Coin | Users burn Canton Coin to pay for synchronizer traffic, and coins are minted according to participation (burn-mint equilibrium) [CN23][cn23] [CN20][cn20] | Using the Global Synchronizer incurs operating costs (traffic). Amounts are unverified |
| Canton Foundation | Formerly Global Synchronizer Foundation. Renamed on 2025-09-22 (name change only). Responsible for Global Synchronizer development and governance and also operates its own SV node [CN24][cn24] [CN31][cn31] | Trust in governance remains (explicit in the trust model) [CN5][cn5] |
| application provider | Provides on-ledger Daml and off-ledger logic and authentication, and may submit users' transactions. Trusted “not to censor” and “to implement correct contract logic” [CN5][cn5] | Censorship or omission before admission is outside the protocol |

### 2.2 Who Reads What (Need-to-Know Visibility)

| Entity | What it can read | What it cannot read | Source |
| --- | --- | --- | --- |
| Party (signatory / observer / controller / choice observer) | Views for which it is an informee (actions, consequences, and necessary context). Divulgence may expose contracts in which it is not a stakeholder | For unrelated portions, it sees “neither the payload nor metadata about the participants and parties involved” | [CN15][cn15] [CN2][cn2] |
| Validator hosting a party | The trust model explicitly states, “Your validator sees all your data and could block you.” External party keys let the party retain its own signing authority, but this is not described as preventing visibility | — | [CN5][cn5] |
| counterparty | Plaintext of shared views. Trusted “not to leak shared secrets” | — | [CN5][cn5] |
| application provider (including off-ledger components) | Implementation-dependent. An entity operating off-ledger order admission and matching may handle plaintext (for example, Cantex “submits swaps offchain,” and CaviarNine provides the matching engine) | — | [CN5][cn5] [CN30][cn30] |
| sequencer | Encrypted envelopes, recipients, order, size, and timing | Contents (session keys are encrypted with informee participants' public keys) | [CN4][cn4] [CN16][cn16] |
| mediator | Each view's informee list, confirmation policy, and each participant's approval/rejection | Contents. It also helps hide participants' identities from one another | [CN16][cn16] [CN3][cn3] |
| Unrelated parties and other validators | Receive nothing | — | [CN2][cn2] |
| Auditor | Plaintext of relevant views if incorporated into the design as an observer or similar role. Portable proofs that verify only a proposition without receiving secrets are not part of the standard path described in the public materials | — | [CN2][cn2] |

The official docs explicitly describe “Timing Attacks”: inferences may be drawn from “when transactions occur, transaction sizes, and activity patterns” even without seeing the contents. They list batching and adding noise as design-level countermeasures. [CN2][cn2] The Canton FAQ likewise explains that infrastructure operators see “only the limited metadata necessary for ordering and consistency.” [CN20][cn20]

### 2.3 Global Synchronizer and Participation Conditions

- The go-live of the Global Synchronizer and Canton Coin was announced on 2024-07-01. The 31 participating organizations include SBI Digital Asset Holdings, Tradeweb, Broadridge, and Ownera. [CN9][cn9]
- On 2026-06-29, Logical Synchronizer Upgrade went live on Mainnet in Canton 3.5. Canton self-reported four protocol version upgrades since Mainnet launch. [CN10][cn10]
- SV count: Visa announced its participation as “one of 40 Super Validators” in its own announcement on 2026-03-25. [CN28][cn28] Canton's “45+” announcement (reportedly dated 2026-04-04) was found only in secondary republications; the primary source is unverified (Section 8).
- Validator participation: the FAQ describes “a public network. Anyone can apply to run a validator node (sponsorship is currently required).” [CN20][cn20] Evaluating Canton at G2 requires first confirming access conditions for evaluation networks (DevNet / TestNet, sponsor, and costs).
- Governance: handled by Canton Foundation (formerly GSF). Linux Foundation announcements dated 2024-07-01 and 2025-03-19 state that it supports GSF. [CN9][cn9] [CN25][cn25] The relationship with Linux Foundation after the renaming is unverified in primary sources (Section 8).

### 2.4 Boundaries of Atomic Composability and DvP

| Scope | Confirmed guarantee | Boundary | Source |
| --- | --- | --- | --- |
| Contracts on the same synchronizer | “A Daml transaction executes on a single synchronizer, and all input contracts must be assigned to that synchronizer.” Within that scope, updates across multiple applications and participants all commit or all abort | Does not include updates to off-ledger bank accounts or existing CSDs | [CN6][cn6] |
| Multiple synchronizers | Cash and securities contracts are reassigned to a common Settlement Sync and then exchanged in a single Daml transaction. “Either both transfers occur or neither does” | Reassignment is “a non-atomic procedure involving two confirmation requests on two synchronizers.” Contracts cannot be used while pending; if assignment fails, they remain pending until resolved | [CN7][cn7] [CN6][cn6] |
| Token Standard (CIP-0056, approved 2025-03-31, Final) | A holder locks assets as an allocation for a particular settlement. Once all allocations are available, the settlement app submits one transaction in which “all transfers settle or none do.” The lock lasts until the settlement deadline (“allocations are only valid until that deadline”; “become available again to their owner immediately thereafter”). **The CIP body contains no definitions of withdraw / cancel / revoke**; “withdrawing an allocation” appears only as an example in the description of metadata key `splice.lfdecentralizedtrust.org/reason`. The Splice reference implementation's Daml interface `Splice.Api.Token.AllocationV1` defines `Allocation_Withdraw`, controlled solely by the sender (a SHOULD that it “not cause settlement to fail before the `settlement.allocateBefore` deadline”), and `Allocation_Cancel`, jointly controlled by sender, receiver, and executor (normally delegated to the executor). The behavior of individual registry implementations (`allocation_withdrawImpl`) is unverified | Allocation contents (assets, quantities, and counterparties) are plaintext to the registry, app, and owner. Hiding the correspondence between reservations and orders from matching nodes is outside the standard's scope | [CN26][cn26] [CN34][cn34] |
| Ledgers outside Canton | Designs can map semantics through adapters / tokenization | Hold / commit / abort / finality on external ledgers are outside the Canton protocol's guarantees | [CN7][cn7] |

This provides two counterarguments for ZKFMI. First, settling multiple fills as a batch within DeFMI does not establish uniqueness (retaining the preceding research's conclusion). Second, **advance reservation is already standardized as well**. OCLOB's separation of `ReservationAdmission` / `ReservationPermit` (withholding ledger identifiers from matching nodes) creates a confidentiality boundary absent from Token Standard allocations and can be explained as a difference. Explain it in terms of “who can read the reservation linkage,” rather than the existence of the feature. [L1][l1] [L4][l4]

### 2.5 Difference from Confidential Computation

Canton limits visibility by **not distributing data** and by **having the parties that receive it validate it in plaintext**. This is a choice stated in Canton's own design documents, not an inference from missing documentation.

The 2020 whitepaper explains that advanced cryptography can provide confidentiality up to and including MPC, but its computational cost constrains scalability. Canton therefore chooses a design that explicitly trusts participants and restricts the scope of data sharing. [CN17][cn17]

The official Canton blog dated 2025-05-12 characterizes ZKP as having “limits in bearing the weight of institutional trust” and as “still experimental for general-purpose smart contract privacy,” while stating that Canton provides “smart contract privacy with full auditability.” [CN18][cn18] Neither the official blog dated 2025-08-14, the privacy model docs, nor the FAQ describes the use of ZK / MPC / FHE / TEE in the protocol. [CN19][cn19] [CN2][cn2] [CN20][cn20]

| Comparison unit | Canton's standard path | OCLOB's new CLI/Docker path | Judgment |
| --- | --- | --- | --- |
| Entities holding original orders in plaintext | Order submitter, its validator, contractual counterparty (and its validator), and the app provider operating off-ledger matching | Order submitter (corporate side). Each of the 7 nodes holds only its own share, and the coordinator does not receive the original. Collusion by 3 or more nodes reconstructs it [L4][l4] | An axis on which ZKFMI could demonstrate a difference. The difference disappears if the customer permits disclosure to its own validator and app operator |
| Correctness verification | Relevant participants decrypt views and re-execute Daml. The mediator does not independently validate Daml logic [CN3][cn3] | Joint proofs for MPC outputs, with 5 verification nodes validating the full proofs [L4][l4] | Both establish correctness for the “submitted input set.” Neither automatically eliminates pre-admission omission or censorship |
| Third-party verification | Plaintext verification if designed with an observer. A portable proof that does not receive secrets is absent from the standard path | Aims to verify propositions defined by zkPI. Requires explicit statements of the propositions checked by the verifier and its readback dependencies [L13][l13] | Potential ZKFMI value. However, adding a verifier to a Canton app cannot be ruled out |
| What can be stated as “absent from Canton” | The protocol's standard path does not include a mechanism to hide inputs from the computing entities (explicit in the design documents) | — | Can be stated definitively |
| What cannot be stated as “absent from Canton” | Applications on Canton integrating external MPC / ZK / TEE, or future additions to the protocol | — | Cannot be stated definitively. Public implementations are unverified (Section 8) |

Compare confidentiality through “the list of entities that read the originals,” “collusion conditions,” “observation of communication and timing,” and “information disclosed during recovery,” rather than node counts or method names. This is consistent with comparison decisions C02 and C04.

### 2.6 Operational and Adoption Maturity

| Case | Stage confirmed in primary sources | Classification (survey Section 2 terminology) | Do not generalize to |
| --- | --- | --- | --- |
| Global Synchronizer Mainnet | Go-live 2024-07-01. Canton 3.5 LSU live on 2026-06-29; four protocol upgrades [CN9][cn9] [CN10][cn10] | Mainnet announced | Usage of individual apps, SLOs, or legal authorization |
| Canton Network Pilot | 2024-03-12: 22 dApps and more than 350 **simulated** transactions on TestNet. Participants listed by role in the body total 15+13+4+3+1=36 companies; the page states 45 companies [CN11][cn11] | Adoption / demonstration | Production assets or ongoing commercial trading |
| Broadridge DLR | DLR migrated to Canton in 2023; cash is off-chain, and securities ownership transfers through smart contracts (DA customer story, 2024-06-21). The 2025-09-10 announcement self-reports average daily repo processing of $280bn and monthly processing of $5.9T in August 2025. That announcement's reference to Canton concerns a market-data distribution app [CN13][cn13] [CN12][cn12] | Commercial case announced | Global Synchronizer use, on-chain cash, or a confidential CLOB |
| Tradeweb U.S. Treasury transaction | 2026-07-01: one transaction in which Franklin Templeton exchanged tokenized U.S. Treasuries for USDCx with Virtu. “Synchronized on-chain settlement.” DTCC Tokenization Services were planned for “later this year” [CN29][cn29] | Commercial case announced (single transaction) | Sustained processing capacity or the full lifecycle |
| DTCC | 2025-12-17: following an SEC No-Action Letter, a plan to mint a subset of DTC-custodied U.S. Treasuries on Canton. An MVP was planned for a “controlled production environment” in the first half of 2026 [CN27][cn27] | Adoption / demonstration / planned | MVP completion or commercial launch (Section 8) |
| Visa | 2026-03-25: joined as an SV, described as “one of 40 Super Validators” [CN28][cn28] | Adoption announced | Visa products operating on Canton |
| JSCC / Mizuho / Nomura / DA | 2026-04-20: launched a PoC for digital JGB collateral management. Connects existing systems to Canton while preserving JGBs' legal status under the book-entry transfer law and the Financial Instruments and Exchange Act; tests 24/365 and cross-border operation. Selected for the FSA's PIP in 2026-02 [J6][j6] | Adoption / demonstration (launched) | Completion, commercialization, or end date (not stated in primary sources) |
| MUFG / DA / Progmat / Secured Finance | 2026-08-13: began collaborating on a PoC to bring JGB repo on-chain. “While preserving JGBs' legal nature as book-entry government bonds, account management institutions update their transfer account books in coordination with the blockchain”; “considering tokenized deposits and stablecoins as digital money.” Part of a demonstration supported under FSA PIP (2026-02) [J3][j3] | Adoption / demonstration (launched) | Completion, commercialization, or legal delivery |
| Progmat/DCC “Tokenized Government Bonds / On-Chain Repo WG” | Launched 2026-05; examines legal, accounting, tax, operational, and technical aspects, with a report targeted for 2026-10 (according to Secured Finance's participation announcement) [J7][j7] | Investigation launched | Technical platform selection or commercialization |
| Trading apps on Canton | Cantex is a “Canton-native AMM” that “submits swaps offchain” and settles atomically on Canton. CaviarNine provides its matching engine. A CLOB is unverified in primary sources. Hydra X announced a structured-note token on Canton (2025-04-17), with no description of a CLOB [CN30][cn30] [CN32][cn32] | Specification confirmed | An operating confidential CLOB or a matching entity that cannot read orders |

ZKFMI is currently a single-host research MVP, with smoke_only evidence for batch settlement of two fills and two continuous-trading rounds (Section 7). The maturity gap with Canton is substantial and should not be understated. Canton's cases must nevertheless be read with distinctions between “launched,” “single transaction,” “self-reported,” and “planned.”

## 3. Rows Ready to Use in the Comparison Tables

### 3.1 Table Format from Section 3 of the Initial Survey (| Target | Main competing task | Confidentiality and trust boundary | Confirmed stage and implication for comparison |)

| **Canton Network** | Ledger for institutional Daml apps, ordering and confirmation through synchronizers, holdings / transfers / allocations under the Token Standard (CIP-0056), and multi-application DvP in a single transaction on the same synchronizer | Only relevant parties and their validators read their views in plaintext; synchronizers see only encrypted envelopes and ordering metadata. A party's own validator reads all of its data. Design documents explicitly place confidential computation that hides inputs from the computing entities outside the protocol's scope. App providers performing off-ledger matching may handle plaintext | Global Synchronizer Mainnet (2024-07, LSU 2026-06), Broadridge DLR self-reported processing volumes (cash off-chain), a single Tradeweb transaction (2026-07), a DTCC MVP plan, Visa SV participation, and Japanese PoC launches by JSCC / Mizuho / Nomura (2026-04) and MUFG / Progmat (2026-08). **Highest-priority comparison and integration candidate** [CN1][cn1] [CN5][cn5] [CN7][cn7] [CN17][cn17] [CN26][cn26] [CN29][cn29] [J3][j3] [J6][j6] |

### 3.2 Table Format from Section 2 of Comparison Decisions v1.1 (| Target | Role confirmed from public materials | Key confidentiality and verification boundaries | Treatment of maturity | Treatment within ZKFMI |)

| **Canton Network** | Daml apps, validators hosting parties, synchronizers coordinating ordering and confirmation, the SV-operated Global Synchronizer (CometBFT, more than 2/3), reservation-based settlement through Token Standard allocations, and cross-app atomic transactions | Unrelated parties and synchronizers do not read payloads. Host validators and relevant parties read the corresponding views. Allocations are plaintext to the registry, app, and owner. Canton explicitly states that it chooses not to use MPC/ZK to hide inputs even from computing entities (not merely an absence of documentation) | Distinguish Global Synchronizer Mainnet, commercial DLR (self-reported), TestNet pilot (simulated), a single real transaction, and launched PoCs (U.S. DTCC and three Japanese initiatives) | **Highest-priority comparison and platform/integration candidate**: compare confidentiality boundaries (list of entities reading originals), input sets, market rules, reservation visibility, DvP, operating costs (traffic), participation conditions (sponsorship), and operational responsibility within the same workflow [Supplementary Canton research][canton] [CN17][cn17] [CN22][cn22] [CN26][cn26] |

### 3.3 Rows to Add to or Replace in the Comparison-Axis Table in Section 6 of the Supplementary Canton Research

| Comparison axis | Confirmed scope for Canton | ZKFMI's current position | Judgment |
| --- | --- | --- | --- |
| MPC that hides inputs even from computing entities (replacement) | The whitepaper (2020) explicitly rejects advanced cryptography including MPC because of computational cost and instead limits visibility under stronger trust assumptions. The 2025 blog characterizes ZKP for general-purpose privacy as experimental. In the standard path, relevant participants decrypt views and re-execute Daml [CN17][cn17] [CN18][cn18] [CN3][cn3] | Research implementation of MPC with 7 nodes and at most 2 malicious nodes. Two continuous-trading rounds are smoke_only [L16][l16] | **An axis on which ZKFMI could demonstrate a difference, grounded in Canton's design choice.** External MPC integration at the Canton app layer is a design candidate with no verified implementation. Superiority remains undetermined until compared under the same workflow, collusion, availability, and cost conditions |
| Eligibility, credit, and reservations (replacement) | Token Standard allocations standardize “lock until a deadline, then all-or-nothing settlement in one settlement-app transaction” (sender withdrawal and cancellation by sender / receiver / executor are defined in the Splice reference interface, not in the CIP body). The registry, app, and owner read allocation assets, quantities, and counterparties in plaintext [CN26][cn26] [CN34][cn34] | Separates ReservationAdmission / ReservationPermit, withholding ledger identifiers, assets, and side from matching nodes. Issuer signatures entail trust in readback [L1][l1] | Compare who reads the correspondence between reservations and orders, rather than whether a reservation feature exists. Include issuer, authoritative state, revocation, and concurrent updates |
| Operating costs, participation conditions, and governance (addition) | Synchronizer traffic is paid for by burning Canton Coin. Validator participation requires sponsorship. Governance rests with Canton Foundation (formerly GSF) and SV voting [CN23][cn23] [CN20][cn20] [CN24][cn24] | Operating costs for the in-house MPC node group and DeFMI verification nodes are unmeasured. No independent operators | Costs are unmeasured for both. Before building up costs for the same workflow at G2/G3, confirm Canton's evaluation-network access conditions |
| Confidential CLOB / RFQ (addition) | Cantex is an AMM that “submits swaps offchain” and settles atomically on Canton. A CLOB is unverified in primary sources. No public implementation of a design in which the matching entity cannot read orders has been verified [CN30][cn30] | Research implementations of OCLOB (price-time priority, 5-of-7 admission ordering) and QOMM (RFQ) [L4][l4] [L10][l10] | Canton's ecosystem uses the pattern “off-chain operator-side matching + on-chain atomic settlement.” ZKFMI's potential difference concerns the matching entity's visibility, not the market mechanism itself |

## 4. Decisions on Retaining or Revising C01–C09

| ID | Decision | Additional evidence and wording changes |
| --- | --- | --- |
| C01 MPC, ZK, and proofs of fill computation do not establish uniqueness | **Retain** | No change. In Canton, relevant participants verify Daml rules by re-execution [CN3][cn3]. The existing evidence for prior work by Renegade and Prime Match remains unchanged |
| C02 The comparison axes are “which secrets are hidden from whom” and “which rules and states are bound to the same transaction” | **Retain; add evidence** | Fix Canton's “who reads what” as an entity list (Section 2.2). Since design documents explain why Canton does not adopt confidential computation, describe the difference along this axis as “different design choices,” not an “absence of documentation” [CN17][cn17] [CN18][cn18]. Also turn the possibility that the difference disappears when customers permit disclosure to their own validators and app operators into a G0 decision question |
| C03 Differences in price formation do not imply universal superiority | **Retain** | The official Cantex description checked in this review is of an AMM; do not infer the mechanisms of all Canton trading apps from this one example. An equivalent confidential CLOB is unverified. When adding Canton to the market-mechanism comparison, assess “individual applications on Canton,” rather than treating Canton itself as a market mechanism [CN30][cn30] |
| C04 ZKFMI cannot be said to be uniformly stronger in secret-sharing security | **Retain** | No change. In the Canton comparison, also note that SV `f = floor((n-1)/3)` is ordering fault tolerance, not a confidentiality guarantee [CN22][cn22] |
| C05 Existing platforms precede ZKFMI in instruction standardization, DvP, and coordination across ledgers | **Retain; add evidence** | Add Canton's Token Standard (allocations, a single transaction by the settlement executor, and deadlines) as a concrete example of an existing platform [CN26][cn26]. Limit the description of zkPI's added value to “binding the reservation-order correspondence to an instruction while keeping it hidden from matching nodes” |
| C06 ZKFMI is a research MVP with a maturity gap relative to finished institutional products | **Retain; update evidence** | Two continuous-trading rounds with smoke_only evidence (runs 005 and 006, artifact SHA-256 `dcaca99d…0300`) can be added. Independent operation, WAN, and production safety remain false [L16][l16] [L7][l7]. Add DTCC, Visa, and three Japanese PoCs on the Canton side (Section 2.6) |
| C07 In Japan, connection to existing securities workflows and the cash leg may be an adoption condition | **Retain; add evidence** | Add the JSCC / Mizuho / Nomura JGB collateral PoC and Progmat/DCC WG [J6][j6] [J7][j7]. Note that the MUFG PoC retains JGBs as book-entry government bonds and considers tokenized deposits / stablecoins for the cash leg, so its institutional premises differ from ZKFMI's cash-leg candidates (Kinexys / Fnality / Partior) [J3][j3] |
| C08 System-wide quantum resistance cannot be established as a competitive advantage | **Retain** | No change. Canton's PQC status was not verified in this review; mark it “unverified” in the comparison table, not “unsupported” |
| C09 Canton is a priority competitor and also a candidate implementation platform and integration destination | **Retain; add conditions** | Add evaluation-network access conditions (sponsorship), traffic costs, Token Standard compatibility, and the guarantee difference between zkPI verification inside Daml and off-ledger verification as prerequisites for integration-destination evaluation [CN20][cn20] [CN23][cn23] [CN26][cn26] |

Do not add a new decision ID. The content that could form C10 (“compare reservation visibility, not the existence of reservations”) is adequately covered by adding evidence to C02 and C05.

## 5. Specific Strategy Changes

### 5.1 S01–S06

| ID | Summary of current v1.1 | Change | Evidence |
| --- | --- | --- | --- |
| S01 | Focus the initial use case on institutional secondary trading of digital securities where “orders must not be disclosed to the operator until their order is fixed” | **Retain; refine the definition.** Redefine “operator” as “computing entities, including the organization's own validator, app operator, and matching engine.” With three parallel Canton PoCs in Japanese JGB repo and collateral management, do not make direct replacement of the same workflow the initial use case. Instead, focus on portions of those workflows requiring secrecy even from computing entities (confidential orders, rate quotes, and collateral allocation) | [CN5][cn5] [CN17][cn17] [J3][j3] [J6][j6] [J7][j7] |
| S02 | Make the first sales unit an integration module for confidential trading, advance reservation, and zkPI verification | **Retain; specify the adapter contract.** Explicitly list “Daml applications on Canton” as candidate receivers. Add mappings to Token Standard allocations (lock until a deadline, one transaction by the settlement executor, and sender withdraw / cancel in the Splice reference interface). Include the difference in guarantees between “verification within a Daml contract” and “acceptance of an off-ledger verifier's signature” as a contract item | [CN26][cn26] [CN34][cn34] [CN3][cn3] |
| S03 | Use the already-running in-house MPC / DeFMI path as the baseline and first complete continuous trading and consistency under failure | **Retain; update evidence status.** Two continuous-trading rounds have been achieved (smoke_only). Next are cancellation, expiry, concurrent updates, and shutdown/recovery, followed by independent operation. Removal of Aethel dependencies proceeds in parallel as foundation work, but S03's completion conditions include an “end-to-end receipt without Aethel” (the same as the existing wording in strategy Section 4) | [L16][l16] [L7][l7] |
| S04 | Prioritize Canton and Renegade for market and settlement comparisons, Arcium/Zama for confidential-computation infrastructure, and Ownera for integration design. Also treat Canton as an implementation and integration candidate | **Retain; fix the comparison unit.** Compare at the level of “Daml app + Token Standard + synchronizer + validator operation,” not a platform-level “Canton versus DeFMI L1.” Add traffic costs, validator participation conditions, Canton Foundation governance, and choice of assigned synchronizer (Global / private). State Canton's design position on confidential computation as a confirmed fact | [CN6][cn6] [CN20][cn20] [CN23][cn23] [CN17][cn17] |
| S05 | Combine a public core and verification specifications with integration and operational support | **Retain; add guidance on specifications.** Write the public verification specifications (zkPI propositions, wire, and verifier) in a form that can also be implemented as a Daml interface or off-ledger verifier on Canton. Use Canton's open governance (Foundation, Splice OSS, and CIPs) as a comparison for publication policy | [CN26][cn26] [CN24][cn24] |
| S06 | Develop PQC as migration capability at each boundary | **Retain.** Since Canton's PQC status is unverified, mark it “unverified” in the comparison table and claim neither superiority nor inferiority | — |

### 5.2 G0–G4

| Gate | Change | Evidence |
| --- | --- | --- |
| G0 Customer problem | Add three decision questions. (a) Is disclosure of orders to the organization's own validator, app operator, and matching engine acceptable? (If so, a Canton-style approach is likely to suffice.) (b) Does the organization already participate, or plan to participate, in a Canton PoC or WG? (c) Is the cash leg tokenized deposits / stablecoins (on Canton), or a DeFMI cash rail? Add the pass condition “at least one organization documents a requirement to hide inputs even from the computing entities.” Revisit S01 if no such requirement is documented | [CN5][cn5] [J3][j3] [J6][j6] |
| G1 Continuous end-to-end path | Keep the objective unchanged. Update the evidence status to “two end-to-end rounds achieved as smoke_only; next are cancellation, expiry, concurrent updates, and shutdown/recovery.” Leave recapture of the baseline file (a new snapshot) to the parent task | [L16][l16] [L7][l7] |
| G2 One integration destination | Name Canton as a candidate and fix the acceptance path. (1) Confirm evaluation-network (DevNet / TestNet) access conditions, sponsor, and costs. (2) Represent the target assets as Token Standard-compliant Daml assets and map reservations to allocations. (3) Choose the zkPI verification location (inside Daml / off-ledger + signature) and record the guarantee difference. (4) Execute DvP in one transaction on the same synchronizer. (5) Read back finalized state through the Ledger API. (6) Check allocation release on expiry, sender withdrawal, executor cancellation, and retry behavior. (7) Record the information each validator, app, and registry could read. Mocks and other companies' Canton cases do not establish integration evidence (retain the existing wording) | [CN20][cn20] [CN26][cn26] [CN6][cn6] |
| G3 Conditions for limited adoption | If choosing the Canton path, include “the validator operator reads all of the organization's data” among the recorded independent-operation conditions. Add dependencies on SVs, Canton Foundation, and traffic as operational acceptance items (costs, governance changes, and upgrades) | [CN5][cn5] [CN10][cn10] [CN23][cn23] |
| G4 Expansion decision | No change. When using Canton's self-reported figures (processing volumes, SV count, or participating organizations) in adoption decisions, include source, date, and the label “self-reported” | [CN12][cn12] [CN28][cn28] |

### 5.3 Incorporating the Aethel Policy

- **Position in the comparison:** Canton has protocol / synchronizer / validator / application layers, with business applications (Broadridge DLR, Tradeweb, Cantex, etc.) built on the foundation. By likewise separating the “foundation (DeFMI / OCLOB / QOMM / zkPI / DeKYX / DeCCP)” from “business applications (Aethel, etc.),” ZKFMI can be compared with Canton at the same layer in the table's “role” column. Aethel is excluded from the 18-target comparison (it is on the user side, not a competitor).
- **Implementation status:** At 12:07 UTC, uncommitted changes existed in defmi (deletion of `rust/qomm-avalanche-vm/src/execution/aethel*.rs`), dekyx (deletion of `crates/dekyx-aethel`), and deccp (deletion of `crates/deccp-aethel`). The defmi README's “Depends on: aethel” remained as at the baseline time. **Do not treat the separation as complete.** Completion evidence requires checking dependencies, startup, and communication paths across all target repositories and obtaining an end-to-end receipt without Aethel (the existing condition in strategy Section 4).
- **Documentation caution:** The integrated comparison document may state “ZKFMI does not depend on Aethel” only after the above receipt is obtained. Until then, write “policy to remove the dependency.”

## 6. Corrections to the Preceding Research (With Evidence URLs)

There is no fatal error. The following refine names, sources, and counts, and add missing evidence.

| # | Location | Current wording | Correction | Evidence |
| --- | --- | --- | --- | --- |
| 1 | Supplementary Canton research Sections 1 and 9 (CN8); comparison decisions Section 7 | “Global Synchronizer Foundation” | “Canton Foundation (formerly Global Synchronizer Foundation, renamed on 2025-09-22; name change only).” The multi-synchronizer page in the official Canton docs also says “under the governance of the Canton Foundation.” The MUFG announcement likewise says “operated by Canton Foundation” | https://canton.foundation/the-gsf-is-now-canton-foundation/ , https://docs.canton.network/overview/learn/multi-synchronizer , https://www.mufg.jp/dam/pressrelease/2026/pdf/news-20260813-002_ja.pdf |
| 2 | Supplementary Canton research Section 1 (Canton Network row) | Attributes “network of networks” to CN1 (Architecture Overview) | The retrieved CN1 body did not contain this phrase. Change the source to the Canton technical primer (2025-04-01) or the Validators section of the Splice docs | https://www.canton.network/blog/a-technical-primer , https://docs.sync.global/overview/overview.html |
| 3 | Supplementary Canton research Section 5 (Pilot row) | “45 participating companies in total” | The role-based breakdown in the announcement body totals 15+13+4+3+1=36 companies. “45” is in the page summary. State both: “45 participants (page summary); role-based total in the body: 36” | https://www.canton.network/canton-network-press-releases/the-canton-network-completes-the-most-comprehensive-blockchain-pilot-to-date-for-tokenized-real-world-assets |
| 4 | Supplementary Canton research Section 5 (Broadridge row) | Uses the 2025-09-10 announcement as evidence for processing volumes | Correct. However, note that this announcement's Canton reference is to “a market-data distribution app on Canton”; the source for DLR's migration to Canton (2023) and its cash off-chain architecture is the DA customer story (2024-06-21). $280bn is self-reported “average daily repo trading in August 2025,” with $5.9T monthly | https://www.broadridge.com/press-release/2025/billions-in-average-daily-processed-trade-volumes-on-broadridge-dlt-repo-platform , https://blog.digitalasset.com/blog/customer-story-broadridge |
| 5 | Supplementary Canton research Section 9 (CN14) | investors.tradeweb.com URL | Retrieval failed with timeout and 403 in this review. Add Canton's official republication URL for the same text. The content (Franklin Templeton→Virtu, tokenized UST versus USDCx, synchronized on-chain settlement, 2026-07-01) was confirmed from the republication | https://www.canton.network/canton-network-press-releases/tradeweb-on-chain-us-treasuries-canton |
| 6 | Supplementary Canton research Sections 2 and 6 (MPC row) | Infers the difference from plaintext verification in the standard path | The Canton whitepaper (2020-02-04) and official Canton blog (2025-05-12) explicitly state the design reasons for not adopting MPC/ZK. Cite this as a “design choice,” not an “absence of documentation” | https://www.canton.io/publications/canton-whitepaper.pdf , https://www.canton.network/blog/zero-knowledge-proofs-whe-privacy-needs-more |
| 7 | Supplementary Canton research Sections 3, 4, and 6; comparison decision C05 | Describes reservations and DvP as “implementable” in Daml contracts | The Token Standard (CIP-0056, approved 2025-03-31) has already standardized reservation-based settlement through allocations. Write “already standardized, but in plaintext,” not “implementable” | https://raw.githubusercontent.com/canton-foundation/cips/main/cip-0056/cip-0056.md |
| 8 | Supplementary Canton research Section 5; customer hypothesis in strategy Section 2 | Japanese cases cover only MUFG / Progmat | Add the JSCC / Mizuho / Nomura / DA JGB collateral PoC (2026-04-20, selected for FSA PIP), the Progmat/DCC WG (launched 2026-05, report targeted for 2026-10), and SBI Digital Asset Holdings' inclusion among the 2024-07-01 go-live participants | https://blog.digitalasset.com/press-release/launch-of-proof-of-concept-trial-for-digital-collateral-management-using-japanese-government-bonds-jgbs , https://prtimes.jp/main/html/rd/p/000000006.000172212.html , https://www.canton.network/canton-network-press-releases/the-canton-networks-global-synchronizer-and-canton-coin-go-live |
| 9 | Supplementary Canton research Section 1 (Global Synchronizer row) | “2/3-majority BFT” | Correct. Add that SVs run CometBFT orderers, block production requires “agreement from more than 2/3 of SV nodes,” and fault tolerance is `f = floor((n-1)/3)`. Add traffic payment through Canton Coin burning and the sponsorship requirement for validator participation as operating conditions | https://docs.canton.network/overview/reference/super-validator-components , https://docs.canton.network/global-synchronizer/understand/overview , https://www.canton.network/faq |
| 10 | Comparison decisions Section 4 (continuous trading row); strategy Section 5 (OCLOB's current position) | “Not achieved at the baseline time. rough-001 was rejected” | Correct as a description of the baseline time (10:55:32 UTC). Subsequent runs 005 (11:18:37Z) and 006 (11:29:09Z) recorded completion of two end-to-end rounds as smoke_only. During integration, add a separate row as “additional evidence after the baseline time” and capture a new snapshot without altering the baseline file | Section 7 (local) |
| 11 | Baseline file | OCLOB HEAD `ac93685…`, README and ledger SHA-256 | At 12:07 UTC, OCLOB HEAD was `f8010fe16269b0a1821b27247ee43304fff8a89b`. README SHA-256 had changed to `31e7c6f7…dac34`, and ledger SHA-256 to `7a6f6586…28c7`. The baseline file itself is correct (a record of that time). A new baseline is needed | Section 7 (local) |

## 7. Updated Local Implementation Evidence (After the Baseline Time)

| Item | Record at the baseline time (10:55:32 UTC) | Check at 12:07 UTC | Treatment |
| --- | --- | --- | --- |
| Continuous-trading contract `oclob-native-cycle-v1` (SHA-256 `b79ab992…d9a` unchanged) | Latest record: rough-001 / rejected | Ledger line 28, repair-005 (11:18:37Z), and line 29, final-006 (11:29:09Z), are `smoke_only`. Two completed matching rounds, 3 fills, 8 receivable-right imports, guarantee-capacity update numbers 5/5 for both corporations, 14+7 node confirmations, and matching roots and restart across 5 verification nodes [L7][l7] | **Remains smoke evidence.** Single host, 2 corporations, trust in the configured DeFMI read service, no UI / WAN / independent operators, and no economic-performance evidence (as stated in the ledger's limitations field) |
| Artifact | — | `oclob/artifacts/oclob_native_cycle.json` SHA-256 `dcaca99d40e63bae9dc3dfa79163be8fbd5914a0b787a27c055bcd2257420300`, `independent_operators=false`, `completed_native_rounds=2`, second transaction `2sKQweqnWRjuGqZEdEPdUiH7Mp9sfVACH8PD1d7WFt1VzdXry7` [L16][l16] | Can be added as evidence for comparison decision C06 and strategy G1. Do not treat as evidence of production acceptance or independent operation |
| Batch settlement of multiple fills | `oclob_native_multifill.json` SHA-256 `ed871ddd…b970`, smoke_only | Same hash reconfirmed [L6][l6] | Unchanged |
| Aethel dependency | Policy only | Uncommitted `*-aethel` deletions in defmi / dekyx / deccp | In progress. Do not treat as complete |

This review did not rerun these paths. It read artifacts, ledgers, and docs; it did not consult logs from the execution environment (Softbank).

## 8. Unverifiable and Unverified Items

- **SV count:** Canton's “45+ Super Validators” (2026-04-04) was found only in secondary republications and refers to a different date from Visa's “40” announcement (2026-03-25). The exact current count is unverified. [CN28][cn28]
- **Relationship between Canton Foundation and Linux Foundation:** Secondary information suggests the relationship changed after the renaming (2025-09-22), but this is unverified in primary sources. [CN24][cn24]
- **DTCC MVP:** Planned for a “controlled production environment” in the first half of 2026. Completion and commercial launch are unverified. Tradeweb's 2026-07-01 announcement describes it as planned for “later this year.” [CN27][cn27] [CN29][cn29]
- **Duration of the JSCC / Mizuho / Nomura PoC:** Primary sources do not state an end date (the reported “around the end of September” is unverified). [J6][j6]
- **FSA PIP support decision in February 2026:** Stated in the MUFG and DA announcements, but the corresponding FSA page could not be identified. The FSA page dated 2026-02-27 announcing support under the “FinTech Proof-of-Concept Hub” (https://www.fsa.go.jp/news/r7/sonota/20260227-2/20260227-2.html ) was opened, but lists only one Hitachi case and is not the relevant page. Only the establishment of PIP (2025-11-07) was confirmed on an FSA page. [J8][j8]
- **Confidential CLOB on Canton:** A Cantex CLOB, a Hydra X CLOB on Canton, and public implementations in which the matching entity cannot read orders are unverified. Do not say they “do not exist.” [CN30][cn30] [CN32][cn32]
- **MPC/ZK integration at the Canton app layer:** Public implementations are unverified. There is also no guarantee that such capabilities will not be added to the protocol in the future.
- **Canton's PQC status:** Unverified.
- **Broadridge DLR architecture:** The synchronizer used, connection to the Global Synchronizer, and presence of on-chain cash are unverified. [CN13][cn13]
- **Performance and cost:** Canton and ZKFMI have not been measured under the same conditions for latency, processing capacity, costs, or recovery time. Canton Coin traffic charges are also unverified. **Claim neither superiority nor inferiority.**
- **Legal delivery:** The Japanese PoCs state an objective of preserving JGBs' legal status under the book-entry transfer law, but results have not been published. ZKFMI's legal finality has not been accepted either.
- **Canton pilot PDF:** As in the preceding research, it was not read in full (the announcement was used instead).
- **Canton's self-descriptions, such as “the only public chain with privacy”:** Not verified. Do not use as a basis for comparison.
- **Customer demand:** Whether disclosure to the organization's own validator is sufficient, or whether inputs must remain hidden even from computing entities, remains unverified until checked at G0.

## 9. Primary Sources

Existing IDs (CN1–CN14 and J3) use the same URLs as the [supplementary Canton research][canton]. CN15–CN34 and J6–J8 were added in this review. All were checked on 2026-09-05 (times in Section 1).

| ID | Primary source | Main points checked | Retrieval method |
| --- | --- | --- | --- |
| CN1 | [Canton Docs: Architecture Overview][cn1] | Participants store only contracts in which their parties are stakeholders. Sequencer / mediator roles. The phrase “network of networks” is absent from the body | Web retrieval |
| CN2 | [Canton Docs: Privacy Model][cn2] | View decomposition; unrelated parties see neither payload nor metadata; divulgence; Timing Attacks (time, size, patterns). No description of ZK/MPC/FHE | Web retrieval |
| CN3 | [Canton Docs: Smart Contract Consensus][cn3] | “Proof of Stakeholder”; re-execution, authorization, and ACS checks; mediator does not independently validate Daml logic; confirmation policy | Web retrieval |
| CN4 | [Canton Docs: Transaction Lifecycle][cn4] | Session-key encryption per view; sequencer retains encrypted views for a limited period but cannot decrypt them | Web retrieval |
| CN5 | [Canton Docs: Trust Model][cn5] | “Your validator sees all your data and could block you”; external party keys; trust in counterparties, app providers, synchronizers, and governance | Web retrieval |
| CN6 | [Canton Docs: Multi-Synchronizer][cn6] | One transaction uses one synchronizer; reassignment is non-atomic; Global Synchronizer is under Canton Foundation governance | Web retrieval |
| CN7 | [Canton Docs: Cross-Synchronizer DvP Example][cn7] | Exchange in one transaction after reassignment to a common Settlement Sync; pending-state boundary | Web retrieval |
| CN8 | [Splice Docs: Global Synchronizer Overview][cn8] | 2/3 BFT; SV roles; Canton Coin; GSF; “network of networks” | Web retrieval |
| CN9 | [Global Synchronizer / Canton Coin go-live, 2024-07-01][cn9] | Go-live; 31 participating organizations (including SBI Digital Asset Holdings); Linux Foundation support for GSF | Web retrieval |
| CN10 | [Logical Synchronizer Upgrades, 2026-06-29][cn10] | Canton 3.5; LSU live on Mainnet; four protocol upgrades | Web retrieval |
| CN11 | [Canton Network Pilot completion, 2024-03-12][cn11] | TestNet; 22 dApps; more than 350 simulated transactions; 36 companies by role (45 in the summary) | Web retrieval |
| CN12 | [Broadridge, 2025-09-10][cn12] | Average daily $280bn and monthly $5.9T in August 2025 (self-reported); market-data distribution app on Canton | Web retrieval |
| CN13 | [Digital Asset: Broadridge customer story, 2024-06-21][cn13] | Migration to Canton in 2023; cash off-chain; securities ownership transferred by smart contracts | Web retrieval |
| CN14 | [Tradeweb (investors.tradeweb.com), 2026-07-01][cn14] | Timed out in this review. Replaced by CN29 | Unavailable |
| CN15 | [Daml Ledger Model: Privacy][cn15] | “Need-to-know basis”; “at the subtransaction level”; definitions of informee / witness / projection / divulgence | Body extracted with curl |
| CN16 | [Canton Architecture: Overview and Assumptions][cn16] | Sequencer's “only two functions”; mediator learns no contents and receives informee lists; participants' identities hidden from one another | Body extracted with curl |
| CN17 | [Canton whitepaper, 2020-02-04][cn17] | Design policy: advanced cryptography including MPC is computationally expensive, so visibility is limited under stronger trust assumptions | Local PDF extraction |
| CN18 | [Canton blog: When Privacy Needs Proof, 2025-05-12][cn18] | Position that ZKP for general-purpose privacy is experimental, while Canton provides auditable privacy | Web retrieval |
| CN19 | [Canton blog: Institutional-Grade Privacy, 2025-08-14][cn19] | Data separation and encryption; operators see only limited metadata. No description of ZK/MPC/FHE/TEE | Web retrieval |
| CN20 | [Canton FAQ][cn20] | Sub-transaction privacy; operators see limited metadata; sponsorship for validator applications; atomic composition | Web retrieval |
| CN21 | [Canton technical primer, 2025-04-01][cn21] | “Network of networks”; validators hold only relevant data; synchronizer likened to a post office that cannot open sealed letters | Web retrieval |
| CN22 | [Canton Docs: Super Validator Components][cn22] | Sequencer, mediator, CometBFT orderer, scan, governance app; more than 2/3; `f = floor((n-1)/3)` | Web retrieval |
| CN23 | [Canton Docs: Global Synchronizer Overview][cn23] | Traffic payments by burning Canton Coin; Canton Foundation operates an SV node | Web retrieval |
| CN24 | [Canton Foundation: Renaming from GSF, 2025-09-22][cn24] | Name change only; Global Synchronizer governance | Web retrieval |
| CN25 | [Linux Foundation, 2025-03-19][cn25] | More than 30 GSF participants; Goldman Sachs, HKFMI, and Moody's join; LF support | Web retrieval |
| CN26 | [CIP-0056 Canton Network Token Standard][cn26] | Final; approved 2025-03-31. Allocation locks and deadlines; all-or-nothing settlement in one settlement-app transaction. No definitions of withdraw / cancel / revoke in the body (full text rechecked with curl during revision) | Web retrieval (raw) + curl |
| CN27 | [DTCC / Digital Asset, 2025-12-17][cn27] | SEC No-Action Letter; minting a subset of DTC-custodied U.S. Treasuries on Canton; MVP planned for the first half of 2026 | Web retrieval |
| CN28 | [Visa, 2026-03-25][cn28] | Joined as an SV; “one of 40 Super Validators” | Web retrieval |
| CN29 | [Canton's official republication of the Tradeweb announcement, 2026-07-01][cn29] | Franklin Templeton→Virtu; tokenized UST versus USDCx; synchronized on-chain settlement; DTCC Tokenization Services planned | Web retrieval |
| CN30 | [Cantex official site][cn30] | “Canton-native AMM”; “submits swaps offchain”; matching engine provided by CaviarNine; atomic settlement on Canton. Only the docs.cantex.io home page was opened; “How Cantex Works” was not opened | Web retrieval |
| CN31 | [Canton Foundation: About][cn31] | Global Synchronizer development and governance; SV operation; Digital Asset is a Premier Member | Web retrieval |
| CN32 | [Hydra X, 2025-04-17][cn32] | Structured-note token on Canton. No description of a CLOB | Web retrieval |
| CN33 | [Canton Docs: Synchronizer][cn33] | Only encrypted envelopes and metadata; cannot decrypt payloads; BFT by SVs | Web retrieval |
| CN34 | [Splice: `Splice.Api.Token.AllocationV1` Daml interface][cn34] | `Allocation_Withdraw` (controller: sender; SHOULD not cause settlement to fail before `allocateBefore`), `Allocation_Cancel` (joint control by sender, receiver, and executor, normally delegated to the executor), `Allocation_ExecuteTransfer` (before `settleBefore`). Retrieved from the main branch; main HEAD at that time was commit `fe492f45a8ba7073120c081b583fa13bd4254be6` (2026-09-04T19:28:43Z) | curl (during revision at 12:30 UTC) |
| J3 | [MUFG: JGB repo demonstration launch, 2026-08-13][j3] | 4 companies + DA + Progmat + Secured Finance; preservation of the legal nature of book-entry government bonds; tokenized deposits / stablecoins; FSA PIP; Canton Foundation | Local PDF extraction |
| J6 | [JSCC / Mizuho / Nomura / DA: JGB collateral PoC, 2026-04-20 (official DA republication)][j6] | Preserves legal status under the book-entry transfer law and the Financial Instruments and Exchange Act; connects existing systems to Canton; 24/365 and cross-border operation; selected for FSA PIP | Web retrieval (jpx.co.jp and Nomura returned 403) |
| J7 | [Secured Finance: Joining the Progmat/DCC WG, 2026-05-08][j7] | WG launch month; perspectives examined; report targeted for 2026-10 | Web retrieval |
| J8 | [FSA: Establishment of the Payment Innovation Project (PIP), 2025-11-07][j8] | “Today (November 7), we launched the Payment Innovation Project (PIP), dedicated to payments, within the FinTech Proof-of-Concept Hub.” The page for the February 2026 support decision remains unidentified | curl (opened during revision at 12:30 UTC) |
| L1 | defmi README (same as L1 in the baseline file) | Separation of ReservationAdmission / ReservationPermit | Local |
| L4 | oclob README | Corporate-side sharing; 7 nodes; originals withheld from the coordinator; limitations | Local (updated after the baseline time) |
| L6 | oclob/artifacts/oclob_native_multifill.json | SHA-256 `ed871ddd…b970`; smoke_only | Local |
| L7 | oclob/research/experiment-ledger.jsonl | Runs 005 and 006 on lines 28 and 29 | Local (updated after the baseline time) |
| L10 | qomm README | RFQ mechanism | Local |
| L13 | zkpi README | Propositions and verification scope | Local |
| L16 | oclob/artifacts/oclob_native_cycle.json | SHA-256 `dcaca99d…0300`; `completed_native_rounds=2`; `independent_operators=false` | Local (generated after the baseline time) |

[survey]: COMPETITORS_EX_CANTON_2026-09-05.md
[decisions]: COMPETITIVE_DECISIONS_2026-09-05.md
[strategy]: ../strategy/ZKFMI_STRATEGY_2026-09-05.md
[baseline]: STRATEGY_BASELINE_2026-09-05.json
[canton]: CANTON_NETWORK_2026-09-05.md
[cn1]: https://docs.canton.network/overview/learn/architecture
[cn2]: https://docs.canton.network/overview/learn/privacy-model
[cn3]: https://docs.canton.network/overview/reference/smart-contract-consensus
[cn4]: https://docs.canton.network/overview/reference/transaction-lifecycle
[cn5]: https://docs.canton.network/overview/learn/trust-model
[cn6]: https://docs.canton.network/overview/learn/multi-synchronizer
[cn7]: https://docs.canton.network/overview/reference/cross-sync-dvp-example
[cn8]: https://docs.sync.global/overview/overview.html
[cn9]: https://www.canton.network/canton-network-press-releases/the-canton-networks-global-synchronizer-and-canton-coin-go-live
[cn10]: https://www.canton.network/blog/logical-synchronizer-upgrades
[cn11]: https://www.canton.network/canton-network-press-releases/the-canton-network-completes-the-most-comprehensive-blockchain-pilot-to-date-for-tokenized-real-world-assets
[cn12]: https://www.broadridge.com/press-release/2025/billions-in-average-daily-processed-trade-volumes-on-broadridge-dlt-repo-platform
[cn13]: https://blog.digitalasset.com/blog/customer-story-broadridge
[cn14]: https://investors.tradeweb.com/news-releases/news-release-details/tradeweb-facilitates-landmark-chain-us-treasuries-transaction
[cn15]: https://docs.daml.com/concepts/ledger-model/ledger-privacy.html
[cn16]: https://docs.daml.com/canton/architecture/overview.html
[cn17]: https://www.canton.io/publications/canton-whitepaper.pdf
[cn18]: https://www.canton.network/blog/zero-knowledge-proofs-whe-privacy-needs-more
[cn19]: https://www.canton.network/blog/how-canton-network-delivers-institutional-grade-privacy
[cn20]: https://www.canton.network/faq
[cn21]: https://www.canton.network/blog/a-technical-primer
[cn22]: https://docs.canton.network/overview/reference/super-validator-components
[cn23]: https://docs.canton.network/global-synchronizer/understand/overview
[cn24]: https://canton.foundation/the-gsf-is-now-canton-foundation/
[cn25]: https://www.linuxfoundation.org/press/largest-financial-institutions-join-the-global-synchronizer-foundation
[cn26]: https://raw.githubusercontent.com/canton-foundation/cips/main/cip-0056/cip-0056.md
[cn27]: https://www.canton.network/canton-network-press-releases/dtcc-and-digital-asset-partner-to-tokenize-dtc-custodied-u.s.-treasury-securities-on-the-canton-network
[cn28]: https://usa.visa.com/about-visa/newsroom/press-releases.releaseId.22231.html
[cn29]: https://www.canton.network/canton-network-press-releases/tradeweb-on-chain-us-treasuries-canton
[cn30]: https://www.cantex.io/
[cn31]: https://canton.foundation/about-the-foundation/
[cn32]: https://www.hydrax.io/blog/hydra-x-partners-with-canton-network-to-launch-the-first-structured-product-on-the-network-the-sigma-value-token/
[cn33]: https://docs.canton.network/overview/reference/synchronizer-overview
[cn34]: https://raw.githubusercontent.com/canton-network/splice/main/token-standard/splice-api-token-allocation-v1/daml/Splice/Api/Token/AllocationV1.daml
[j3]: https://www.mufg.jp/dam/pressrelease/2026/pdf/news-20260813-002_ja.pdf
[j6]: https://blog.digitalasset.com/press-release/launch-of-proof-of-concept-trial-for-digital-collateral-management-using-japanese-government-bonds-jgbs
[j7]: https://prtimes.jp/main/html/rd/p/000000006.000172212.html
[j8]: https://www.fsa.go.jp/news/r7/sonota/20251107-2/01.html
[l1]: /Users/shukob/Research/DeFMI/defmi/README.md
[l4]: /Users/shukob/Research/DeFMI/oclob/README.md
[l6]: /Users/shukob/Research/DeFMI/oclob/artifacts/oclob_native_multifill.json
[l7]: /Users/shukob/Research/DeFMI/oclob/research/experiment-ledger.jsonl
[l10]: /Users/shukob/Research/DeFMI/qomm/README.md
[l13]: /Users/shukob/Research/DeFMI/zkpi/README.md
[l16]: /Users/shukob/Research/DeFMI/oclob/artifacts/oclob_native_cycle.json
