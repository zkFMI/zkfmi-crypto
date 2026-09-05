# Cantonを含む18対象との比較判断 v1.2

確定日: **2026-09-05**。対象: ZKFMI全体と、[初回サーベイ][survey]の17対象に[Canton追加調査][canton]を加えた18対象。ローカル実装の基準時刻は **2026-09-05 10:55:32 UTC**。[基準ファイル][baseline]に6リポジトリのHEAD、15資料のSHA-256、未コミット変更の存在を記録した。v1.2は[Claude Fable 5.1 Maxレビュー][fable]を統合した。後続のローカル証拠は[13:14:06 UTCの追加snapshot][update]へ分け、元の基準ファイルを改変していない。

**確定する結論は、ZKFMIの比較上の位置と採用戦略である。機能の独占性、性能優位、顧客需要、本番安全性の立証ではない。** 競合の資料に見当たらない機能を「非対応」とせず、商用発表・仕様・自分たちの実行記録を分けた。今後の選択は[戦略書][strategy]へ結び付ける。

## 1. 確定した判断

| ID | 比較判断 | 根拠と適用範囲 | 戦略への反映 |
| --- | --- | --- | --- |
| C01 | MPC、ZK、約定計算の正しさの証明は独自性の根拠にならない | RenegadeはVALID MATCH MPCで入力注文・残高の有効性、照合の正しさ、出力暗号化を扱う。Cantonの一般architectureでも、Damlで業務規則・権限・状態を表し、関係participantが提出transactionのDaml実行・権限・状態を検証する。ZKFMIと同じ市場appは未確認である。金融MPCにはPrime Matchの先行報告もある [R4][r4] [P1][p1] [Canton 3節][canton] | 「市場規則を証明できるのは自分たちだけ」と訴求しない |
| C02 | 有望な比較軸は、どの秘密を誰から隠すかと、どの市場規則・状態を同じ取引に結び付けるか | Renegadeの委託relayerは担当walletを読む。Cantonは非関係partyとsynchronizerからviewを隠すが、host validatorと権利を持つpartyは該当dataを読む。OCLOBの新経路は法人側で分割するが、3ノード以上の結託や通信観測は別の限界 [R1][r1] [L4][l4] [Canton 2節][canton] | 顧客が許す開示範囲を先に決め、計算者にも隠す必要がある案件に絞る |
| C03 | 価格形成方式の違いは確認できるが、普遍的な優劣ではない | Renegadeの公開説明は外部価格のmidpoint crossing。QOMMは秘密の価格方針を評価するRFQ、OCLOBは確定した受付順に沿う価格・時間優先照合 [R5][r5] [L10][l10] [L4][l4] | RFQ、連続板、midpoint crossingを同条件の「速度順位」に混ぜない |
| C04 | 秘密分散の安全性でZKFMIが一律に強いとは言えない | Arcium Cerberusは少なくとも1者が正直なら秘匿し、異常時はabort。OCLOBは7ノード中最大2不正を想定。Zama KMSはt < n/3で鍵生成・復号の完了を扱う [A2][a2] [Z3][z3] [L4][l4] | 秘匿性、正しさ、処理完了、運営主体の独立性を別項目で示す |
| C05 | 指図の標準化、DvP、台帳間調整にも先行基盤がある | Owneraはintent、asset hold、台帳能力に応じたorchestrationを説明。Cordaは契約状態と一意性を検証する。CantonはToken Standard allocationによる予約と、共通synchronizer上の一Daml transactionによるcross-app DvPを標準化する [O1][o1] [C2][c2] [Canton 4節][canton] | zkPIは既存の権限・台帳契約に追加できる検証内容で提案する。原子性そのものを独自性にしない |
| C06 | ZKFMIは一部の実機能を示した研究MVPであり、機関向け完成品との成熟度差がある | 初回の2約定smokeに加え、後続snapshotは2回の実約定、取消・期限切れ、7MPCノードの再起動までsmoke_onlyを確認。独立運営・WANはfalse [更新snapshot][update]。CantonはMainnet、商用DLR、TestNet pilot、単発の実取引、開始済みPoCを区別しても、ZKFMIより広い運用実績を持つ [L6][l6] [Canton 5節][canton] | 次の重点は継続利用・障害時整合性・独立した運用の検証。競合の成熟度を過小評価しない |
| C07 | 日本では既存証券業務と現金脚への接続が採用条件になり得る | Progmat、ibet for Fin、Kinexys、Fnality等は対象業務の採用・運営実績を公表し、MUFG/ProgmatはCantonでのJGB repo実証協業を開始した。ZKFMIとの接続、Canton実証の完了、法的受渡しは未確認 [J1][j1] [B2][b2] [K1][k1] [F1][f1] [Canton 5節][canton] | 接続型の採用実証を第一候補に置く。契約/APIの利用権と法的役割は別途確認 |
| C08 | 現時点でシステム全体の耐量子性を競争優位として確定できない | zkfmi-cryptoの独立P0と、既存のcommitment・証明・署名・通信・保存状態は別の移行対象 [P0][p0] | PQCは境界ごとの移行計画として提供し、全体対応済みと書かない |
| C09 | Cantonは優先競合であると同時に、追加機能の実装基盤・接続先候補でもある | 公式architectureは業務ロジック・権限・privacy ruleをDamlに置き、Global / private synchronizerを選べる構成を示す。Canton上へ外部MPC結果やzkPI verifierを結ぶ案は設計上の推論であり、同等実装、同じ秘密条件・性能・費用・法的受渡しは未確認 [Canton 6〜8節][canton] | 独立DeFMI L1、QOMM、OCLOB、DeCCPの目標を維持し、G2でCanton接続・同業務の代替構成を評価する |

