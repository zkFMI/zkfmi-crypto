# Canton Network 追加調査 v1.1

確認日: **2026-09-05**。本書は、[Canton以外の初回サーベイ][survey]を歴史的原文として残し、18番目の比較対象としてCanton Networkを追加する調査である。Canton / Digital Asset / Canton Foundation（旧Global Synchronizer Foundation）の仕様・運用資料と、採用者自身の発表を確認した。既存の比較判断が参照する40件に、新規の一次URL 14件を加えた。初版の根拠集合は重複を除く54URLだった。v1.1は[Claude Fable 5.1 Maxのレビュー][fable]を統合し、取得状態と追加資料を区別する。既存サーベイのMUFG資料J3もCanton事例として再利用するため、Cantonに関する初版の参照は15件だった。追加資料は末尾とFableレビューで区別する。

**結論:** Cantonは、選択的な閲覧、当事者による取引検証、複数アプリ・参加者をまたぐ原子的な状態更新、稼働する機関向けアプリを既に持つ、ZKFMIの最重要比較対象である。このため「自分たちだけが市場規則やDvPを証明できる」という独自性は成立しない。一方、公開されたCantonの中核プロトコルは、取引に関係するvalidatorが平文のviewを復号してDamlを再実行する方式である。**計算を担当する各ノードからも完全な入力を隠すMPC**は別の保証であり、ZKFMIが差を示し得る。Damlの業務ロジックとoff-ledger integrationの境界から、CantonアプリへMPC・ZK・参加資格・与信・予約・市場規則を組み合わせる構成は設計可能と推論するが、同じアプリの実装は確認していない。したがって「Cantonは非対応」とも「同等機能が稼働済み」とも扱わない。[CN1][cn1] [CN3][cn3] [CN4][cn4] [CN5][cn5]

## 1. 「Canton」を一つの製品として扱わない

| 層・役割 | 公式資料から確認したこと | 比較で混同しないこと |
| --- | --- | --- |
| **Canton Network** | 独立して運営されるアプリ、validator、複数のsynchronizerを結ぶ「network of networks」。各validatorは自分がhostするpartyの台帳断片だけを保存する [CN1][cn1] [CN8][cn8] | Cantonを使う一社の案件を、全ネットワーク機能の稼働証拠にしない |
| **Damlアプリ** | Daml contractが業務ロジック、signatory / observer / controller、権限と開示範囲を定める。smart contractはvalidator上で実行される [CN1][cn1] [CN2][cn2] | Canton基盤の性質と、個別アプリが実装した注文、与信、予約、清算規則を分ける |
| **validator / participant node** | 公式docsではvalidatorはparticipant nodeを運用するNetwork上の役割。partyをhostし、そのpartyが関係するcontractを保存し、受け取ったtransaction viewを復号・検証する [CN1][cn1] [CN3][cn3] | 「validator」という語から、全validatorが全取引を読む通常のpublic chainを想定しない |
| **synchronizer** | sequencerが暗号化messageを順序付けて配信し、mediatorが関係validatorの確認を集約してcommit / rejectを宣言する。Damlの内容自体を検証する主体ではない [CN1][cn1] [CN3][cn3] | 順序・確認の調整と、業務ロジック・入力内容の正しさの検証を分ける |
| **Global Synchronizer** | Super Validatorが分散運営する公開synchronizer。2/3多数のBFTでmessage ordering・confirmationを行う。アプリはprivate synchronizerや複数synchronizerも選べる [CN6][cn6] [CN8][cn8] | Cantonアプリが常にGlobal Synchronizerを使う、または全処理をそこで行うとは限らない |
| **application provider** | on-ledger Damlとoff-ledger業務ロジック・認証を提供し、利用者のtransactionをsubmitする場合がある [CN5][cn5] | protocol上の検証と、off-ledger入力選定・認証・送信を正しく実装するproviderへの信頼を分ける |

## 2. 誰が何を読めるか

Cantonはtransactionを階層的なviewへ分解し、各viewを対応するinformee / witnessのparticipantへ暗号化して配信する。非関係partyはそのviewを受け取らず、synchronizerは暗号化payloadを復号しない。signatory、observer、controllerなど、アプリが指定した当事者は自分のviewを読み、結果を検証する。[CN2][cn2] [CN4][cn4]

ただし、秘密が全ての運営・計算主体から隠れるわけではない。

