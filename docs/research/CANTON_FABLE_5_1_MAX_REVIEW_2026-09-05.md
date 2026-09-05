# Cantonを含む18対象比較の再評価 — Claude Fable 5.1 / effort max

確認日: **2026-09-05**（本セッションで確認できる実時刻: 最初のWeb取得の保存 11:56:32 UTC、curl取得 12:02〜12:03 UTC、ローカル確認 12:07:04 UTC、改訂時の追加取得 12:30 UTC。先行担当の作業時刻は含まない）。担当: Claude Fable 5.1（model id `claude-fable-5-1`、effort 最大、サブエージェント・代替モデルなし）。Codex親タスクからの委任で、[Canton以外の初回サーベイ][survey]、[比較判断 v1.1][decisions]、[戦略 v1.1][strategy]、[基準ファイル][baseline]、[Canton追加調査 v1.0][canton]を読み取り専用で参照し、Cantonを加えた18対象で比較判断と戦略を再評価した。**本書の書き込み先はこのファイルだけ**である。コード、Cargo、lock、他リポジトリ、README、既存文書は変更していない。commit、push、外部連絡、実験、ビルド、テスト、認証設定の変更も行っていない。

前提とするユーザー決定: **ZKFMI基盤全体からAethel依存をなくす。** Aethelは基盤を利用する業務アプリの向きに揃える。親タスクがコード分離を進めているため、本書はこの方針を比較・戦略の前提に置くが、**分離実装が完了したとは書かない**（12:07 UTC時点で defmi / dekyx / deccp の作業ツリーに `*-aethel` crate の未コミット削除差分があることを確認した。完了の証拠ではない）。

親タスク統合注（2026-09-05）: 技術判断はFableのレビューを維持した。長い原文引用を要約に替え、Cantex一例から全アプリをAMM中心とする一般化を除いた。基盤分離とOCLOBの後続検証は親タスクの比較v1.2と別snapshotで更新する。

改訂 v1.1（12:30 UTC）: 親タスクの指摘3点を反映した。(1) allocationの取り下げに関する記述を、CIP-0056本文とSplice参照interfaceの原文に合わせて修正（2.4節、3.3節、5.1節、9節）。(2) 一次資料の参照件数と取得成功件数を区別して再集計（1節、9節）。(3) 取得時刻を本セッションで確認できる実時刻に限定（冒頭、1節）。結論は変更していない。

## 0. 結論

1. **Cantonは最優先の比較対象であり、同時に接続先・実装先候補である。** この位置付け（先行調査の結論、比較判断C09、戦略S04）を維持する。Canton公式資料は、必要な当事者だけが平文を読む可視性、当事者validatorによる再実行検証、同一synchronizer上の一transactionでの複数アプリDvPを説明し、Mainnet運用・商用アプリ・国内PoCの一次資料がある。[CN1][cn1] [CN3][cn3] [CN7][cn7] [CN10][cn10] [J3][j3] [J6][j6]
2. **「計算主体にも入力を渡さない」秘匿計算はCanton protocolの範囲外であることを、記載の不在ではなくCanton自身の設計文書で確認した。** 2020年のCanton whitepaperは、MPCまでの高度な暗号は計算負荷が大きく、より強い信頼仮定で可視性を限定する方針を採ると明記する。2025年のCanton公式blogもZKPを汎用privacyには実験段階と位置付ける。[CN17][cn17] [CN18][cn18] これはZKFMIが差を示し得る軸の根拠になる。一方、Canton上のapplicationが外部MPC・ZKと連携することを禁じる記述はなく、「Cantonでは不可能」とは言えない。
3. **Cantonには「事前予約→原子的決済」の標準もある。** Canton Network Token Standard（CIP-0056）は、期限まで資産をlockするallocationと、settlement appが一transactionで全移転を実行する「全部か無か」の決済を定める。予約・DvPの機能名はZKFMIの独自性にならない。差は、予約と注文の対応を照合ノードから隠すか、registry・app・validatorが平文で読むかにある。[CN26][cn26]
4. **国内では、ZKFMIが狙う業務の隣接領域でCantonのPoCが3件並走している。** JSCC・みずほ・野村（2026-04-20、JGB担保管理）、MUFG・Progmat（2026-08-13、JGBレポ）、Progmat/DCCのWG（2026-05開始、報告書2026-10目標）。いずれも開始・検討段階であり、商用化・法的受渡し完了の証拠ではない。[J6][j6] [J3][j3] [J7][j7] 戦略上は、同じ業務での正面代替を避け、計算主体からも隠す要件がある部分に絞る。
5. **C01〜C09は全て維持する。修正は根拠の追加と表現の精密化に限る。** S01〜S06、G0〜G4も方向を維持し、G0の判定質問、G2のCanton受入経路、S02のadapter契約にToken Standardとの対応を加える。
6. **ローカル証拠は基準時刻より進んだが、smoke_onlyのままである。** 継続取引契約 `oclob-native-cycle-v1` は基準時刻後の実行005・006で2回の照合・決済を完了した。単一ホスト、独立運営者なし、WANなし、設定済みDeFMI読取サービスへの信頼は変わらない。[L16][l16] [L7][l7]
7. **先行調査に致命的な誤りはない。** 修正すべきは、Global Synchronizer Foundationが2025-09-22にCanton Foundationへ改名済みであること、「network of networks」の出典、pilot参加社数の数え方、Tradeweb発表の取得可能URLである（6節）。

## 1. 検証の方法

- 先行調査が引く一次資料15件（CN1〜CN14、J3）のうち14件を開いて原文と照合した。CN14（investors.tradeweb.com）はtimeoutで取得できず、同文のCanton公式転載（CN29）で内容を確認した。加えて、Daml ledger modelのprivacy章、Canton whitepaper、Canton公式blog・FAQ、Super Validator構成、Token Standard（CIP本文とSplice参照interface）、Canton Foundation、DTCC・Visa・Tradeweb発表、JSCC/みずほ/野村・Progmat関連の国内発表、金融庁PIP設置ページを新規に23件開いた（CN15〜CN34の20件、J6〜J8の3件。9節）。
- 件数の内訳（URLの重複なし）:

| 区分 | 件数 | 内訳 |
| --- | --- | --- |
| ID付きで参照した一次資料 | 38 | 既存15（CN1〜CN14、J3）+ 新規23（CN15〜CN34、J6〜J8）。URLは全て異なる。CN14とCN29は同一発表の別URLなので、文書としては37件 |
| うち開封・照合できたもの | 37 | CN14以外の全て。CN15・CN16はcurlで本文抽出、J3・CN17はPDFをダウンロードしてローカル抽出（CN17は1〜3ページのみ）、J8・CN34は改訂時（12:30 UTC）にcurlで開封 |
| うち取得できなかったもの | 1 | CN14（timeout）。CN29で代替 |
| IDを付けずに開封したページ | 8 | canton.network（トップ、global-synchronizer）、docs.cantex.io（トップ）、docs.sync.global / docs.dev.sync.global（token standard）、progmat.co.jp/news、fsa.go.jp（2026-02-27「FinTech実証実験ハブ」支援決定）、docs.daml.com/daml/intro/7_Composing.html（目次のみ）。本文の根拠には使っていない |
| 取得を試みて失敗し、IDも付けないURL | 7 | tradeweb.com（403）、businesswire.com（403）、jpx.co.jp JSCC（403）、nomuraholdings.com（403）、docs.digitalasset.com 3.3・3.4のledger-privacy（404）、MUFG英語版PDF（未読） |