「市場規則の証明」は結局、選んだ関数と入力に対する正しさの証明である。汎用MPC/ZK基盤でも同種の関数を実装し得る。ZKFMIの比較上の価値は、規則、入力受付、資格、予約、決済、復旧をつなぐ実装と運用証拠に置く。既存POSITION文書の「他は回路、こちらは市場」という短い表現だけを新規性の証明に使わない。

## 2. 18対象の扱いを確定

「優先比較」は開発・提案の意思決定に直結する対象。「接続候補」は連携済みや提携先を意味しない。「監視」は無視する意味ではなく、現段階で個別導入を進めない位置付けである。以下の優先順位は本調査の判断であり、市場シェア順位ではない。

| 対象 | 公開資料から確認できる役割 | 秘密・検証の重要な境界 | 成熟度の扱い | ZKFMIでの扱い |
| --- | --- | --- | --- | --- |
| **Canton Network** | Damlアプリ、partyをhostするvalidator、順序・確認を調整するsynchronizer、cross-app atomic transaction | 非関係partyとsynchronizerはpayloadを読まない。host validatorと関係partyは該当viewを読む。Daml検証と、MPCで計算者にも入力を隠す保証は別 | Global Synchronizer Mainnet、商用DLR、TestNet pilot、単発実取引、開始済みPoCを分離 | **最優先比較・基盤/接続候補**: 秘密境界、入力集合、市場規則、予約、DvP、運用責任を同じ業務で比較 [Canton追加調査][canton] |
| **Renegade** | MPCと共同SNARKによるmidpoint crossing | 委託relayerは担当注文を読める。正しい照合の証明がある。全市場の受付集合・順序保証の同等性は未確認 | mainnet開始公表 | **優先比較**: OCLOBの受付順・市場方式、法人からの分割との違い [R1][r1] [R4][r4] [R5][r5] |
| **Arcium** | 汎用MPCアプリ基盤 | dishonest-majority / detect-and-abort。金融業務の意味はアプリが定める | Mainnet Alpha公表 | **優先比較・基盤候補**: 同じ業務を構築する費用と信頼条件 [A2][a2] [サーベイ][survey] |
| **Zama** | FHEによる秘密状態演算・分散鍵管理 | 復号権限、KMS、演算結果の検証を分ける。入力ZKPoKと業務全体の証明も別 | mainnet・秘密入札の実施公表 | **基盤比較**: Arciumとともに自前実装の妥当性を判断 [Z2][z2] [Z3][z3] [Z4][z4] |
| **R3 Corda** | 機関間の契約・資産状態管理 | 共有範囲を限定。notaryと取引参加者の開示は異なる | 証券決済向け採用公表 | **優先比較・接続候補**: 台帳採用の理由を上回る追加価値があるか [C1][c1] [C2][c2] |
| **Kinexys** | 銀行決済・資産トークン化 | 銀行運営・預金・顧客関係。銀行から入力を隠す保証とは別 | 商用業務・取引事例公表 | **機関向け比較・現金脚候補** [K1][k1] |
| **Fnality** | 機関向け現金決済 | 決済制度、裏付け資金、参加者・運用条件 | 監督資料に限度付き運用の記載 | **現金脚候補**: 独自現金基盤の導入負担を比較 [F1][f1] |
| **Partior** | 国際支払・FX PvP | 参加銀行と支払経路。秘密注文市場の仕様は未確認 | 支払稼働事例とDvP PoCを区別 | **現金脚・PvP候補** [T1][t1] |
| **Ownera / FinP2P** | 取引intentと複数台帳の実行調整 | 署名・receipt・合意、元台帳のhold/atomic能力に依存 | 具体的なAPI・仕様を確認 | **優先比較・接続設計の参照** [O1][o1] |
| **Swift共有台帳** | 銀行間支払の調整 | 共有層と銀行の資産・資金管理を分離。既存決済経路を使用 | 2026-07発表はlive pilot準備 | **機関接続の監視**。全機能稼働と扱わない [S1][s1] |
| **Chainlink** | 外部情報・ポリシー・クロスチェーン連携 | メッセージ、TEE、署名等の保証と業務証明を区別 | 機能ごとの仕様・事例 | **接続候補・zkPIの比較相手** [LNK1][lnk1] |
| **Progmat** | 国内ST発行・管理 | 発行・信託・販売の役割と台帳を分ける | ST事例、Avalanche移行完了の提供者発表と、Canton JGB repo実証の開始を分離 | **国内の優先比較・接続候補**。Canton実証を既存STの本番機能へ一般化しない [J1][j1] [J5][j5] [Canton 5節][canton] |
| **BOOSTRY / ibet for Fin** | 国内STとコンソーシアム運営 | 標準契約、参加組織、発行・流通の実務 | 運営開始の参加者発表 | **国内の優先比較・接続候補** [B2][b2] |
| **Prime Match** | 金融機関と顧客の秘密在庫照合 | 著者が定義した銀行・顧客のMPCモデル | 2023年のlive運用報告。現在の稼働は未確認 | **先行研究比較**。金融MPC初という主張を棄却 [P1][p1] |
| **Penumbra** | shielded poolとbatch DEX | 現行swap入力のasset/amountは公開、claim等の秘匿と区別 | 仕様確認。sealed-bid版は将来拡張 | **市場設計の監視** [N1][n1] |
| **Dusk** | 規制資産向け台帳・選択開示 | 公開account modelもあり、アプリ別に確認が必要 | mainnet接続仕様・協業公表 | **証券業務の監視**。協業相手の免許を全アプリへ一般化しない [D1][d1] [D2][d2] |
| **Aztec** | private/publicアプリ基盤 | 端末側証明と多者の秘密入力計算は別。版の状態に注意 | Alpha V5の脆弱性告知、V6修正完了は今回未確認 | **基盤の監視** [X2][x2] |
| **Hyperledger Fabric** | 許可組織間の台帳とprivate data | 許可peerは実データを読む。その他へhashを共有 | 仕様確認 | **内製の代替案**: 閲覧組織の限定で十分なら比較対象 [H1][h1] |