- **自分のvalidator:** hostするpartyのcontract dataを保存し、公式trust modelは、そのvalidatorが自分のpartyの全データを読めると明記する。第三者運営validatorを使う場合は、データを漏らさず正しくconsensusへ参加することをその運営者へ信頼する。外部party keyを使えばvalidatorによる署名を避けられるが、validatorのデータ閲覧までなくす説明ではない。[CN5][cn5]
- **取引当事者:** 当事者とそのvalidatorは権利に応じたviewを復号する。counterpartyが受け取った秘密を漏らさないことは、暗号だけで除去される信頼ではない。[CN5][cn5]
- **synchronizer:** sequencerは暗号化内容を読まないがrecipient routingを行い、mediatorはviewごとのinformeeとapprove / reject、confirmation policyを知る。公開privacy guideも時間、message size、activity patternからの推測を設計課題として挙げる。[CN2][cn2] [CN3][cn3]
- **application provider:** off-ledgerで注文受付、入力選定、認証、外部API連携を行う場合、その実装と非検閲性を信頼する。[CN5][cn5]

したがって、Cantonの強みは**必要な当事者間だけで平文を共有するsub-transaction privacy**である。ZKFMIのMPCが目指す「完全な注文入力を、許容結託数以下の各計算ノードにも渡さない」保証とは比較単位が異なる。Cantonでもoff-ledger applicationが外部MPCを実行して結果だけをDaml transactionへ入力する、またはMPCと連携するapplicationを作る余地があると設計上推論できる。これは実装確認ではないが、Canton全体の機能欠落ではなく、公開した標準経路とZKFMI現行経路の差として扱う。[CN1][cn1]

2020年whitepaperは、高度な暗号の負荷と、信頼を明示して共有範囲を限定する設計の選択を説明する。この標準経路とMPCの差には設計上の根拠があるが、Cantonアプリが外部MPC・ZKと連携できないという証拠ではない。[CN17][cn17]

## 3. 何を検証し、何を検証しないか

submitting participantはcommandをDaml engineで解釈し、transaction treeとroot hashを作る。関係participantは、自分宛てのviewについて、Damlの再実行結果、signatory / controllerのauthorization、必要な署名、自分のActive Contract Setに対するinput contractの有効性を確認して、署名付きapprove / rejectをmediatorへ返す。mediatorは確認policyを満たしたかを集計するが、Daml logicを独自に再検証しない。[CN3][cn3] [CN4][cn4]

これは、**提出されたDaml transactionの正しさ、権限、関係contractの二重使用防止**を強く扱う。Damlが業務ロジック・権限・multi-party agreementを扱うことから、注文contract、参加資格、与信枠、asset reservation、価格・時間優先やRFQの規則を実装し、同じtransactionへ束縛する構成は設計可能と推論できる。[CN1][cn1] 同じ市場アプリの実装・性能は確認していないが、ZKFMIだけが資格・予約・市場規則を実装できるとは言えない。

一方、次の命題は中核protocolだけから自動的には得られず、個別アプリと受付境界を確認する必要がある。

- off-ledgerで到着した全注文が省略なくcontract化・submitされたか。
- synchronizerへsubmitされる前の受付順、検閲、遅延操作がなかったか。
- matching transactionが参照した注文集合が、法令・市場規則上の対象集合と一致するか。
- 外部のKYC/KYB、与信原帳、保管口座、現金脚がDaml contractの状態と一致するか。
- transactionの非関係監査者が、秘密を受け取らずに同じ命題をportableなZK proofとして検証できるか。

これらをDaml contract、署名、observer、外部証明、MPC、受付commitmentで追加実装する余地はある。公開資料に実装記述がない項目を「Cantonは非対応」とは書かない。ZKFMI側も、zkPIが束縛する関数・入力集合・正本readbackと、受付前の検閲限界を具体的に示して初めて差を主張できる。

## 4. 原子性とDvPの境界

同じsynchronizerへassignされたcontractは、一つのDaml transactionで複数アプリ・複数participantをまたいで更新でき、全体がcommitするか全体がabortする。Global Synchronizerは、独立アプリのvalidatorが共通に接続できるsynchronizerとして、この構成を支える。[CN6][cn6] [CN7][cn7]