- 取得時刻（本セッションの記録で確認できるものだけ）: 最初のWeb取得バッチの保存ファイル 11:56:32 UTC（バッチの開始時刻は未記録）、2回目 11:59:00 UTC、3回目 12:00:03 UTC、curlによるDaml docs取得 12:02:59〜12:03:02 UTC、ローカルのgit・ledger確認（`date -u`）12:07:04 UTC、改訂時のCIP-0056・Splice・金融庁ページ取得 12:30:14〜12:30:15 UTC。
- 取得手段: Web取得ツール、`curl`によるHTML本文取得（Daml docsは目次だけが返るため本文をcurlで抽出）、PDFのローカル抽出（MUFG発表、Canton whitepaper）。
- 取得できなかったもの: `investors.tradeweb.com`（timeout）と `tradeweb.com`（403）は、同文を掲載するCanton公式の転載[CN29][cn29]で代替した。JSCC（jpx.co.jp）と野村の発表ページは403のため、Digital Asset公式blogの同文発表[J6][j6]で代替した。`docs.digitalasset.com` 3.x のledger model privacyページは404で、`docs.daml.com` 2.10.6の同章[CN15][cn15]を使った。金融庁の2026年2月の決済高度化プロジェクト支援決定ページは特定できず、MUFG・DAの発表の記載に留めた。
- 本書は公開資料の照合である。Cantonノードの実行、競合アプリの利用、性能測定、コード監査、顧客連絡は行っていない。**速度・費用・優位性の数値は測っていないので主張しない。** 各社の自己申告値は出典と時点を付けて引用するだけで、順位付けに使わない。

## 2. Cantonの位置付け（一次資料で確認した範囲）

### 2.1 一つの製品として扱わない層構造

| 層 | 一次資料で確認したこと | 比較上の意味 |
| --- | --- | --- |
| Daml（契約言語・ledger model） | privacyは「need-to-know basisに基づき、subtransaction単位で提供」。partyは自分が利害を持つcontractに影響する部分と、その帰結だけを知る。informee / witness / projection / divulgenceで開示範囲を定義する [CN15][cn15] | 可視性は業務ロジック（signatory / observer / controller）で決まる。protocolがそれを強制する |
| Canton protocol | transactionをviewへ分解し、view単位で受信者を限定して暗号化。sequencerは「順序付けと宛先配送の二つの機能だけ」を持ち、内容は暗号化されて読めない。mediatorも内容を学ばず、各viewのinformee一覧と承認/拒否だけを受け取る [CN16][cn16] [CN4][cn4] | 順序・確認の調整と、内容の検証を分離する |
| validator / participant node | partyをhostし、hostするpartyが利害関係者であるcontractだけを保存する。Daml再実行はここで行う [CN1][cn1] | 「validator」は全台帳を検証する公開chainのノードではない |
| synchronizer（sequencer + mediator） | 暗号化envelopeと順序付け・宛先用のmetadataだけを見る。Daml logicを独自に検証しない [CN3][cn3] [CN33][cn33] | 内容の正しさは関係participantの確認に依存する |
| Global Synchronizer | Super Validator（SV）が分散運営する公開synchronizer。各SVはsequencer、mediator、CometBFT ordererを運用し、ブロック生成には「SVノードの2/3超の合意」が必要。許容故障数は `f = floor((n-1)/3)` [CN22][cn22] [CN8][cn8] | 順序付けのBFT合意であり、業務内容の検証ではない |
| Canton Coin | 利用者はsynchronizer trafficの支払いにCanton Coinをburnし、参加に応じてmintされる（burn-mint equilibrium） [CN23][cn23] [CN20][cn20] | Global Synchronizer利用には運用費用（traffic）が発生する。額は未確認 |
| Canton Foundation | 旧Global Synchronizer Foundation。2025-09-22に改名（名称変更のみ）。Global Synchronizerの開発・governanceを担い、自らSVノードも運用 [CN24][cn24] [CN31][cn31] | governanceへの信頼は残る（trust modelが明記） [CN5][cn5] |
| application provider | on-ledger Damlとoff-ledgerロジック・認証を提供し、利用者のtransactionをsubmitする場合がある。「検閲しないこと」「正しいcontract logicを実装すること」を信頼する [CN5][cn5] | 受付前の検閲・省略はprotocolの外にある |

### 2.2 誰が何を読むか（need-to-know可視性）

| 主体 | 読めるもの | 読めないもの | 出典 |
| --- | --- | --- | --- |
| 当事者（signatory / observer / controller / choice observer） | 自分がinformeeであるview（action、帰結、必要な文脈）。divulgenceで非stakeholderのcontractを見ることがある | 自分が関与しない部分は「payloadも、関与participant・partyのmetadataも」見ない | [CN15][cn15] [CN2][cn2] |
| 当事者をhostするvalidator | trust modelは「Your validator sees all your data and could block you」と明記。external party keyで署名を自分で持てるが、閲覧まで防ぐ説明ではない | — | [CN5][cn5] |
| counterparty | 共有されたviewの平文。「共有された秘密を漏らさないこと」を信頼する | — | [CN5][cn5] |
| application provider（off-ledger部分を含む） | 実装次第。注文受付・照合エンジンをoff-ledgerで運営する場合、その主体は平文を扱い得る（例: Cantexは「swapをoffchainで提出」し、CaviarNineが照合エンジンを提供する） | — | [CN5][cn5] [CN30][cn30] |
| sequencer | 暗号化envelope、宛先、順序、サイズ、タイミング | 内容（session keyはinformee participantの公開鍵で暗号化） | [CN4][cn4] [CN16][cn16] |
| mediator | 各viewのinformee一覧、confirmation policy、各participantの承認/拒否 | 内容。参加者同士の身元も隠す役割を持つ | [CN16][cn16] [CN3][cn3] |
| 非関係party・その他のvalidator | 何も受け取らない | — | [CN2][cn2] |
| 監査者 | observer等として設計に組み込めば該当viewの平文。秘密を受け取らずに命題だけを検証するportableな証明は、公開資料の標準経路には含まれない | — | [CN2][cn2] |

公式docsは、内容を見なくても「いつ取引が起きたか、transactionのサイズ、活動のパターン」から推測され得ることを「Timing Attacks」として明記し、batchingやnoise付加を設計側の対策として挙げる。[CN2][cn2] Canton FAQも、基盤運営者は「順序と一貫性のために必要な限られたmetadataだけ」を見ると説明する。[CN20][cn20]

### 2.3 Global Synchronizer と参加条件

- 2024-07-01にGlobal SynchronizerとCanton Coinのgo-liveを発表。参加31組織の中にSBI Digital Asset Holdings、Tradeweb、Broadridge、Ownera等が含まれる。[CN9][cn9]
- 2026-06-29、Canton 3.5でLogical Synchronizer UpgradeがMainnetで稼働。Mainnet開始以来、protocol版の更新が4回あったと自己申告。[CN10][cn10]
- SV数: Visaは2026-03-25の自社発表で「40 Super Validatorsの一つ」として参加を公表。[CN28][cn28] Canton側の「45+」発表（2026-04-04とされる）は二次転載でしか確認できず、一次資料未確認（8節）。
- validator参加: FAQは「公開ネットワーク。誰でもvalidator nodeを申請できる（現在はsponsorshipが必要）」と説明する。[CN20][cn20] G2でCantonを評価する場合、評価用ネットワークの利用条件（DevNet / TestNet、sponsor、費用）を先に確認する必要がある。
- governance: Canton Foundation（旧GSF）が担う。Linux Foundationは2024-07-01と2025-03-19の発表でGSFを支援すると記載。[CN9][cn9] [CN25][cn25] 改名後のLinux Foundationとの関係は一次資料で未確認（8節）。

