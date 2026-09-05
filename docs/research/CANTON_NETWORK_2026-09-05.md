# Supplementary Canton Network Research v1.1

Review date: **2026-09-05**. This document adds Canton Network as the 18th comparison target while preserving the [initial survey excluding Canton][survey] as a historical source. It checked specifications and operational materials from Canton / Digital Asset / Canton Foundation (formerly Global Synchronizer Foundation), along with adopters' own announcements. Fourteen new primary-source URLs were added to the 40 referenced by the existing comparison decisions. The first edition's evidence set contained 54 distinct URLs. v1.1 integrates the [Claude Fable 5.1 Max review][fable], distinguishing retrieval status from supplementary materials. Because MUFG source J3 from the existing survey was also reused as a Canton case, the first edition had 15 Canton references. Supplementary materials are identified separately at the end and in the Fable review.

**Conclusion:** Canton is ZKFMI's most important comparison target: it already provides selective visibility, transaction verification by the parties, atomic state updates across multiple applications and participants, and operating institutional applications. Consequently, the claim that “only we can prove market rules or DvP” does not establish uniqueness. The publicly documented core Canton protocol, however, has the validators involved in a transaction decrypt plaintext views and re-execute Daml. **MPC that hides complete inputs even from each node performing the computation** provides a different guarantee, on which ZKFMI could demonstrate a difference. From the boundary between Daml business logic and off-ledger integration, we infer that a Canton application could be designed to combine MPC, ZK, eligibility, credit, reservations, and market rules, but we have not verified an implementation of the same application. Therefore, treat neither “Canton does not support this” nor “equivalent functionality is already operating” as established. [CN1][cn1] [CN3][cn3] [CN4][cn4] [CN5][cn5]

## 1. Do Not Treat “Canton” as a Single Product

| Layer / role | What official materials confirm | Distinctions to preserve in comparisons |
| --- | --- | --- |
| **Canton Network** | A “network of networks” connecting independently operated applications, validators, and multiple synchronizers. Each validator stores only the ledger fragments of the parties it hosts [CN1][cn1] [CN8][cn8] | One company's Canton project is not evidence that all network capabilities are operating |
| **Daml applications** | Daml contracts define business logic, signatory / observer / controller roles, authorization, and disclosure. Smart contracts execute on validators [CN1][cn1] [CN2][cn2] | Separate properties of the Canton foundation from order, credit, reservation, and clearing rules implemented by individual applications |
| **validator / participant node** | Official docs define a validator as a Network role that operates a participant node. It hosts parties, stores their relevant contracts, and decrypts and validates received transaction views [CN1][cn1] [CN3][cn3] | The term “validator” does not imply a conventional public chain in which all validators read all transactions |
| **synchronizer** | The sequencer orders and delivers encrypted messages; the mediator aggregates confirmations from relevant validators and declares commit / reject. It does not itself validate Daml contents [CN1][cn1] [CN3][cn3] | Separate ordering and confirmation coordination from validation of business logic and input correctness |
| **Global Synchronizer** | A public synchronizer operated in a distributed manner by Super Validators. Performs message ordering and confirmation using 2/3-majority BFT. Apps can also choose private or multiple synchronizers [CN6][cn6] [CN8][cn8] | Canton apps do not necessarily always use the Global Synchronizer or perform all processing there |
| **application provider** | Provides on-ledger Daml and off-ledger business logic and authentication, and may submit users' transactions [CN5][cn5] | Separate protocol verification from trust in a provider to implement off-ledger input selection, authentication, and submission correctly |

## 2. Who Can Read What

Canton decomposes transactions into hierarchical views and encrypts each view for delivery to the participants of the corresponding informees / witnesses. Unrelated parties do not receive the view, and synchronizers do not decrypt encrypted payloads. The parties specified by the application, such as signatories, observers, and controllers, read their own views and validate the results. [CN2][cn2] [CN4][cn4]

However, secrets are not hidden from every operating or computing entity.