## 3. 近接比較の決着点

### 3.1 Cantonとの違い

Cantonを「機関向けだが秘密計算や原子的決済はない台帳」と扱う比較を棄却する。公式docsは、Daml transactionをviewへ分け、関係participantだけが復号・再実行・権限・Active Contract Setを検証し、synchronizerが暗号化messageの順序とcommit / abortを調整する構成を説明する。共通synchronizer上では、複数アプリ・participantにまたがる一transactionのDvPを原子的に実行できる。[Canton 1〜4節][canton]

確認できる差は秘密の相手である。Cantonの標準経路では、host validatorはpartyのdataを持ち、取引に関係するvalidatorは自分のviewを平文で検証する。OCLOBの法人側share生成経路は、許容結託数以下の各MPC nodeへ完全入力を渡さない。Canton上で外部MPCの結果をDamlへ渡す構成や、資格・与信・予約・matching ruleをDamlへ表す構成は設計候補だと推論できるが、同等appの実装は確認していない。この可能性があるため、ZKFMIの排他的な新規性とは扱わない。

CantonのProof of Stakeholderは、提出されたDaml transactionと関係contractの正しさを当事者が検証する。市場全体から注文を省略しなかったか、submit前の受付順や検閲、外部与信・保管原帳との一致は個別applicationの境界である。ZKFMIも受付前検閲を自動で解消しない。**同じ受付集合と外部脚を定めた比較なしに、どちらか一方だけが市場を証明すると言わない。**