### 2.4 原子的composabilityとDvPの境界

| 対象 | 確認した保証 | 境界 | 出典 |
| --- | --- | --- | --- |
| 同一synchronizer上のcontract | 「Daml transactionは単一のsynchronizer上で実行され、全入力contractはそのsynchronizerにassignされていなければならない」。その範囲で複数アプリ・participantをまたぐ更新が全部commitか全部abort | off-ledgerの銀行勘定・既存CSDの更新は含まない | [CN6][cn6] |
| 複数synchronizer | cash contractとsecurities contractを共通のSettlement Syncへ再assignした後、一つのDaml transactionで交換。「両方の移転が起きるか、どちらも起きないか」 | reassignmentは「二つのsynchronizer上の二つのconfirmation requestを伴う非原子的手続き」。pending中は使えず、assignment失敗時は解決までpendingに残る | [CN7][cn7] [CN6][cn6] |
| Token Standard（CIP-0056、2025-03-31承認、Final） | holder が特定の決済のために資産をallocationとしてlock。全allocationが揃うとsettlement appが一transactionを提出し「全移転が決済されるか、どれも決済されない」。lockは決済期限まで（「allocations are only valid until that deadline」「become available again to their owner immediately thereafter」）。**CIP本文にはwithdraw / cancel / revokeの定義はなく**、metadata key `splice.lfdecentralizedtrust.org/reason` の説明に「withdrawing an allocation」が例示されるだけ。Splice参照実装のDaml interface `Splice.Api.Token.AllocationV1` には、sender（送り手）だけが制御する `Allocation_Withdraw`（「`settlement.allocateBefore` 期限前なら決済を失敗させない」SHOULD）と、sender・receiver・executorの共同制御の `Allocation_Cancel`（通常はexecutorへ委任）が定義される。個別registryの実装（`allocation_withdrawImpl`）の挙動は未確認 | allocationの内容（資産・数量・相手）はregistry・app・所有者に平文。照合ノードから予約と注文の対応を隠す仕組みは標準の範囲外 | [CN26][cn26] [CN34][cn34] |
| Canton外の台帳 | adapter / tokenizationで意味を写す設計は可能 | 外部台帳のhold / commit / abort / finalityはCanton protocolの保証外 | [CN7][cn7] |

これはZKFMIにとって二つの反証を含む。第一に、DeFMI内で複数約定を一括決済する構成は独自性にならない（先行調査の結論を維持）。第二に、**「事前予約」も標準化済み**である。OCLOBの `ReservationAdmission` / `ReservationPermit` の分離（照合ノードへ台帳識別子を渡さない）は、Token Standardのallocationにはない秘密境界であり、差として説明できる。ただし機能の存在ではなく「誰が予約の対応を読むか」で説明する。[L1][l1] [L4][l4]

### 2.5 秘匿計算との違い

Cantonの可視性限定は、**データを配らないこと**と**受け取った当事者が平文で検証すること**で成り立つ。これは記載の不在からの推論ではなく、Canton自身の設計文書が述べる選択である。

2020年のwhitepaperは、高度な暗号でMPCまでの秘匿性を実現できる一方、計算負荷が拡張性を制約すると説明する。そのためCantonは、参加者への信頼を明示してデータの共有範囲を限定する設計を選ぶ。[CN17][cn17]

2025-05-12のCanton公式blogは、ZKPを「institutional trustの重みを支えるには限界がある」「汎用のsmart contract privacyには依然として実験段階」と位置付け、Cantonは「full auditabilityを備えたsmart contract privacy」を提供すると述べる。[CN18][cn18] 2025-08-14の公式blog、privacy model docs、FAQのいずれにもZK・MPC・FHE・TEEをprotocolに用いる記述はない。[CN19][cn19] [CN2][cn2] [CN20][cn20]

| 比較単位 | Cantonの標準経路 | OCLOB新CLI/Docker経路 | 判断 |
| --- | --- | --- | --- |
| 注文原文を平文で持つ主体 | 注文者、そのvalidator、契約上の相手方（とそのvalidator）、off-ledger照合を運営するapp provider | 注文者（法人側）。7ノードは各自の分割だけを持ち、調整役は原文を受け取らない。3ノード以上の結託で復元される [L4][l4] | ZKFMIが差を示し得る軸。顧客が自社validator・app運営者への開示を許すなら差は消える |
| 正しさの検証 | 関係participantがviewを復号してDamlを再実行。mediatorはDaml logicを独自検証しない [CN3][cn3] | MPC出力に対する共同証明、5検証ノードによる証明全文の検証 [L4][l4] | どちらも「提出された入力集合」に対する正しさ。受付前の省略・検閲はどちらも自動では解消しない |
| 第三者検証 | observerとして設計すれば平文で検証。秘密を受け取らないportable proofは標準経路にない | zkPIで定義した命題の検証を目指す。verifierが検証する命題とreadback依存の明示が条件 [L13][l13] | ZKFMIの候補価値。ただしCanton appにverifierを追加する構成は排除できない |
| 「Cantonに無い」と言える範囲 | protocolの標準経路に、計算主体から入力を隠す仕組みは含まれない（設計文書が明記） | — | 断言可 |
| 「Cantonに無い」と言えない範囲 | Canton上のapplicationが外部MPC / ZK / TEEと連携すること、将来protocolに追加されること | — | 断言不可。公開実装は未確認（8節） |

秘匿性の比較は、ノード数や方式名ではなく「原文を読む主体の一覧」「結託条件」「通信・タイミング観測」「復旧時に開く情報」で行う。この方針は比較判断C02・C04と一致する。

### 2.6 運用・採用の成熟度