- **Your own validator:** Stores contract data for the parties it hosts; the official trust model explicitly states that it can read all data of its own parties. When using a validator operated by a third party, you trust that operator not to leak data and to participate correctly in consensus. External party keys can avoid signing by the validator, but are not described as eliminating its visibility into the data. [CN5][cn5]
- **Transaction parties:** Parties and their validators decrypt views according to their rights. Trust that a counterparty will not leak received secrets is not removed by cryptography alone. [CN5][cn5]
- **Synchronizer:** The sequencer cannot read encrypted contents but performs recipient routing; the mediator knows each view's informees, approve / reject responses, and confirmation policy. The public privacy guide also identifies inferences from timing, message size, and activity patterns as design concerns. [CN2][cn2] [CN3][cn3]
- **Application provider:** When it performs off-ledger order admission, input selection, authentication, and external API integration, its implementation and freedom from censorship are trusted. [CN5][cn5]

Canton's strength is therefore **sub-transaction privacy that shares plaintext only among the necessary parties**. This differs in comparison scope from the guarantee targeted by ZKFMI's MPC: “complete order inputs are not provided even to individual computing nodes within the tolerated collusion threshold.” As a design inference, an off-ledger application on Canton could run external MPC and submit only its result to a Daml transaction, or an application could be built to integrate MPC. This is not a verified implementation. Treat it as a difference between the publicly documented standard path and ZKFMI's current path, rather than a missing capability of Canton as a whole. [CN1][cn1]

The 2020 whitepaper explains the cost of advanced cryptography and the choice to make trust explicit while restricting data sharing. The difference between this standard path and MPC has a design basis, but does not establish that Canton applications cannot integrate external MPC or ZK. [CN17][cn17]

## 3. What Is and Is Not Verified

The submitting participant interprets commands through the Daml engine and constructs a transaction tree and root hash. For views addressed to them, the relevant participants check Daml re-execution results, signatory / controller authorization, required signatures, and validity of input contracts against their own Active Contract Sets, then return signed approve / reject responses to the mediator. The mediator tallies whether the confirmation policy is satisfied but does not independently revalidate Daml logic. [CN3][cn3] [CN4][cn4]

This strongly addresses **correctness and authorization of submitted Daml transactions, and prevention of double use of the relevant contracts**. Because Daml handles business logic, authorization, and multi-party agreement, we infer that a design could implement order contracts, eligibility, credit limits, asset reservations, and price-time-priority or RFQ rules, binding them to the same transaction. [CN1][cn1] An implementation or performance of the same market application has not been verified, but ZKFMI cannot claim to be the only system capable of implementing eligibility, reservations, and market rules.

The following propositions, however, do not follow automatically from the core protocol alone; individual applications and admission boundaries must be checked.

- Whether every order arriving off-ledger was converted into a contract and submitted without omission.
- Whether admission ordering, censorship, or latency manipulation occurred before submission to the synchronizer.
- Whether the order set referenced by a matching transaction matches the set required by law or market rules.
- Whether external KYC/KYB, credit master records, custody accounts, and cash legs match the Daml contract state.
- Whether an auditor unrelated to the transaction can verify the same proposition as a portable ZK proof without receiving secrets.

These could be added through Daml contracts, signatures, observers, external proofs, MPC, and admission commitments. Do not label an item “unsupported by Canton” because public materials do not describe an implementation. ZKFMI can claim a difference only after specifying the functions, input sets, and authoritative-state readback bound by zkPI, along with its limits concerning pre-admission censorship.

## 4. Boundaries of Atomicity and DvP

Contracts assigned to the same synchronizer can be updated across multiple applications and participants in one Daml transaction, with all updates committing or all aborting. The Global Synchronizer supports this arrangement as a synchronizer to which validators of independent applications can connect in common. [CN6][cn6] [CN7][cn7]

In the official DvP example using cash and securities contracts on different synchronizers, both are reassigned to a common settlement synchronizer and then exchanged in one Daml transaction. **The settlement step is atomic**, but unassignment and assignment are separate transactions; between them, contracts are pending and unavailable for use. Failure may leave them pending until resolved. [CN7][cn7]

Accordingly, compare Canton's atomicity within the following limits.

| Scope | Confirmed guarantee | Unverified generalization |
| --- | --- | --- |
| Daml contracts on the same synchronizer | All updates or no updates within one transaction | Automatic atomicity extending to off-ledger bank accounts or existing CSD updates |
| Multiple Canton applications and participants | Atomic composition if relevant validators connect to the same synchronizer and the required contracts are handled in the same transaction | Arbitrary existing applications are already connected, authorized, and accepted for legal delivery with one another |
| Multiple synchronizers | Atomic execution of settlement after reassignment to a common synchronizer | The entire reassignment completes in one transaction, or assets remain usable while pending |
| Ledgers outside Canton | Designs can map semantics into Canton contracts through adapters or tokenization | Canton protocol alone guarantees hold / commit / abort / finality on external ledgers |