異なるsynchronizer上のcash contractとsecurities contractを使う公式DvP例では、両方を共通のsettlement synchronizerへreassignしてから、一つのDaml transactionで交換する。**settlement stepは原子的**だが、unassignmentとassignmentは別transactionであり、その間contractはpendingで利用できない。失敗時には解決までpendingに残り得る。[CN7][cn7]

よって、Cantonの原子性を次のように限定して比較する。

| 対象 | 確認した保証 | 確認していない一般化 |
| --- | --- | --- |
| 同一synchronizer上のDaml contract | 一つのtransaction内で全更新または無更新 | off-ledgerの銀行勘定や既存CSDの更新まで自動で原子的になること |
| 複数Cantonアプリ・participant | 関係validatorが同じsynchronizerへ接続し、必要なcontractを同じtransactionで扱えば原子的にcompose可能 | 任意の既存アプリが既に互いに接続・許可・法的受渡し済みであること |
| 複数synchronizer | 共通synchronizerへのreassignment後のsettlementを原子的に実行 | reassignment全体が一transactionで完了すること、pending中も資産を使えること |
| Canton外の台帳 | adapterやtokenizationでCanton contractへ意味を写す設計は可能 | 外部台帳のhold / commit / abort / finalityをCanton protocolだけで保証すること |

これはZKFMIにとって重要な反証である。DeFMI内で複数約定を一括決済する構成だけでは、Cantonに対する独自性にならない。差を出すなら、MPC入力、受付集合、資格・与信・予約、market rule、zkPIと決済をどの命題で結ぶか、外部脚がどのtrust boundaryに残るかを示す必要がある。

### Token Standardの予約型決済

CIP-0056はallocationによる資産予約と、settlement appの一つのDaml transactionによる全脚の決済を標準化する。予約・DvPは単なる実装可能性の仮説ではない。[CN26][cn26]

Splice参照interfaceはsenderのwithdrawと、sender・receiver・executorの共同承認によるcancelを区別する。withdrawは再allocationが間に合う `allocateBefore` より前なら決済を失敗させないSHOULDを記す。実行期限は `settleBefore` である。具体的な処理は各registryの実装に委ねるため、期限前なら無条件に取り下げられるとは一般化しない。[CN34][cn34]

## 5. 商用事例とネットワーク段階

| 事例 | 一次資料で確認した段階 | この比較で採用する証拠 | 一律に一般化しないこと |
| --- | --- | --- | --- |
| Global Synchronizer | 2024-07-01にgo-liveを発表。2026-06-29にはCanton 3.5のLogical Synchronizer UpgradeがMainnetでliveと発表 [CN9][cn9] [CN10][cn10] | 公開分散synchronizerが計画だけではなく運用・upgrade段階にある | 個別Cantonアプリの利用量、SLO、法的許認可、cross-app DvPの全件稼働 |
| Canton Network Pilot | 22独立dAppで350超の**simulated transactions**をTestNet上で実行したと公式発表 [CN11][cn11] | 複数アプリをまたぐ原子的compositionの大規模pilot | 本番資産、継続商用取引、参加数（要約45社、本文の役割別合計36社）の全社本番採用 |
| Broadridge DLR | Broadridgeは2025年8月に平均日次$280bnのrepo transaction処理を発表。Digital Assetの2024-06-21顧客事例は、2023年のCanton移行と、当時cashをoff-chainに残して証券ownershipを移す構成を説明 [CN12][cn12] [CN13][cn13] | Canton/Daml系の機関向けアプリが高い商用処理量を持つ証拠 | Global Synchronizerを使うcross-app DvP、on-chain cash、秘密CLOB、全Canton機能の本番証拠 |
| Tradewebの米国債取引 | 2026-07-01、tokenized U.S. TreasuryとUSDCxのreal-time transaction 1件をCantonのsynchronized settlementで完了とTradewebが発表。Fableは原URLのtimeout後、Canton公式転載で照合 [CN14][cn14] [CN29][cn29] | 実参加者・実資産カテゴリを伴うon-chain cash / security取引の具体例 | 市場全体の継続処理能力、全ライフサイクル、DTCC Tokenization Servicesの全面稼働 |
| MUFG / ProgmatのJGB repo | 2026-08-13、Cantonを使う実証協業の**開始**を発表。JGBとdigital moneyのDvP、repo lifecycle自動化を検討 [J3][j3] | 国内でZKFMIが狙う業務とCantonが直接重なる需要仮説 | 実証完了、商用化、法的受渡し、全機能の受入れ |