| 事例 | 一次資料で確認した段階 | 分類（サーベイ2節の表記） | 一般化しないこと |
| --- | --- | --- | --- |
| Global Synchronizer Mainnet | 2024-07-01 go-live。2026-06-29 Canton 3.5 LSU稼働、protocol更新4回 [CN9][cn9] [CN10][cn10] | Mainnet公表 | 個別アプリの利用量、SLO、法的許認可 |
| Canton Network Pilot | 2024-03-12、TestNet上で22 dApp・350超の**simulated** transaction。本文の役割別参加者は15+13+4+3+1=36社、ページ記述は45社 [CN11][cn11] | 採用・実証 | 本番資産、継続商用取引 |
| Broadridge DLR | DLRは2023年にCantonへ移行、cashはoff-chain、証券の所有権をsmart contractで移転（DA顧客事例、2024-06-21）。2025-09-10発表は2025年8月の平均日次$280bn・月間$5.9Tのrepo処理を自己申告。同発表のCanton言及は市場データ配信アプリ [CN13][cn13] [CN12][cn12] | 商用事例公表 | Global Synchronizer利用、on-chain cash、秘密CLOB |
| Tradeweb米国債取引 | 2026-07-01、Franklin TempletonがVirtuへtokenized U.S. TreasuryをUSDCxと交換した1件。「synchronized on-chain settlement」。DTCC Tokenization Servicesは「later this year」に予定 [CN29][cn29] | 商用事例公表（単発） | 継続処理能力、全ライフサイクル |
| DTCC | 2025-12-17、SECのNo-Action Letterを受け、DTC保管の米国債の一部をCanton上でmintする計画。MVPは2026年上半期に「controlled production environment」で予定 [CN27][cn27] | 採用・実証・予定 | MVP完了・商用開始（8節） |
| Visa | 2026-03-25、SVとして参加。「40 Super Validatorsの一つ」と記載 [CN28][cn28] | 採用公表 | Visa製品のCanton上稼働 |
| JSCC・みずほ・野村・DA | 2026-04-20、JGBデジタル担保管理のPoC開始。振替法・金商法上のJGBの法的地位を維持しつつ既存システムとCantonを接続、24/365・クロスボーダーを検証。2026-02にFSAのPIPに選定 [J6][j6] | 採用・実証（開始） | 完了、商用化、終了時期（一次資料に記載なし） |
| MUFG・DA・Progmat・Secured Finance | 2026-08-13、JGBレポのオンチェーン化PoC協業開始。「JGBの振替国債としての法的性質は維持しつつ、ブロックチェーンと連動して口座管理機関が有する振替口座簿を更新」「デジタルマネーとしてトークン化預金やステーブルコインの利用を検討」。FSA PIPの支援決定（2026-02）を受けた実証の一部 [J3][j3] | 採用・実証（開始） | 完了、商用化、法的受渡し |
| Progmat/DCC「トークン化国債・オンチェーンレポWG」 | 2026-05開始、法律・会計・税務・実務・技術の観点で検討、報告書は2026-10目標（Secured Financeの参画発表による） [J7][j7] | 検討開始 | 技術基盤の選定、商用化 |
| Canton上の取引アプリ | Cantexは「Canton-native AMM」で「swapをoffchainで提出」しCantonで原子的に決済。CaviarNineが照合エンジンを提供。CLOBは一次資料で未確認。Hydra XはCanton上で仕組債トークン（2025-04-17）を発表、CLOBの記述なし [CN30][cn30] [CN32][cn32] | 仕様確認 | 秘密CLOBの稼働、照合主体が注文を読まないこと |

ZKFMI側の現在位置は、単一ホストの研究MVPで、2約定一括決済と継続取引2回のsmoke_onlyである（7節）。Cantonの成熟度との差は大きく、過小評価しない。ただしCantonの各事例も「開始」「単発」「自己申告」「予定」を区別して読む。

## 3. 比較表へそのまま使える行

### 3.1 初回サーベイ3節の表形式（| 対象 | 主に競合する仕事 | 秘密・信頼の境界 | 確認できた段階と比較上の意味 |）

| **Canton Network** | 機関向けDamlアプリの台帳、synchronizerによる順序・確認、Token Standard（CIP-0056）による保有・移転・allocation、同一synchronizer上の一transactionでの複数アプリDvP | 関係partyとそのvalidatorだけが自分のviewを平文で読み、synchronizerは暗号化envelopeと順序付け用metadataだけを見る。自分のvalidatorは自分の全データを読む。計算主体から入力を隠す秘匿計算はprotocolの範囲外であることを設計文書が明記。off-ledger照合を行うapp providerは平文を扱い得る | Global Synchronizer Mainnet（2024-07、LSU 2026-06）、Broadridge DLRの自己申告処理量（cash off-chain）、Tradewebの単一取引（2026-07）、DTCC MVP計画、Visa SV参加、国内はJSCC/みずほ/野村（2026-04）とMUFG/Progmat（2026-08）のPoC開始。**最優先比較・接続候補** [CN1][cn1] [CN5][cn5] [CN7][cn7] [CN17][cn17] [CN26][cn26] [CN29][cn29] [J3][j3] [J6][j6] |

### 3.2 比較判断 v1.1 2節の表形式（| 対象 | 公開資料から確認できる役割 | 秘密・検証の重要な境界 | 成熟度の扱い | ZKFMIでの扱い |）

| **Canton Network** | Damlアプリ、partyをhostするvalidator、順序・確認を調整するsynchronizer、SVが運営するGlobal Synchronizer（CometBFT、2/3超）、Token Standardのallocationによる予約型決済、cross-app atomic transaction | 非関係partyとsynchronizerはpayloadを読まない。host validatorと関係partyは該当viewを読む。allocationはregistry・app・所有者に平文。MPC/ZKで計算主体にも入力を隠す方式は設計上採らないと自ら明記（記載不在ではない） | Global Synchronizer Mainnet、商用DLR（自己申告）、TestNet pilot（simulated）、単発の実取引、開始済みPoC（米DTCC、国内3件）を分離 | **最優先比較・基盤/接続候補**: 秘密境界（原文を読む主体一覧）、入力集合、市場規則、予約の可視性、DvP、運用費用（traffic）、参加条件（sponsorship）、運用責任を同じ業務で比較 [Canton追加調査][canton] [CN17][cn17] [CN22][cn22] [CN26][cn26] |

### 3.3 Canton追加調査6節の比較軸表へ追加・置換する行

| 比較軸 | Cantonの確認範囲 | ZKFMIの現在位置 | 判断 |
| --- | --- | --- | --- |
| 計算者にも入力を隠すMPC（置換） | whitepaper（2020）はMPCを含む高度な暗号を計算負荷の理由で採らず、より強い信頼仮定で可視性を限定すると明記。2025年blogはZKPを汎用privacyには実験段階と位置付け。標準経路は関係participantがviewを復号してDamlを再実行 [CN17][cn17] [CN18][cn18] [CN3][cn3] | 7ノード・最大2不正のMPCを研究実装。継続取引2回のsmoke_only [L16][l16] | **ZKFMIが差を示し得る軸で、根拠はCantonの設計上の選択。** Canton app層への外部MPC連携は設計候補で実装未確認。同じ業務・結託・可用性・費用で比較するまで優劣未確定 |
| 参加資格・与信・予約（置換） | Token Standardのallocationが「期限までlock、settlement appの一transactionで全部か無か」を標準化（senderによるwithdrawとsender・receiver・executorによるcancelはSplice参照interfaceで定義、CIP本文には未記載）。allocationの資産・数量・相手はregistry・app・所有者が平文で読む [CN26][cn26] [CN34][cn34] | ReservationAdmission / ReservationPermitを分け、照合ノードへ台帳識別子と資産・側を渡さない。issuer署名はreadbackへの信頼 [L1][l1] | 予約機能の有無ではなく、予約と注文の対応を誰が読むかで比較する。issuer、正本、失効、更新競合まで比較する |
| 運用費用・参加条件・governance（追加） | synchronizer trafficはCanton Coinのburnで支払う。validator参加はsponsorshipが必要。governanceはCanton Foundation（旧GSF）とSVの投票 [CN23][cn23] [CN20][cn20] [CN24][cn24] | 自前MPCノード群・DeFMI検証ノードの運用費用は未測定。独立運営者なし | 両者とも費用は未測定。G2/G3で同じ業務の原価を積み上げる前に、Canton側の評価用ネットワークの利用条件を確認する |
| 秘密CLOB / RFQ（追加） | Cantexは「swapをoffchainで提出」しCantonで原子的決済するAMM。CLOBは一次資料で未確認。照合主体が注文を読まない設計の公開実装は未確認 [CN30][cn30] | OCLOB（価格・時間優先、受付順5-of-7）、QOMM（RFQ）を研究実装 [L4][l4] [L10][l10] | Cantonのecosystemは「off-chainの運営者側照合 + on-chainの原子的決済」の型を採る。ZKFMIが差を出すのは照合主体の可視性であり、市場方式そのものではない |

## 4. C01〜C09の維持・修正判断