This is an important counterargument for ZKFMI. Batch settlement of multiple fills within DeFMI alone does not establish uniqueness relative to Canton. To demonstrate a difference, specify the propositions linking MPC inputs, admission sets, eligibility, credit, reservations, market rules, zkPI, and settlement, and identify the trust boundaries that still contain external legs.

### Reservation-Based Settlement in the Token Standard

CIP-0056 standardizes asset reservations through allocations and settlement of all legs in one Daml transaction submitted by the settlement app. Reservation and DvP are not merely hypothetical implementation possibilities. [CN26][cn26]

The Splice reference interface distinguishes sender withdrawal from cancellation jointly authorized by sender, receiver, and executor. For withdrawal before `allocateBefore`, when reallocation can still occur in time, it states a SHOULD that settlement not be caused to fail. The execution deadline is `settleBefore`. Specific processing is left to each registry implementation, so do not generalize that withdrawal is unconditionally possible before the deadline. [CN34][cn34]

## 5. Commercial Cases and Network Stages

| Case | Stage confirmed in primary sources | Evidence used in this comparison | Do not generalize uniformly to |
| --- | --- | --- | --- |
| Global Synchronizer | Go-live announced on 2024-07-01. On 2026-06-29, Canton 3.5 Logical Synchronizer Upgrade was announced as live on Mainnet [CN9][cn9] [CN10][cn10] | Public distributed synchronizer has reached operation and upgrade stages, beyond planning | Usage, SLOs, legal authorization, or comprehensive cross-app DvP operation of individual Canton apps |
| Canton Network Pilot | Official announcement reports more than 350 **simulated transactions** across 22 independent dApps on TestNet [CN11][cn11] | Large-scale pilot of atomic composition across multiple applications | Production assets, ongoing commercial trading, or production adoption by every participant (45 in the summary, 36 in the role-based body total) |
| Broadridge DLR | Broadridge announced average daily repo transaction processing of $280bn in August 2025. Digital Asset's customer story dated 2024-06-21 describes the 2023 migration to Canton and the then-current arrangement of transferring securities ownership while keeping cash off-chain [CN12][cn12] [CN13][cn13] | Evidence of high commercial processing volumes in an institutional Canton/Daml application | Cross-app DvP using Global Synchronizer, on-chain cash, a confidential CLOB, or production evidence for all Canton capabilities |
| Tradeweb U.S. Treasury transaction | On 2026-07-01, Tradeweb announced completion of one real-time transaction involving tokenized U.S. Treasuries and USDCx through Canton's synchronized settlement. After the original URL timed out, Fable checked the official Canton republication [CN14][cn14] [CN29][cn29] | A concrete on-chain cash / securities transaction involving real participants and real asset categories | Sustained market-wide processing capacity, the full lifecycle, or full operation of DTCC Tokenization Services |
| MUFG / Progmat JGB repo | On 2026-08-13, announced the **start** of a demonstration collaboration using Canton. Investigates JGB / digital-money DvP and repo lifecycle automation [J3][j3] | A demand hypothesis in Japan that directly overlaps Canton's scope with ZKFMI's target workflows | Completion of the demonstration, commercialization, legal delivery, or acceptance of all capabilities |

In Japan, the JSCC / Mizuho / Nomura / DA JGB collateral PoC (announced 2026-04-20) and Progmat/DCC WG (launched 2026-05) were also confirmed. PoCs and WGs represent different testing and investigation stages; neither establishes commercialization or completed legal delivery. [J6][j6] [J7][j7]

Global Synchronizer Foundation was renamed Canton Foundation on 2025-09-22. Do not infer a governance change from the name change. [CN24][cn24] Broadridge's cash off-chain description refers to the 2024 architecture; its cash leg as of 2026 is unverified.

Broadridge's processing volumes, Tradeweb's single transaction, and MUFG's demonstration launch are different kinds of evidence. Do not count a long list of companies using Canton as evidence of an identical architecture, feature set, or maturity level.

## 6. Comparison Decisions for ZKFMI