CantonはZKFMIの競合であると同時に、Daml asset / cash contractへMPC order processing、zkPI verifier、資格・予約adapterを加える実装先、またはsettlement接続先になり得る。顧客需要、同条件の性能・費用、API利用権、法的許認可は未確認であるため、現時点でDeFMIを廃止・置換しない。

具体的な比較単位は **Daml app + Token Standard + synchronizer + validator運営** とする。予約やDvPの有無を独自性にしない。withdraw・共同承認cancelの条件、zkPIをDaml内で検証する場合とoff-ledger verifierの署名を受け入れる場合の保証差、参加sponsor・traffic費用・Ledger APIの確定readbackまでG2で比較する。CIP本文とSplice interfaceを区別し、対象registryの実装は実APIで確認する。[Canton 4節][canton] [Fable 5節][fable]

### 3.2 Renegadeとの違い

Renegadeが「署名だけで照合結果を信用する方式」という比較は棄却する。公式リポジトリは、正しい照合と有効な入力を対象とする共同証明を明記している。[R4][r4]

確認できる違いは、公開説明のmidpoint価格、担当relayerの閲覧範囲、ZKFMI側のRFQ/連続板という業務選択である。[R5][r5] ただし、自前relayerを運用する顧客は外部委託先への平文開示を避けられる。ZKFMIの「運営者に見せない」という訴求は、自前relayer案も含めた運用負担と結託条件の比較が必要になる。

OCLOBの受付順証明も、ネットワーク全体で最初に送信された時刻や、受付前の検閲がないことまで保証しない。QOMMの最小価格も、指定された参加者・入力集合内での命題であり、市場全体や法令上の最良執行の達成とは別である。**比較対象の集合と受付境界を明記して初めて、差を主張する。**

### 3.3 Arcium / Zamaとの違い

MPC方式名、ノード数、FHEという名称を点数化しない。比較するのは、同じ入力・規則・出力・復号権限を実装した場合の信頼条件と運用費用である。Arciumのabort特性とZama KMSの鍵処理の完了条件は、別の対象に対する保証である。[A2][a2] [Z3][z3]

Zamaのcoprocessor説明は入力のZKPoK、FHE演算、commitment、署名を区別している。これを「業務結果全体が単一のZK証明で検証できる」と読み替えない。一方、ZKFMIにのみ外部検証があるとも断定しない。[Z4][z4]