| ID | 判断 | 根拠の追加・表現の修正 |
| --- | --- | --- |
| C01 MPC・ZK・約定計算の証明は独自性の根拠にならない | **維持** | 変更なし。CantonはDaml規則を関係participantが再実行検証する [CN3][cn3]。Renegade・Prime Matchの先行は既存根拠のまま |
| C02 比較軸は「どの秘密を誰から隠すか」と「どの規則・状態を同じ取引に結ぶか」 | **維持・根拠追加** | Cantonの「誰が読むか」を主体一覧（2.2節）で固定する。Cantonが秘匿計算を採らない理由を設計文書で示せるため、この軸の差は「記載不在」ではなく「設計上の選択の違い」と書ける [CN17][cn17] [CN18][cn18]。同時に、顧客が自社validator・app運営者への開示を許す場合に差が消えることを、G0の判定質問に落とす |
| C03 価格形成方式の違いは普遍的な優劣ではない | **維持** | 今回確認したCantexの公式説明はAMMであり、この一例からCanton上の全取引アプリの方式は推定しない。同等の秘密CLOBは未確認。市場方式の比較にCantonを加えるときは「Canton上の個別アプリ」を対象にし、Canton自体を市場方式と扱わない [CN30][cn30] |
| C04 秘密分散の安全性でZKFMIが一律に強いとは言えない | **維持** | 変更なし。Cantonとの比較でも、SVの `f = floor((n-1)/3)` は順序付けの故障許容であり、秘匿性の保証ではないと注記する [CN22][cn22] |
| C05 指図の標準化・DvP・台帳間調整に先行基盤がある | **維持・根拠追加** | CantonのToken Standard（allocation、settlement executorの一transaction、期限）を先行基盤の具体例として追加する [CN26][cn26]。zkPIの追加価値は「予約と注文の対応を照合ノードから隠したまま、指図に束縛する」ことに限定して書く |
| C06 ZKFMIは研究MVPで、機関向け完成品との成熟度差がある | **維持・証拠更新** | 継続取引2回のsmoke_only（実行005・006、artifact SHA-256 `dcaca99d…0300`）を追加できる。独立運営・WAN・本番安全性はfalseのまま [L16][l16] [L7][l7]。Canton側はDTCC・Visa・国内PoC3件を追加（2.6節） |
| C07 日本では既存証券業務と現金脚への接続が採用条件になり得る | **維持・根拠追加** | JSCC/みずほ/野村のJGB担保PoC、Progmat/DCC WGを追加 [J6][j6] [J7][j7]。MUFG PoCはJGBを振替国債のまま扱い、現金脚にトークン化預金・ステーブルコインを検討する構成であり、ZKFMIの現金脚候補（Kinexys / Fnality / Partior）とは制度上の前提が異なることを注記 [J3][j3] |
| C08 システム全体の耐量子性を競争優位として確定できない | **維持** | 変更なし。Canton側のPQC状況は本調査で未確認であり、比較表には「未確認」と表示する（「非対応」としない） |
| C09 Cantonは優先競合であり、実装基盤・接続先候補でもある | **維持・条件追加** | 接続先評価の前提条件として、評価用ネットワークの利用条件（sponsorship）、traffic費用、Token Standardへの対応、zkPI検証をDaml内で行うかoff-ledgerで行うかの保証差を追加する [CN20][cn20] [CN23][cn23] [CN26][cn26] |

新しい判断IDは追加しない。C10相当の内容（「予約機能の有無ではなく予約の可視性で比較する」）はC02とC05の根拠追加で足りる。

## 5. 戦略への具体的変更

### 5.1 S01〜S06

| ID | 現行v1.1の要旨 | 変更 | 根拠 |
| --- | --- | --- | --- |
| S01 | 初期用途を、法人のデジタル証券二次取引で「順番が確定するまで注文を運営者に開かない」要求がある案件に絞る | **維持、定義を精密化。** 「運営者」を「自社validator、app運営者、照合エンジンを含む計算主体」と定義し直す。国内のJGBレポ・担保管理はCanton上のPoCが3件並走しているため、同じ業務での正面代替提案を初期用途にしない。代わりに、その業務の中で計算主体からも隠す要件がある部分（秘密の注文・レート提示・担保配分）に絞る | [CN5][cn5] [CN17][cn17] [J3][j3] [J6][j6] [J7][j7] |
| S02 | 最初の販売単位を秘密取引・事前予約・zkPI検証の導入モジュールとする | **維持、adapter契約を具体化。** 受け手候補に「Canton上のDaml application」を明記。adapter契約に、Token Standardのallocation（期限までのlock、settlement executorの一transaction、Splice参照interfaceのsender withdraw / cancel）との対応を追加。zkPI検証を「Daml contract内で検証」か「off-ledger verifierの署名を受け入れる」かで保証が違うことを契約項目にする | [CN26][cn26] [CN34][cn34] [CN3][cn3] |
| S03 | 稼働済みの自前MPC・DeFMI経路を基準に、継続取引と障害時整合性を先に完成させる | **維持、証拠状況を更新。** 継続取引2回は達成（smoke_only）。次の対象は取消・期限切れ・同時更新・停止復旧、その後に独立運営。Aethel依存の除去は基盤側の作業として並走するが、S03の完了条件には「Aethelなしの全経路receipt」を含める（戦略4節の既存記述と同じ） | [L16][l16] [L7][l7] |
| S04 | CantonとRenegadeを市場・決済の優先比較、Arcium/Zamaを秘密計算基盤、Owneraを接続設計の比較に使う。Cantonは実装先・接続先候補にも置く | **維持、比較単位を固定。** Cantonとの比較は「Daml app + Token Standard + synchronizer + validator運営」の単位で行い、「Canton対DeFMI L1」の基盤単位では行わない。比較項目に traffic費用、validator参加条件、Canton Foundation governance、割当てられたsynchronizerの選択（Global / private）を追加。Cantonの秘匿計算に対する設計上の立場は確認済み事実として記載する | [CN6][cn6] [CN20][cn20] [CN23][cn23] [CN17][cn17] |
| S05 | 公開core・検証仕様と導入・運用支援を組み合わせる | **維持、仕様の書き方を追加。** 公開する検証仕様（zkPIの命題、wire、verifier）を、Canton上のDaml interfaceまたはoff-ledger verifierとしても実装できる形式で書く。Cantonのopen governance（Foundation、Splice OSS、CIP）は公開方針の比較対象にする | [CN26][cn26] [CN24][cn24] |
| S06 | PQCは各境界の移行能力として整備する | **維持。** Canton側のPQC状況は未確認のため、比較表に「未確認」と表示し、優位も劣位も主張しない | — |

### 5.2 G0〜G4