国内ではJSCC・みずほ・野村・DAのJGB担保PoC（2026-04-20発表）と、Progmat/DCCのWG（2026-05開始）も確認した。PoCとWGは試験・検討の異なる段階で、商用化・法的受渡し完了の証拠ではない。[J6][j6] [J7][j7]

Global Synchronizer Foundationは2025-09-22にCanton Foundationへ改名した。名称変更からgovernance変更を推定しない。[CN24][cn24] Broadridgeのcash off-chain説明は2024年の構成で、2026年現在の現金脚は未確認である。

Broadridgeの処理量、Tradewebの一取引、MUFGの実証開始は、それぞれ異なる証拠である。Cantonを利用する企業名の長い一覧を、同一構成・同一機能・同一成熟度の証拠として数えない。

## 6. ZKFMIとの比較判断

| 比較軸 | Cantonの確認範囲 | ZKFMIの現在位置 | 判断 |
| --- | --- | --- | --- |
| 誰が秘密を読むか | 非関係partyとsynchronizerから隠す。host validatorと権利を持つpartyは該当viewを読む | 新CLI経路では法人側でshare化し、各MPC nodeは完全入力を持たない。3 node以上の結託・通信観測等は別限界 | **ZKFMIが差を示し得る軸**。顧客が自社validatorへの開示を許すなら差は小さくなる |
| 計算者にも入力を隠すMPC | 確認した標準transaction経路は、関係participantがviewを復号してDamlを再実行 | 7 node・最大2不正のMPCを研究実装 | Canton上への外部MPC連携は設計候補で、実装未確認。同じ業務・結託・可用性・費用で比較するまで優劣未確定 |
| 参加資格・与信・予約 | Damlは一般に権限・状態・業務規則を表現・検証できる。Token Standardの予約型決済を確認。同等の資格・与信appは未確認で、外部正本との一致はadapter次第 [CN26][cn26] | DeKYX、予約、与信・保証枠を全経路へ結ぶ目標 | 機能名は独自性にならない。issuer、正本、失効、更新競合まで比較する |
| 市場規則 | 決定的なDaml logicを当事者がvalidationする一般構成を確認。同一のCLOB / RFQ appは未確認 | QOMM / OCLOBの具体的なRFQ・価格時間優先とzkPIを実装中 | 同じCLOB / RFQをCantonで構築できる可能性を認めるが、実装済みとは扱わない |
| 入力集合 | submitted transactionが参照するcontractとrootを束縛する。off-ledger受付の完全性はapplication境界 | 受付順・注文集合を証明へ束縛する目標。ただし受付前検閲は残る | どちらも「市場全体」を自動で証明しない。受付commitmentと対象集合を比較する |
| 決済原子性 | 同一synchronizerの一Daml transactionでcross-app DvP。cross-synchronizerはreassignment後のsettlementがatomic | DeFMI native rail内の複数約定一括決済smoke | Cantonは強い先行比較。外部台帳を含む同じ脚で検証するまでZKFMI優位を主張しない |
| 第三者検証 | 関係validatorが各viewを検証。auditorをobserver等にできる。非関係者向けportable ZK proofとは別 | zkPIで定義した命題の独立検証を目指す | ZKFMI候補価値。ただしCanton appへzkPI verifierを追加できるため、排他的ではない |
| operational trust | 自分/委託validator、counterparty、application provider、synchronizer、governanceへ役割別の信頼が残る | MPC node群、coordinator、readback、DeFMI validator、鍵・運営者の独立性が未受入 | ノード数で点数化せず、秘密・正しさ・可用性・検閲・復旧の主体を比較する |
| 成熟度・性能・法制度 | Mainnet、商用DLR、pilot・単発transaction・開始済みPoCが併存 | 単一hostの研究MVPとsmoke | Cantonの成熟度を過小評価しない。同条件の性能、顧客需要、個別業務の許認可は未確認 |

## 7. 戦略への反映