自前MPCは現行の実行可能な基準として維持する。基盤変更の判断には同じ全経路の結果が必要であり、本調査ではどの基盤も導入・置換していない。

### 3.4 Ownera / Cordaとの違い

Owneraはintentの署名、複数機関の合意、receiptの確認と、台帳能力に応じたDvPを既に説明する。[O1][o1] Cordaも契約条件・状態遷移と二重消費防止を扱う。[C2][c2]

zkPIが追加する候補価値は、秘密の入力に対して評価した具体的な規則と資産予約を指図へ結び、受け手が定義された命題を検証できること。ただし、受け手が証明を検証せず署名やdigestだけを受け入れるadapterなら、保証はそのadapter/署名者への信頼まで下がる。DeFMI内の原子性を、任意の銀行システムをまたぐ原子性へ拡張して説明しない。

## 4. 自分たちの実装について確認した範囲

| 項目 | 今回の判断 | 証拠と制限 |
| --- | --- | --- |
| 注文からMPCへの情報分離 | 新CLI/Docker経路の設計・実装記述を確認 | 法人側分割、7ノード、調整役へ原文を渡さない。ブラウザ互換デモには平文を持つ経路が残る [L4][l4] |
| 複数約定の決済 | **既存の実行成果物を読んで確認** | 2約定、1取引、14ノード/約定確認、全5検証者のroot一致と再起動。成果物SHA-256は基準ファイルに固定。今回再実行はしていない [L5][l5] [L6][l6] |
| 継続する取引 | 基準後の追加証拠は **smoke_only** | cycle-final-006で2回・3約定。lifecycle-final-004で取消・実期限切れ・返却資産再利用・7MPCノード再起動・5台帳一致を記録。rough-001のrejectedは保持。本タスクで再実行・remote logの独自検査はしていない [更新snapshot][update] |
| 台帳確定の信頼 | 読取サービスへの信頼が残る | 各ノードの独自readbackはあるが、独立した合意証明の直接検証ではない [L4][l4] |
| 決済の秘密範囲 | 全フィールド秘匿ではない | native railのasset IDと決済メタデータは公開 [L1][l1] |
| 身元・資格 | scope内の仮名性 | DeKYXは資格の選択開示を行うが、発行者記録と完全にunlinkableではない。KYC/KYB認証済み製品ではない [L14][l14] |
| 清算 | 研究用の清算・リスク状態機械 | DeCCP単体は資産保管者や認可された清算機関ではない [L15][l15] |
| 暗号安全性 | 修正済みの欠陥と、未受入の本番保証を区別 | 旧note proofの欠陥を修正した記録あり。現行Triptych依存は実験用で、独立監査・旧状態移行は別条件 [L3][l3] |
| QOMMの経済効果 | 確認実験・外部検証を未通過として扱う | 既存契約はsmoke段階。合成データのsmokeを価格優位や顧客効果に昇格しない [L11][l11] |
| Aethel依存 | ソース・依存グラフの分離を実装 | アプリ固有4統合クレートをAethelへ移動。6基盤の全feature metadataでAethel 0件。Aethelをmountせず基盤469件、アプリ57件のテストを通過。デプロイ・旧状態移行・変更後ライブ全経路は別の受入れ [依存分離][independence] |
| PQC | 独立P0の実装 | 既存全サービスへの配線や全体の耐量子性は未実証 [P0][p0] |

OCLOBは別タスクで変更中である。追加snapshotは契約・manifest・artifact・ledgerのhashと実行済みverdictだけを取り込み、作業を停止・変更していない。lifecycle-final-004は旧配布DeFMI revisionでの実行で、今回の分離後バイナリのライブ証拠へ流用しない。同時実行、無人復旧、UI、独立運営の未受入れ条件を維持する。

## 5. 採用できる表現と棄却する表現