| ゲート | 変更 | 根拠 |
| --- | --- | --- |
| G0 顧客課題 | 判定質問を3つ追加する。(a) 自社validator・app運営者・照合エンジンへの注文開示を許容するか（許容するならCanton型で足りる可能性が高い）。(b) 既にCanton上のPoC・WGに参加しているか、参加予定か。(c) 現金脚はトークン化預金・ステーブルコイン（Canton上）か、DeFMI cash railか。通過条件に「少なくとも1組織が、計算主体からも隠す要件を文書で示す」を加える。示されなければS01を見直す | [CN5][cn5] [J3][j3] [J6][j6] |
| G1 継続する全経路 | 目標は変更しない。証拠状況を「2回の全経路完了はsmoke_onlyで達成、次は取消・期限切れ・同時更新・停止復旧」に更新する。基準ファイルの再取得（新しいsnapshot）を親タスクに委ねる | [L16][l16] [L7][l7] |
| G2 一つの接続先 | Cantonを名指しの候補にし、受入経路を固定する。(1) 評価用ネットワーク（DevNet / TestNet）の利用条件・sponsor・費用の確認。(2) 対象資産をToken Standard準拠のDaml assetとし、予約をallocationに対応付ける。(3) zkPIの検証位置（Daml内 / off-ledger + 署名）を決め、保証差を記録。(4) 同一synchronizer上の一transactionでDvP。(5) Ledger APIによる確定readback。(6) 期限切れ時のallocation解放、senderのwithdraw・executorのcancel、再送の挙動。(7) validator・app・registryが読めた情報の一覧を記録。mockやCanton上の他社事例は接続証拠にしない（既存記述を維持） | [CN20][cn20] [CN26][cn26] [CN6][cn6] |
| G3 限定採用の条件 | Canton経路を採る場合、「validator運営者は自社の全データを読む」ことを独立運営条件の記録項目に含める。SV・Canton Foundation・trafficへの依存を運用条件（費用、governance変更、upgrade）として受入れ項目に加える | [CN5][cn5] [CN10][cn10] [CN23][cn23] |
| G4 拡大判断 | 変更なし。Canton側の自己申告値（処理量、SV数、参加社数）を採用判断の根拠にするときは、出典・時点・「自己申告」を付ける | [CN12][cn12] [CN28][cn28] |

### 5.3 Aethel方針の反映

- **比較上の位置付け:** Cantonは「protocol / synchronizer / validator / application」の層構造を持ち、業務アプリ（Broadridge DLR、Tradeweb、Cantex等）は基盤の上に載る。ZKFMIも「基盤（DeFMI / OCLOB / QOMM / zkPI / DeKYX / DeCCP）」と「業務アプリ（Aethel等）」を分ける方針を採ることで、比較表の「役割」列でCantonと同じ層で比較できる。Aethelは18対象の比較には入れない（競合ではなく利用者側）。
- **実装状況:** 12:07 UTC時点で defmi（`rust/qomm-avalanche-vm/src/execution/aethel*.rs` の削除）、dekyx（`crates/dekyx-aethel` の削除）、deccp（`crates/deccp-aethel` の削除）に未コミット差分がある。defmi READMEの「Depends on: aethel」は基準時点の記述のまま。**分離完了とは扱わない。** 完了の証拠は、対象全リポジトリの依存・起動・通信経路の確認とAethelなしの全経路receiptである（戦略4節の既存条件）。
- **文書上の注意:** 統合後の比較文書で「ZKFMIはAethelに依存しない」と書けるのは、上記receiptが得られた後である。それまでは「依存をなくす方針」と書く。

## 6. 先行調査の修正点（根拠URL付き）

致命的な誤りはない。以下は名称・出典・数え方の精密化と、不足していた根拠の追加である。

| # | 対象箇所 | 現行の記述 | 修正 | 根拠 |
| --- | --- | --- | --- | --- |
| 1 | Canton追加調査1節・9節（CN8）、比較判断7節 | 「Global Synchronizer Foundation」 | 「Canton Foundation（旧Global Synchronizer Foundation、2025-09-22に改名。名称変更のみ）」。Canton公式docsの複数synchronizerページも「under the governance of the Canton Foundation」と記載。MUFG発表も「Canton Foundation によって運営」と記載 | https://canton.foundation/the-gsf-is-now-canton-foundation/ 、 https://docs.canton.network/overview/learn/multi-synchronizer 、 https://www.mufg.jp/dam/pressrelease/2026/pdf/news-20260813-002_ja.pdf |
| 2 | Canton追加調査1節（Canton Networkの行） | 「network of networks」の出典をCN1（Architecture Overview）に置く | 取得したCN1本文にこの語句はなかった。出典はCanton技術primer（2025-04-01）またはSplice docsのValidators節に変更する | https://www.canton.network/blog/a-technical-primer 、 https://docs.sync.global/overview/overview.html |
| 3 | Canton追加調査5節（Pilotの行） | 「全参加45社」 | 発表本文の役割別内訳は15+13+4+3+1=36社。「45」はページの要約文にある数。「参加45社（ページ要約）、本文の役割別合計36社」と併記する | https://www.canton.network/canton-network-press-releases/the-canton-network-completes-the-most-comprehensive-blockchain-pilot-to-date-for-tokenized-real-world-assets |
| 4 | Canton追加調査5節（Broadridgeの行） | 2025-09-10発表を処理量の根拠にする | 記述は正しい。ただし同発表のCanton言及は「市場データ配信アプリをCanton上で提供」であり、DLR本体のCanton移行（2023）とcash off-chainの構成はDA顧客事例（2024-06-21）が出典であることを注記する。$280bnは「2025年8月の平均日次repo取引」、月間$5.9Tの自己申告 | https://www.broadridge.com/press-release/2025/billions-in-average-daily-processed-trade-volumes-on-broadridge-dlt-repo-platform 、 https://blog.digitalasset.com/blog/customer-story-broadridge |
| 5 | Canton追加調査9節（CN14） | investors.tradeweb.com のURL | 本調査ではtimeoutと403で取得できなかった。同文を掲載するCanton公式の転載URLを併記する。内容（Franklin Templeton→Virtu、tokenized UST対USDCx、synchronized on-chain settlement、2026-07-01）は転載で確認 | https://www.canton.network/canton-network-press-releases/tradeweb-on-chain-us-treasuries-canton |
| 6 | Canton追加調査2節・6節（MPCの行） | 標準経路が平文検証であることから差を推論 | Canton whitepaper（2020-02-04）とCanton公式blog（2025-05-12）が、MPC/ZKを採らない設計上の理由を明記している。「記載の不在」ではなく「設計上の選択」として引用する | https://www.canton.io/publications/canton-whitepaper.pdf 、 https://www.canton.network/blog/zero-knowledge-proofs-whe-privacy-needs-more |
| 7 | Canton追加調査3節・4節・6節、比較判断C05 | 予約・DvPをDaml contractで「実装可能」と記述 | Token Standard（CIP-0056、2025-03-31承認）がallocationによる予約型決済を標準化済み。「実装可能」ではなく「標準化済み、ただし平文」と書く | https://raw.githubusercontent.com/canton-foundation/cips/main/cip-0056/cip-0056.md |
| 8 | Canton追加調査5節、戦略2節の顧客仮説 | 国内事例はMUFG/Progmatのみ | JSCC・みずほ・野村・DAのJGB担保PoC（2026-04-20、FSA PIP選定）、Progmat/DCCのWG（2026-05開始、報告書2026-10目標）、2024-07-01 go-live参加者にSBI Digital Asset Holdingsが含まれることを追加する | https://blog.digitalasset.com/press-release/launch-of-proof-of-concept-trial-for-digital-collateral-management-using-japanese-government-bonds-jgbs 、 https://prtimes.jp/main/html/rd/p/000000006.000172212.html 、 https://www.canton.network/canton-network-press-releases/the-canton-networks-global-synchronizer-and-canton-coin-go-live |
| 9 | Canton追加調査1節（Global Synchronizerの行） | 「2/3多数のBFT」 | 正しい。SVがCometBFT ordererを運用し、ブロック生成に「SVノードの2/3超の合意」、許容故障 `f = floor((n-1)/3)` を追記する。trafficはCanton Coinのburnで支払う点、validator参加にsponsorshipが必要な点を運用条件として追加する | https://docs.canton.network/overview/reference/super-validator-components 、 https://docs.canton.network/global-synchronizer/understand/overview 、 https://www.canton.network/faq |
| 10 | 比較判断4節（継続する取引の行）、戦略5節（OCLOBの現在位置） | 「基準時刻では未達。rough-001はrejected」 | 基準時刻（10:55:32 UTC）の記述としては正しい。その後の実行005（11:18:37Z）・006（11:29:09Z）で2回の全経路完了をsmoke_onlyで記録している。統合時は「基準時刻後の追加証拠」として別行で追加し、基準ファイルは改変せず新snapshotを取る | 7節（ローカル） |
| 11 | 基準ファイル | OCLOB HEAD `ac93685…`、README・ledgerのSHA-256 | 12:07 UTC時点のOCLOB HEADは `f8010fe16269b0a1821b27247ee43304fff8a89b`。README SHA-256は `31e7c6f7…dac34`、ledger SHA-256は `7a6f6586…28c7` に変わっている。基準ファイル自体は正しい（当時の記録）。再基準化が必要 | 7節（ローカル） |