| Comparison axis | Confirmed scope for Canton | ZKFMI's current position | Judgment |
| --- | --- | --- | --- |
| Who reads secrets | Hidden from unrelated parties and synchronizers. Host validators and entitled parties read the relevant views | In the new CLI path, sharing occurs on the corporate side, and each MPC node lacks complete inputs. Collusion by 3 or more nodes, communication observation, and similar issues are separate limitations | **An axis on which ZKFMI could demonstrate a difference.** The difference shrinks if customers permit disclosure to their own validators |
| MPC that hides inputs even from computing entities | In the verified standard transaction path, relevant participants decrypt views and re-execute Daml | Research implementation of MPC with 7 nodes and at most 2 malicious nodes | External MPC integration on Canton is a design candidate with no verified implementation. Superiority remains undetermined until compared under the same workflow, collusion, availability, and cost conditions |
| Eligibility, credit, and reservations | Daml can generally express and verify authorization, state, and business rules. Reservation-based settlement under the Token Standard is confirmed. Equivalent eligibility / credit apps are unverified; consistency with external authoritative records depends on adapters [CN26][cn26] | Aims to connect DeKYX, reservations, and credit / guarantee capacity across the end-to-end path | Feature names do not establish uniqueness. Compare issuer, authoritative state, revocation, and concurrent updates as well |
| Market rules | Confirmed general architecture in which parties validate deterministic Daml logic. An identical CLOB / RFQ app is unverified | Implementing concrete QOMM / OCLOB RFQ and price-time-priority mechanisms with zkPI | Recognize the possibility of building the same CLOB / RFQ on Canton, without treating it as already implemented |
| Input set | Binds contracts and roots referenced by submitted transactions. Completeness of off-ledger admission lies at the application boundary | Aims to bind admission ordering and order sets into proofs. Pre-admission censorship remains possible | Neither automatically proves the “entire market.” Compare admission commitments and target sets |
| Settlement atomicity | Cross-app DvP in one Daml transaction on the same synchronizer. Across synchronizers, settlement is atomic after reassignment | Smoke evidence for batch settlement of multiple fills within the DeFMI native rail | Canton is a strong existing comparison. Do not claim ZKFMI superiority until verified with the same legs, including external ledgers |
| Third-party verification | Relevant validators check each view. Auditors can be observers or similar roles. This differs from a portable ZK proof for unrelated parties | Aims for independent verification of propositions defined by zkPI | Potential ZKFMI value. Not exclusive, because a zkPI verifier could be added to a Canton app |
| operational trust | Role-specific trust remains in the organization's own / outsourced validator, counterparties, application providers, synchronizers, and governance | MPC node group, coordinator, readback, DeFMI validators, and independence of keys and operators have not been accepted | Compare the entities responsible for confidentiality, correctness, availability, censorship, and recovery, rather than assigning scores by node count |
| Maturity, performance, and legal framework | Mainnet, commercial DLR, pilots, a single transaction, and launched PoCs coexist | Single-host research MVP and smoke evidence | Do not understate Canton's maturity. Performance under identical conditions, customer demand, and authorization for individual workflows are unverified |

## 7. Implications for Strategy

1. **Priority comparison:** Make Canton, alongside Renegade, a highest-priority comparison across confidentiality scope, market rules, reservations, DvP, and operational responsibility.
2. **Option as additional functionality:** Include adding ZKFMI's MPC order processing, zkPI verifier, and eligibility / reservation adapters to Daml asset / cash applications on Canton as a G2 integration candidate.
3. **Option as an integration destination:** Evaluate forwarding settlement instructions to Canton-native contracts through a Global or private synchronizer under the same acceptance conditions as existing-ledger adapters.
4. **Alternative for the full implementation:** Acknowledge that if the same market can be built with Canton + external MPC / ZK, the adoption advantage of a standalone DeFMI L1 may shrink. Do not decide on replacement now, because customer demand, performance, costs, independent operation, and legal delivery have not been measured under the same conditions.
5. **Preserve existing objectives:** Do not cancel implementation or research objectives for a standalone DeFMI L1, QOMM, OCLOB, or DeCCP. Use the current path as the baseline, complete continuous trading and recovery, and address Canton integration and alternative comparisons later at G2.
6. **Revise the claim:** Reject “only we prove the market”; propose only concrete differences in the confidentiality boundaries customers need and in binding admission, eligibility, reservations, rules, and settlement into one verification contract.

## 8. Unverified Items