| 表現 | 判断 | 使用条件・代わりに示す内容 |
| --- | --- | --- |
| 「秘密の注文受付から、価格・時間優先照合、事前予約、証明付き決済までを結ぶ研究実装」 | 使用可 | 新CLI/Docker経路と、単一ホストの実行範囲を併記 |
| 「2約定を1取引で原子的に決済した」 | 使用可 | 該当smokeの条件と成果物を添える。一般的な処理能力とは言わない |
| 「指定した注文集合・規則と、決済結果の一致を独立に検証することを目指す」 | 条件付き | verifierが実際に検証する命題、署名者・readbackへの信頼を明示 |
| 「世界初の金融MPC」「唯一のMPC＋ZK決済」 | 棄却 | Prime Match、Renegadeが先行 |
| 「競合は回路だけ、自分たちは市場規則を証明する」 | 新規性の根拠として棄却 | Renegadeにも照合の正しさの証明があり、CantonはDaml規則を関係participantが検証する。関数・集合・権限・決済の具体差を示す |
| 「Cantonは注文・資格・予約を実装できない」「Cantonには原子的DvPがない」 | 棄却 | Damlで同業務を構築できる可能性とcross-app atomic transactionを認め、公開済みの個別アプリ範囲だけを比較する |
| 「7ノードなのでArciumより秘密が強い」 | 棄却 | 正直な参加者に関する仮定が異なる |
| 「秘密の台帳なので資産種類・通信・実名まで全て隠れる」 | 棄却 | 公開asset ID、通信観測、DeKYXの発行者との関係を区別 |
| 「本番のFMI/CCP」「全体が耐量子」「競合より高速」 | 現時点では使用不可 | それぞれ制度・運用、全暗号経路、同条件の性能証拠が必要 |

## 6. 次に比較を更新する条件

1. **C02/C06:** 継続取引、取消・期限切れ・競合・復旧、独立運営の新しい受入receiptが得られたとき。
2. **C01/C03/C09:** CantonまたはRenegadeの同じ受付集合・順序・資格・予約・決済条件との対応が明示されたとき。未確認項目を勝手に欠点へ変えない。
3. **C04:** 同じ全経路をArcium/Zamaで実行し、出力・秘密範囲・費用・障害時の違いが測れたとき。
4. **C05/C07:** 台帳接続先がAPI、hold/commit/abort、readback、法的業務の責任分担を確認したとき。
5. **C08:** 署名/KEM以外を含む移行receiptが得られたとき。
6. 各社の正式な版・稼働段階・修正告知が変わったとき。外部提案へ転用する前には該当資料を再確認する。

## 7. 出典と取得上の制限

v1.2追加: Fableは既存のCanton関連15URL中14URLを再取得し、Tradeweb原URLは公式転載で代替、新規23URLを取得した。ID付き参照38URLのうち照合できたものは37URL。原URLと転載を独立した採用事例に二重計上しない。取得時刻と内訳は[Fable 1・9節][fable]、統合文書の参照URL集合は[出典索引][source_index]を参照する。以下の54件はv1.1時点の内訳である。

外部の基礎資料は[初回サーベイ][survey]の37件と、その後に追加したRenegade公式リポジトリ・P2P説明・Zama coprocessorの3件を引き継ぐ。[Canton追加調査][canton]ではCanton / Digital Asset / Global Synchronizer Foundationの仕様・運用資料、Broadridge / Tradewebの採用者発表を含む新規14件を確認し、重複を除く合計を54件とした。MUFGのJGB repo資料は初回サーベイJ3を再利用した。

Canton pilot PDFはWeb取得時のサイズ制限で直接openできなかったため、取得できた公式pilot完了発表を根拠にし、PDFを読了資料へ数えていない。Renegade白書の502、Zama本文のcontent-typeエラー、Kinexysの403も従来どおり読了扱いにしない。

L1〜L15は[基準ファイル][baseline]の原本と対応する。リンク先の作業ツリーが将来変わる可能性があるため、比較時のバイト列は同ファイルのSHA-256を基準にする。本書はソース監査・ベンチマーク・顧客インタビューを実施した記録ではない。

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