## 7. ローカル実装証拠の更新（基準時刻以後）

| 項目 | 基準時刻（10:55:32 UTC）の記録 | 12:07 UTC時点の確認 | 扱い |
| --- | --- | --- | --- |
| 継続取引契約 `oclob-native-cycle-v1`（SHA-256 `b79ab992…d9a` は不変） | 最後の記録はrough-001 / rejected | ledger 28行目 repair-005（11:18:37Z）と29行目 final-006（11:29:09Z）が `smoke_only`。2回の照合完了、3約定、8件の受取権取込み、両法人の保証枠更新番号5/5、14+7件のノード確認、5検証ノードのroot一致と再起動 [L7][l7] | **smoke evidenceに留まる。** 単一ホスト、2法人、設定済みDeFMI読取サービスへの信頼、UI/WAN/独立運営者なし、経済性能なし（ledgerのlimitations欄のとおり） |
| 成果物 | — | `oclob/artifacts/oclob_native_cycle.json` SHA-256 `dcaca99d40e63bae9dc3dfa79163be8fbd5914a0b787a27c055bcd2257420300`、`independent_operators=false`、`completed_native_rounds=2`、第2取引 `2sKQweqnWRjuGqZEdEPdUiH7Mp9sfVACH8PD1d7WFt1VzdXry7` [L16][l16] | 比較判断C06・戦略G1の証拠として追加可能。本番受入・独立運営の証拠にしない |
| 複数約定一括決済 | `oclob_native_multifill.json` SHA-256 `ed871ddd…b970`、smoke_only | 同一ハッシュを再確認 [L6][l6] | 変更なし |
| Aethel依存 | 方針のみ | defmi / dekyx / deccp に `*-aethel` 削除の未コミット差分 | 進行中。完了扱いにしない |

本書はこれらを再実行していない。読んだのはartifact・ledger・docsであり、実行環境（Softbank）のログは参照していない。

## 8. 検証不能・未確認事項

- **SV数:** Canton側の「45+ Super Validators」（2026-04-04）は二次転載でしか確認できず、Visa発表の「40」（2026-03-25）と時点が異なる。現在の正確な数は未確認。[CN28][cn28]
- **Canton FoundationとLinux Foundationの関係:** 改名（2025-09-22）後に関係が変わったとする二次情報があるが、一次資料で未確認。[CN24][cn24]
- **DTCCのMVP:** 2026年上半期に「controlled production environment」で予定。完了・商用開始は未確認。Tradewebの2026-07-01発表では「later this year」に予定と記載。[CN27][cn27] [CN29][cn29]
- **JSCC/みずほ/野村PoCの期間:** 終了時期は一次資料に記載なし（報道の「9月末ごろ」は未確認）。[J6][j6]
- **FSA PIPの2026年2月支援決定:** MUFG・DAの発表が記載するが、金融庁の該当ページは特定できなかった。金融庁の2026-02-27「FinTech実証実験ハブ」支援決定ページ（https://www.fsa.go.jp/news/r7/sonota/20260227-2/20260227-2.html ）は開封したが、掲載は日立製作所の1件のみで該当なし。PIP設置（2025-11-07）のみ金融庁ページで確認。[J8][j8]
- **Canton上の秘密CLOB:** CantexのCLOB、Hydra XのCanton上CLOB、照合主体が注文を読まない設計の公開実装は未確認。「無い」とは書かない。[CN30][cn30] [CN32][cn32]
- **Canton app層のMPC/ZK連携:** 公開実装は未確認。protocolに将来追加されない保証もない。
- **CantonのPQC状況:** 未確認。
- **Broadridge DLRの構成:** どのsynchronizerを使うか、Global Synchronizerとの接続、on-chain cashの有無は未確認。[CN13][cn13]
- **性能・費用:** CantonとZKFMIの遅延・処理能力・費用・復旧時間を同条件で測っていない。Canton Coinによるtraffic費用の額も未確認。**優位・劣位を主張しない。**
- **法的受渡し:** 国内PoCは振替法上のJGBの法的地位を維持することを目的に掲げているが、結果は未公表。ZKFMI側の法的finalityも未受入。
- **Canton pilot PDF:** 先行調査と同様に読了していない（発表文で代替）。
- **Cantonの「唯一のprivacyを持つpublic chain」等の自己表現:** 検証していない。比較の根拠にしない。
- **顧客需要:** 自社validatorへの開示で足りるのか、計算主体からも隠す必要があるのかは、G0で確認するまで未確認。

## 9. 一次資料

既存ID（CN1〜CN14、J3）は[Canton追加調査][canton]と同じURL。CN15〜CN34とJ6〜J8は本書で追加した。全件の確認日は2026-09-05（時刻は1節）。