- Results of implementing on Canton the same confidential CLOB / RFQ, input set, eligibility, credit, reservations, and zkPI propositions as ZKFMI.
- What each entity can read of original orders, contracts, recipient metadata, and communication patterns in real configurations, including managed validators.
- Latency, processing capacity, costs, and recovery times for ZKFMI and Canton with identical inputs, confidentiality conditions, and failure conditions. **Performance superiority is unverified.**
- Connection contracts, API access rights, costs, SLOs, and operator independence for Global Synchronizer or commercial applications.
- Required authorization and legal finality for each entity concerning Japanese JGB repo, digital securities, cash legs, trading markets, and clearing. **Legal authorization is unverified.**
- Whether customers are satisfied with disclosure to validators and counterparties or need MPC that hides inputs even from computing entities. **Customer demand is unverified.**

This research did not run Canton nodes or competing applications, conduct new benchmarks, contact customers, or perform external writes. The official pilot PDF could not be opened directly through Web retrieval because of a size limit, so the retrievable official pilot-completion announcement explicitly describing the same content [CN11][cn11] was used as evidence. The PDF is excluded from the count of materials read in full.

## 9. Primary Sources

| ID | Primary source | Main points checked |
| --- | --- | --- |
| CN1 | [Canton Network Docs: Architecture Overview][cn1] | validator / participant, synchronizer, Daml, and roles in storage, execution, and ordering |
| CN2 | [Canton Network Docs: Privacy Model Explained][cn2] | View-level disclosure, divulgence, timing / size / activity patterns, and validator visibility |
| CN3 | [Canton Network Docs: Smart Contract Consensus][cn3] | Proof of Stakeholder, re-execution, authorization, ACS, and mediator limitations |
| CN4 | [Canton Network Docs: Transaction Lifecycle][cn4] | Transaction tree, root hash, view encryption, and the path from submit to commit |
| CN5 | [Canton Network Docs: Trust Model Overview][cn5] | Trust in validators, counterparties, application providers, synchronizers, and governance |
| CN6 | [Canton Network Docs: Multi-Synchronizer Architecture][cn6] | Private / Global Synchronizer, contract assignation, and reassignment |
| CN7 | [Canton Network Docs: Cross-Synchronizer DvP Example][cn7] | Atomic settlement on a common synchronizer and pending-state boundaries |
| CN8 | [Global Synchronizer Foundation / Splice Docs][cn8] | Operational roles of Super Validators, 2/3 BFT, validators, and governance |
| CN9 | [Global Synchronizer and Canton Coin Go Live, 2024-07-01][cn9] | Go-live of public distributed infrastructure |
| CN10 | [Logical Synchronizer Upgrades, 2026-06-29][cn10] | Announcement of Canton 3.5 / LSU Mainnet operation |
| CN11 | [Canton Network Pilot completion, 2024-03-12][cn11] | TestNet, 22 dApps, and more than 350 simulated transactions |
| CN12 | [Broadridge DLR, 2025-09-10][cn12] | Announcement of average daily processing of $280bn in August 2025 |
| CN13 | [Digital Asset: Broadridge customer story][cn13] | DLR workflow boundaries, including Canton migration, Daml, and off-chain cash |
| CN14 | [Tradeweb, 2026-07-01][cn14] | Single real-time transaction involving tokenized U.S. Treasuries / USDCx |
| J3 | [MUFG: JGB repo demonstration launch, 2026-08-13][j3] | Canton use, DvP / lifecycle demonstration plans, and stage |

[survey]: COMPETITORS_EX_CANTON_2026-09-05.md
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
[j3]: https://www.mufg.jp/dam/pressrelease/2026/pdf/news-20260813-002_ja.pdf

[fable]: CANTON_FABLE_5_1_MAX_REVIEW_2026-09-05.md
[cn17]: https://www.canton.io/publications/canton-whitepaper.pdf
[cn24]: https://canton.foundation/the-gsf-is-now-canton-foundation/
[cn26]: https://raw.githubusercontent.com/canton-foundation/cips/main/cip-0056/cip-0056.md
[cn29]: https://www.canton.network/canton-network-press-releases/tradeweb-on-chain-us-treasuries-canton
[cn34]: https://raw.githubusercontent.com/canton-network/splice/main/token-standard/splice-api-token-allocation-v1/daml/Splice/Api/Token/AllocationV1.daml
[j6]: https://blog.digitalasset.com/press-release/launch-of-proof-of-concept-trial-for-digital-collateral-management-using-japanese-government-bonds-jgbs
[j7]: https://prtimes.jp/main/html/rd/p/000000006.000172212.html