1. **優先比較:** RenegadeだけでなくCantonを、秘密範囲、market rule、予約、DvP、運用責任を通した最優先比較にする。
2. **追加機能としての選択肢:** CantonのDaml asset / cash application上へ、ZKFMIのMPC order processing、zkPI verifier、資格・予約adapterを追加する案をG2の接続候補にする。
3. **接続先としての選択肢:** Global Synchronizerまたはprivate synchronizerを使い、Canton-native contractへsettlement instructionを渡す案を、既存台帳adapterと同じ受入れ条件で評価する。
4. **全面実装の代替案:** 同じ市場をCanton + 外部MPC / ZKで構築できる場合、独立DeFMI L1の採用上の優位が縮む可能性を認める。ただし顧客需要、性能、費用、独立運営、法的受渡しを同条件で測っていないため、現時点で置換を決めない。
5. **既存目標の維持:** 独立DeFMI L1、QOMM、OCLOB、DeCCPの実装・研究目標は取り消さない。現行経路を基準に継続取引・復旧を完了し、Canton接続や代替比較はその後のG2で扱う。
6. **主張の修正:** 「自分たちだけが市場を証明する」を棄却し、顧客が必要とする秘密境界と、受付・資格・予約・規則・決済を一つの検証契約へ結ぶ具体差だけを提案する。

## 8. 未確認事項

- Canton上でZKFMIと同一の秘密CLOB / RFQ、入力集合、資格、与信、予約、zkPI命題を実装した結果。
- managed validatorを含む実構成で、注文原文、contract、recipient metadata、通信patternを各主体が読める範囲。
- ZKFMIとCantonで同じ入力・秘密条件・障害条件を使った遅延、処理能力、費用、復旧時間。**性能優位は未確認。**
- Global Synchronizerまたは商用applicationへの接続契約、API利用権、費用、SLO、運営者の独立性。
- 日本のJGB repo、デジタル証券、現金脚、取引市場、清算について、各主体に必要な許認可と法的finality。**法的許認可は未確認。**
- 顧客がvalidatorやcounterpartyへの開示で足りるのか、計算者にも隠すMPCを必要とするのか。**顧客需要は未確認。**

本調査ではCanton nodeや競合applicationを実行せず、新規benchmark、顧客連絡、外部書込みを行っていない。公式pilot PDFはWeb取得時にサイズ制限で直接openできなかったため、同内容を明記する取得可能な公式pilot完了発表[CN11][cn11]を根拠にし、PDFを読了資料数へ含めていない。

## 9. 一次資料

| ID | 一次資料 | 主に確認したこと |
| --- | --- | --- |
| CN1 | [Canton Network Docs: Architecture Overview][cn1] | validator / participant、synchronizer、Daml、保存・実行・順序付けの役割 |
| CN2 | [Canton Network Docs: Privacy Model Explained][cn2] | view単位の開示、divulgence、timing / size / activity pattern、validatorの閲覧 |
| CN3 | [Canton Network Docs: Smart Contract Consensus][cn3] | Proof of Stakeholder、再実行、authorization、ACS、mediatorの限界 |
| CN4 | [Canton Network Docs: Transaction Lifecycle][cn4] | transaction tree、root hash、view暗号化、submitからcommitまで |
| CN5 | [Canton Network Docs: Trust Model Overview][cn5] | validator、counterparty、application provider、synchronizer、governanceの信頼 |
| CN6 | [Canton Network Docs: Multi-Synchronizer Architecture][cn6] | private / Global Synchronizer、contract assignation、reassignment |
| CN7 | [Canton Network Docs: Cross-Synchronizer DvP Example][cn7] | common synchronizer上のatomic settlementとpending境界 |
| CN8 | [Global Synchronizer Foundation / Splice Docs][cn8] | Super Validator、2/3 BFT、validator、governanceの運用役割 |
| CN9 | [Global Synchronizer and Canton Coin Go Live、2024-07-01][cn9] | 公開分散infrastructureのgo-live |
| CN10 | [Logical Synchronizer Upgrades、2026-06-29][cn10] | Canton 3.5 / LSUのMainnet稼働発表 |
| CN11 | [Canton Network Pilot完了、2024-03-12][cn11] | TestNet、22 dApp、350超のsimulated transaction |
| CN12 | [Broadridge DLR、2025-09-10][cn12] | 2025年8月の平均日次$280bn処理発表 |
| CN13 | [Digital Asset: Broadridge customer story][cn13] | Canton移行、Daml、off-chain cashを含むDLRの業務境界 |
| CN14 | [Tradeweb、2026-07-01][cn14] | tokenized U.S. Treasury / USDCxの単一real-time transaction |
| J3 | [MUFG: JGB repo実証開始、2026-08-13][j3] | Canton利用、DvP・lifecycleの実証計画と段階 |

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