| ID | 一次資料 | 主に確認したこと | 取得手段 |
| --- | --- | --- | --- |
| CN1 | [Canton Docs: Architecture Overview][cn1] | participantは自分のpartyが利害関係者のcontractだけを保存。sequencer / mediatorの役割。「network of networks」の語句は本文になし | Web取得 |
| CN2 | [Canton Docs: Privacy Model][cn2] | view分解、非関係partyはpayloadもmetadataも見ない、divulgence、Timing Attacks（時刻・サイズ・パターン）。ZK/MPC/FHEの記述なし | Web取得 |
| CN3 | [Canton Docs: Smart Contract Consensus][cn3] | 「Proof of Stakeholder」、再実行・authorization・ACS検証、mediatorはDaml logicを独自検証しない、confirmation policy | Web取得 |
| CN4 | [Canton Docs: Transaction Lifecycle][cn4] | view単位のsession key暗号化、sequencerは暗号化viewを一定期間保持するが復号できない | Web取得 |
| CN5 | [Canton Docs: Trust Model][cn5] | 「Your validator sees all your data and could block you」、external party key、counterparty・app provider・synchronizer・governanceへの信頼 | Web取得 |
| CN6 | [Canton Docs: Multi-Synchronizer][cn6] | 一transactionは単一synchronizer、reassignmentは非原子的、Global SynchronizerはCanton Foundationのgovernance下 | Web取得 |
| CN7 | [Canton Docs: Cross-Synchronizer DvP Example][cn7] | 共通Settlement Syncへ再assign後に一transactionで交換、pending境界 | Web取得 |
| CN8 | [Splice Docs: Global Synchronizer Overview][cn8] | 2/3 BFT、SVの役割、Canton Coin、GSF、「network of networks」 | Web取得 |
| CN9 | [Global Synchronizer / Canton Coin go-live、2024-07-01][cn9] | go-live、参加31組織（SBI Digital Asset Holdings等）、Linux FoundationによるGSF支援 | Web取得 |
| CN10 | [Logical Synchronizer Upgrades、2026-06-29][cn10] | Canton 3.5、LSU Mainnet稼働、protocol更新4回 | Web取得 |
| CN11 | [Canton Network Pilot完了、2024-03-12][cn11] | TestNet、22 dApp、350超のsimulated transaction、役割別36社（要約は45社） | Web取得 |
| CN12 | [Broadridge、2025-09-10][cn12] | 2025年8月の平均日次$280bn・月間$5.9T（自己申告）、Canton上の市場データ配信アプリ | Web取得 |
| CN13 | [Digital Asset: Broadridge customer story、2024-06-21][cn13] | 2023年にCantonへ移行、cash off-chain、証券所有権をsmart contractで移転 | Web取得 |
| CN14 | [Tradeweb（investors.tradeweb.com）、2026-07-01][cn14] | 本調査ではtimeout。CN29で代替 | 取得不能 |
| CN15 | [Daml Ledger Model: Privacy][cn15] | 「need-to-know basis」「subtransaction単位」、informee / witness / projection / divulgenceの定義 | curlで本文抽出 |
| CN16 | [Canton Architecture: Overview and Assumptions][cn16] | sequencerの「二つの機能だけ」、mediatorは内容を学ばずinformee一覧を受け取る、参加者の身元を互いに隠す | curlで本文抽出 |
| CN17 | [Canton whitepaper、2020-02-04][cn17] | MPCを含む高度な暗号は計算負荷が大きく、より強い信頼仮定で可視性を限定する設計方針 | PDFローカル抽出 |
| CN18 | [Canton blog: When Privacy Needs Proof、2025-05-12][cn18] | ZKPは汎用privacyには実験段階、Cantonは監査可能なprivacyを提供、という立場 | Web取得 |
| CN19 | [Canton blog: Institutional-Grade Privacy、2025-08-14][cn19] | データ分離と暗号化、運営者は限られたmetadataのみ。ZK/MPC/FHE/TEEの記述なし | Web取得 |
| CN20 | [Canton FAQ][cn20] | sub-transaction privacy、運営者は限られたmetadata、validator申請にsponsorship、atomic composition | Web取得 |
| CN21 | [Canton技術primer、2025-04-01][cn21] | 「network of networks」、validatorは関係データのみ、synchronizerは封書を開けない郵便局の比喩 | Web取得 |
| CN22 | [Canton Docs: Super Validator Components][cn22] | sequencer・mediator・CometBFT orderer・scan・governance app、2/3超、`f = floor((n-1)/3)` | Web取得 |
| CN23 | [Canton Docs: Global Synchronizer Overview][cn23] | Canton Coinのburnによるtraffic支払い、Canton FoundationがSVノードを運用 | Web取得 |
| CN24 | [Canton Foundation: GSFからの改名、2025-09-22][cn24] | 名称変更のみ、Global Synchronizerのgovernance | Web取得 |
| CN25 | [Linux Foundation、2025-03-19][cn25] | GSFに30超の参加、Goldman Sachs・HKFMI・Moody's参加、LFの支援 | Web取得 |
| CN26 | [CIP-0056 Canton Network Token Standard][cn26] | Final、2025-03-31承認。allocationのlockと期限、settlement appの一transactionで全部か無か。withdraw / cancel / revokeの定義は本文になし（改訂時にcurlで全文を再確認） | Web取得（raw）+ curl |
| CN27 | [DTCC / Digital Asset、2025-12-17][cn27] | SEC No-Action Letter、DTC保管米国債の一部をCanton上でmint、MVPは2026年上半期予定 | Web取得 |
| CN28 | [Visa、2026-03-25][cn28] | SVとして参加、「40 Super Validatorsの一つ」 | Web取得 |
| CN29 | [Tradeweb発表のCanton公式転載、2026-07-01][cn29] | Franklin Templeton→Virtu、tokenized UST対USDCx、synchronized on-chain settlement、DTCC Tokenization Servicesは予定 | Web取得 |
| CN30 | [Cantex公式サイト][cn30] | 「Canton-native AMM」、「swapをoffchainで提出」、CaviarNineが照合エンジンを提供、Cantonで原子的決済。docs.cantex.io はトップページだけを開封し、「How Cantex Works」ページは未開封 | Web取得 |
| CN31 | [Canton Foundation: About][cn31] | Global Synchronizerの開発・governance、SV運営、Digital AssetがPremier Member | Web取得 |
| CN32 | [Hydra X、2025-04-17][cn32] | Canton上の仕組債トークン。CLOBの記述なし | Web取得 |
| CN33 | [Canton Docs: Synchronizer][cn33] | 暗号化envelopeとmetadataのみ、payloadを復号できない、SVによるBFT | Web取得 |
| CN34 | [Splice: `Splice.Api.Token.AllocationV1` Daml interface][cn34] | `Allocation_Withdraw`（controller: sender。`allocateBefore` 前なら決済を失敗させないSHOULD）、`Allocation_Cancel`（sender・receiver・executorの共同制御、通常executorへ委任）、`Allocation_ExecuteTransfer`（`settleBefore` 前）。取得はmain branch、その時点のmain先頭commit `fe492f45a8ba7073120c081b583fa13bd4254be6`（2026-09-04T19:28:43Z） | curl（改訂時 12:30 UTC） |
| J3 | [MUFG: JGBレポ実証開始、2026-08-13][j3] | 4社+DA+Progmat+Secured Finance、振替国債の法的性質維持、トークン化預金・ステーブルコイン、FSA PIP、Canton Foundation | PDFローカル抽出 |
| J6 | [JSCC・みずほ・野村・DA: JGB担保PoC、2026-04-20（DA公式転載）][j6] | 振替法・金商法上の地位維持、既存システムとCantonの接続、24/365・クロスボーダー、FSA PIP選定 | Web取得（jpx.co.jp・野村は403） |
| J7 | [Secured Finance: Progmat/DCC WG参画、2026-05-08][j7] | WGの開始月、検討観点、報告書2026-10目標 | Web取得 |
| J8 | [金融庁: 決済高度化プロジェクト（PIP）の設置、2025-11-07][j8] | 「本日（11月７日）、FinTech実証実験ハブ内に、決済分野に特化した「決済高度化プロジェクト」（PIP）を立ち上げました」。2026年2月の支援決定ページは未特定 | curl（改訂時 12:30 UTC に開封） |
| L1 | defmi README（基準ファイルのL1と同一） | ReservationAdmission / ReservationPermitの分離 | ローカル |
| L4 | oclob README | 法人側分割、7ノード、調整役へ原文を渡さない、限界 | ローカル（基準時刻後に更新あり） |
| L6 | oclob/artifacts/oclob_native_multifill.json | SHA-256 `ed871ddd…b970`、smoke_only | ローカル |
| L7 | oclob/research/experiment-ledger.jsonl | 28・29行目の実行005・006 | ローカル（基準時刻後に更新あり） |
| L10 | qomm README | RFQ方式 | ローカル |
| L13 | zkpi README | 命題と検証範囲 | ローカル |
| L16 | oclob/artifacts/oclob_native_cycle.json | SHA-256 `dcaca99d…0300`、`completed_native_rounds=2`、`independent_operators=false` | ローカル（基準時刻後に生成） |

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
