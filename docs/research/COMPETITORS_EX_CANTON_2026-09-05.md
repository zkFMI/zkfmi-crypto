# ZKFMI競合サーベイ：Canton以外

調査日・情報確認日: **2026-09-05**。対象はZKFMI全体（QOMM / OCLOB / zkPI / DeFMI / DeKYX / DeCCP）の顧客課題に重なる**17の製品・基盤**。暗号ライブラリの選定比較ではない。

## 1. 判断の要点

**比較の優先対象は、秘密取引のRenegade、MPC基盤のArcium、金融機関向けのCorda・Kinexys・Progmat、取引と決済を接続するOwneraである。** この優先順位は、機能の重なりと顧客接点からの本調査の判断であり、市場シェア順位ではない。

- **「MPCで秘密注文を扱い、ZKで決済する」だけでは差別化にならない。** Renegadeはその構成を公開している。比較すべきなのは、運営ノードが読める情報、注文集合と受付順への束縛、部分約定、参加資格・与信予約、約定と決済の証明の連続性である。[R1][r1] [R2][r2]
- **汎用秘密計算は競合すると同時に調達候補になる。** Arciumの現行Cerberusはdishonest-majority・detect-and-abort型、ZamaはFHEと分散鍵管理を使う。ZKFMIの価値をMPC基盤の保有だけに置かず、市場規則・権限・資産予約・受渡しを一つの検証可能な業務経路にする点で評価する。[A2][a2] [Z1][z1] [Z3][z3]
- **金融機関の採用判断では、暗号方式以外の差が大きい。** Cordaの証券決済向け採用、Kinexysの銀行サービス、Fnalityの決済制度、ProgmatのST案件は、顧客接続と既存実務の障壁を示す。ZKFMIの研究実装とこれらの商用経路は同じ成熟度ではない。[C1][c1] [K1][k1] [F1][f1] [J1][j1]
- **クロスチェーン接続も独立した競争領域である。** Ownera、Chainlink、Swiftは、それぞれ取引指図の調整、外部データ・ポリシー・メッセージ連携、銀行間支払の調整を扱う。zkPIの採用を目指すなら、既存台帳へどう接続するかまで比較する。[O1][o1] [L1][l1] [S1][s1]
- **差別化候補は「秘密を維持した市場規則の検証と、その結果に束縛された決済」。独占的な新規性や優越性は未確立。** 競合に同機能がないとは、公開資料で確認できないことだけから断定しない。

## 2. 範囲と読み方

### 選定基準

次のいずれかに該当し、公式仕様、運営主体の発表、当局資料、著者による論文のいずれかを確認できる対象を選んだ。

1. 金融機関が台帳・証券発行・担保・決済に採用する基盤。
2. 秘密注文の照合、秘密残高、第三者検証可能な取引を実装・提案する仕組み。
3. 上記を実装する秘密計算、証明、相互運用の代替基盤。

**Canton本体は比較対象から除外した。** 複数ネットワークに対応する企業はCanton以外の機能を比較する。例えば、2026-08-13のMUFG・ProgmatのJGBレポ実証はCantonを使うと原本に明記されているため、本表の「非Cantonの実績」には数えない。[J3][j3]

本調査は公開資料のサーベイである。競合の有料環境への接続、コード監査、取引の実行、相対性能の測定は行っていない。料金・TPS・レイテンシ・資金調達額を条件の違う数字で順位付けしない。

### 成熟度の表記

| 表記 | この文書で意味すること |
| --- | --- |
| 商用事例公表 | 運営者・採用者が対象業務の実施を公表。独立した運用監査を意味しない |
| Mainnet / Alpha公表 | ネットワーク段階の公表。規制市場での商用受渡しとは区別 |
| 採用・実証・予定 | 発表に書かれた段階を維持。将来日程を経過しただけでは稼働扱いにしない |
| 仕様確認 | 動作モデルを仕様で確認。特定導入の稼働状態は未確認 |
| 未確認 | 読んだ資料では確認できない。「未実装」「非対応」とは異なる |

### 自分たちの比較基準

ZKFMI側は、2026-09-05に現行の`defmi/README.md`、`defmi/POSITION.md`、qommの日本語資料、本リポジトリのP0検収報告を読み直した。DeFMIは研究ソフトウェアであり、独立運営者・本番セキュリティ・法的ファイナリティの受入れを完了したとは扱わない。native note経路では**asset IDと決済メタデータは公開**で、全フィールドが秘匿されるという比較はしない。

参照時のDeFMI HEADは`153fe671e523ec573a6c6261f341423a49371f5d`、qomm HEADは`61596e523a4249031ae2471c78bd2249fea8f49b`。このサーベイで既存スタックのE2Eを再実行したわけではない。Webサイト側の古い未了表と現行READMEが一致しない箇所を、都合のよい機能主張には使っていない。

PQCについて確認済みなのは、独立`zkfmi-crypto`のハイブリッド署名/KEMと既知解を含む45テストである。既存サービスへの配線は未実施であり、ZKFMI全体の耐量子対応を意味しない。[P0検収報告](../verification/P0_REPORT.md)

## 3. 金融機関向け台帳・決済・接続の比較

| 対象 | 主に競合する仕事 | 秘密・信頼の境界 | 確認できた段階と比較上の意味 |
| --- | --- | --- | --- |
| **R3 Corda** | 証券・資産の状態管理と機関間ワークフロー | 関係者間のデータ共有とnotaryによる二重消費防止。notaryが全取引内容を読むモデルとは限らない | CSD PragueのDLT決済向け採用を公表。ネットワーク統治と金融業務への導入が強い比較軸 [C1][c1] [C2][c2] |
| **Kinexys / J.P. Morgan** | 銀行決済、資産トークン化、ファンド関連処理 | 銀行運営のサービス、private permissioned基盤や公開チェーン上の商品。銀行からの入力秘匿とは別 | 2026-04-28にJPM CoinのBase提供、Fund Flow初回取引等を公表。現金脚と顧客接点が競合 [K1][k1] [K2][k2] |
| **Fnality** | 機関向けデジタル現金の決済 | 中央銀行の口座に保有する資金を裏付けとする仕組み。秘密注文の価格形成が主対象ではない | BoE最新監督資料でSterling FnPSを確認。段階的運用の条件があり、全通貨・全機能の完成とは扱わない [F1][f1] [F2][f2] |
| **Partior** | クロスボーダー支払、FX PvP | 参加金融機関の決済ネットワーク。秘密市場計算の仕様は本資料では未確認 | 支払ネットワークの稼働事例と、OpenAssetsとのDvP **PoC**を分けて公表。現金脚・PvPの代替/接続候補 [T1][t1] |
| **Ownera / FinP2P** | 台帳をまたぐ資産流通と決済指図の調整 | 各機関のRouter、adapter、参加機関間の合意。元台帳・支払基盤に依存 | 実装APIとorchestration planを公開。zkPI/SDKの導入位置に近い [O1][o1] [O2][o2] |
| **Swift共有台帳** | 銀行間支払の調整とtokenised deposit接続 | Swift運営の共通層と銀行側台帳。最終決済は既存システムを使う経路を説明 | 2026-07-09時点は初期利用準備完了・銀行によるlive pilot準備。全面商用移行とは書かれていない [S1][s1] [S2][s2] |
| **Chainlink CCIP / CRE / ACE** | 外部情報・ポリシー・クロスチェーン処理の接続 | DON、TEE、DKG等。非公開APIへのアクセスと秘密計算を構成する | 2026-05の説明はprivacy機能と開発事例を含む。すべてが規制された金融FMIの本番事例という意味ではない [L1][l1] [L2][l2] |
| **Progmat** | 国内デジタル証券発行・管理、金融商品実務との接続 | 発行者・信託・販売等の制度上の役割が重要。全関係者からの入力秘匿は本資料で未確認 | ST案件一覧とAvalanche L1への移行完了の提供者発表を確認。国内実務への導入が直接競合。CantonのJGB案件は除外 [J1][j1] [J4][j4] [J5][j5] |
| **BOOSTRY / ibet for Fin** | 国内STの発行・流通とコンソーシアム運営 | メンバーによるネットワーク・標準化されたST取扱い。詳細な秘匿範囲は導入構成の確認が必要 | 公式コンソーシアム説明と運営開始の参加者発表を確認。日本の証券業務を一から実装する場合の比較対象 [B1][b1] [B2][b2] |

### Corda: 共有範囲を限定する台帳と、秘密計算の違い

Corda 5.2のnon-validating notaryは入力stateの参照等を扱い、commandや署名の全内容を受け取る必要はない。一方、取引参加ノードの検証と履歴取得には別の開示境界があり、Cordaの全ノードが同じ情報を見るという説明も、どの参加者にも取引が見えないという説明も誤りになる。[C2][c2]

**比較判断:** ZKFMIが狙う「価格ルールや他者の注文を開かずに適用結果を検証する」用途と、Cordaで権利・契約状態を扱う用途を同じ案件で比較する。R3のSolana向け「Corda protocol」は従来のCorda台帳製品と分ける。2025-12の発表は2026年前半の開始予定であり、この発表だけから9月時点の実稼働を断定しない。[C3][c3]

### Kinexys・Fnality・Partior: 現金脚と営業接点が強い

Kinexysのサービス提供は、銀行の預金・顧客関係・運用責任と組み合わされている。Fnalityの監督上の位置づけやPartiorの支払経路も、暗号クレートを実装したことでは代替できない。[K1][k1] [F1][f1] [T1][t1]

**比較判断:** 当面の案件では、これらを全面的に置き換える提案だけでなく、zkPIで外部計算を検証し既存現金脚へ接続する提案を比較する。ただし、接続可能性は本調査の提案であり、ZKFMIとの接続実績はない。

Fnality自身の2023年公表は初回live payment、BoEの2025–26年報は2024年12月の限度付き運用開始を記載する。イベントの表現と日付が異なるため、一つの「全面本番開始日」に統合していない。[F1][f1] [F2][f2]

### Ownera・Swift・Chainlink: 指図の形式だけでは競争できない

FinP2Pは、取引の意図を複数Router間の合意と各台帳の命令に変換する具体的な仕組みを持つ。Swiftも銀行間commitmentを調整する共有層を整備している。**「既存台帳をつなぐ」「指図を標準化する」だけをzkPI固有の価値としない。**[O1][o1] [S1][s1]

Chainlinkの秘密HTTP処理はTEEと分散鍵生成を用いると説明される。ZKによる証明、MPCによる秘密入力の共同計算、TEE内での平文処理は信頼の置き方が違う。ベンダーの「verifiable」という一語から、同じ第三者検証能力があると採点しない。[L1][l1]

**比較判断:** 採用者には、指図の中に何の命題が束縛されるか、署名だけでなく何を再検証できるか、二重実行や片脚失敗の責任がどこにあるかを示す。台帳間で合意したことと法的な受渡しが完了したことを分けて比較する。

### Progmat・ibet for Fin: 国内では制度と運用への接続を比較する

STの発行・権利移転・販売・記録管理を担う組織との接続が競争軸になる。ProgmatのST案件一覧と、ibet for Finのネットワーク・標準契約の説明は、この領域が暗号実装以外の業務を伴うことを示す。[J1][j1] [B1][b1]

非Cantonの台帳経路について、Avalancheの2026-02-25発表はCordaから専用Avalanche L1への移行計画を記載し、AvaCloudの公式投稿はその後の移行完了を公表している。後者は確認時に相対日付「1mo」と表示されたため、正確な完了日を推定しない。移行完了は提供者発表として扱い、稼働状況や性能を独自検証したものではない。[J4][j4] [J5][j5]

**比較判断:** 日本の案件では優先して比較する。秘密の適格性・保証枠判定や担保計算が、既存の登録・信託・決済業務へどう組み込めるかを具体化する。投信案件のリリース原本は2026-08-28付、Web掲載は09-03で、外部投資家への募集・販売をしない実証と明記されている。「実運用環境での実証」を商用販売開始に読み替えない。この原本だけでは利用チェーンを特定できないため、非Canton台帳の実績を示す根拠には使わない。[J2][j2]

## 4. 秘密注文・秘密資産取引の比較

| 対象 | 市場・検証の仕組み | 秘密・信頼の境界 | 確認できた段階と比較上の意味 |
| --- | --- | --- | --- |
| **Renegade** | pairwise MPCがcollaborative SNARKを生成し、オンチェーンで残高を更新 | 接続先relayerは自分が担当するwalletの注文・残高を平文で読める。他relayerには隠す | Arbitrum One mainnet開始を公式サイトが公表。OCLOB/秘密取引の近接比較対象 [R1][r1] [R2][r2] [R3][r3] |
| **Prime Match** | 金融機関と顧客の在庫照合をMPCで行う | 著者が定義した参加者・銀行の脅威モデル。ZKFMIと同じノード構成ではない | 2023年論文でJ.P. Morganのlive運用を報告。金融MPCが初めてという訴求を否定する先行実装 [P1][p1] |
| **Penumbra** | shielded poolとブロック単位のbatch DEX、取引/claimの証明 | 現行仕様ではswap入力のassetとamountを公開。shielded transferの秘匿範囲とは異なる | 実装仕様を確認。sealed-bid batch swapは参照仕様で将来機能と明記 [N1][n1] [N2][n2] |
| **Dusk** | 規制資産を意識した台帳・秘密移転・選択開示 | 取引モデルとアプリによる。公開account modelも存在 | Mainnetの接続仕様、NPEX等との協業を確認。各金融市場の実稼働と免許適用は個別確認が必要 [D1][d1] [D2][d2] |

### Renegade: 最初に比較すべき秘密取引の実装

MPCの結果としてZK証明を出し、秘密状態の決済につなげる構成は既に存在する。一方、公式のrelayer仕様では、顧客が委託したrelayerは注文とwallet残高を平文で読む。顧客が自分のrelayerを運営する選択も用意されている。[R1][r1] [R2][r2]

**比較判断:** ZKFMIの秘密分散が顧客から計算ノードまで実際に維持されるなら、この委託境界に差がある。ただし、ノード群の結託条件、企業クライアントからのshare配送、復号可能なgatewayの有無まで示す必要がある。さらに、注文の有効性を示す証明と、対象注文を省略せず価格・時間優先を適用したことの証明を分ける。Renegadeが後者に非対応だと断定するための十分な監査はしていない。

### Prime Match: 商用MPCの先行事例

著者による論文要旨は、プライバシーを保った在庫照合とJ.P. Morganでのlive運用を報告している。ここから確認できるのは論文発表時の実施であり、2026年の現在稼働・件数・契約提供範囲を再確認したものではない。[P1][p1]

**比較判断:** 計算対象、許容する結託、誤動作時の挙動、外部監査者が確認できる命題、決済までの接続範囲で比べる。速度の比較は同一条件の再実行なしでは行わない。

### Penumbra: 「private DEX」のラベルだけでは秘匿範囲は分からない

公式仕様では、通常のshielded transferとswap入力の情報開示は異なる。swapはassetとamountを明らかにし、後続のclaimで秘匿出力を生成する。sealed-bid版は将来拡張として説明される。[N1][n1] [N2][n2]

**比較判断:** OCLOB/QOMMとの比較は「第三者からwalletが結び付くか」と「約定前の注文価格・量が読めるか」を別行にする。バッチ決済の実行順保護も、連続板の価格・時間優先とは別の市場設計である。

### Dusk: 規制資産向けの業務設計が重なる

Duskは秘密移転、選択開示、資産ライフサイクルとDvPを意識した構成を説明する。一方、取引所の接続ガイドはpublic account modelのMoonlightを指定しているため、Duskの全取引を一律に秘密とは扱わない。[D1][d1] [D2][d2]

**比較判断:** NPEXとの協業や特定事業者の免許を、任意のDusk上アプリに認可が及ぶ根拠にはしない。ZKFMI側もDeCCPのコードがあるだけで法的CCPやnovationが成立したとは扱わず、同じ基準で比較する。

## 5. 秘密計算・アプリ基盤の比較

| 対象 | 提供する機能 | 秘密・信頼の境界 | 確認できた段階と比較上の意味 |
| --- | --- | --- | --- |
| **Arcium** | Solanaと協調する汎用MPC、MXE、秘密アプリ | 現行Cerberusは少なくとも1メンバーが正直という仮定で秘密を保ち、異常検出時はabort。可用性は別条件 | 公式サイトはMainnet Alphaと表示。MPC開発基盤・秘密取引アプリの競合/調達候補 [A1][a1] [A2][a2] |
| **Zama Protocol** | FHEによる暗号化状態上の演算、機密トークンと権限制御 | FHE演算と復号権限・分散鍵管理を分けて考える。KMSはstrong honest majorityを仮定 | 2025-12-31 mainnet開始、2026-01の秘密入札を公式公表。汎用秘密金融アプリの競合/調達候補 [Z1][z1] [Z2][z2] [Z3][z3] |
| **Aztec** | Ethereum L2上のprivate/publicアプリ、端末側証明 | private witnessを端末側に置く構成。複数企業の秘密入力を共同計算する仕組みとは別 | Alpha V5。2026-08-07に重大な証明系脆弱性を公表、V6での修正予定を記載 [X1][x1] [X2][x2] |
| **Hyperledger Fabric** | permissioned台帳、契約実行、組織間private data | 許可された組織のpeerに実データ、channel全体にはhash。ordererはprivate dataを受け取らない | 公式仕様を確認。金融機関が既存基盤上で内製する場合の比較対象 [H1][h1] |

### Arcium: 自分たちより弱い仮定のMPCと決め付けない

現行docsはCerberusをdishonest-majority・detect-and-abort型と明記する。「自分たちは複数ノードだから安全、競合は中央管理」という比較は成立しない。少なくとも一者の正直さで守る秘匿性と、処理を最後まで完了できる可用性を分ける。[A2][a2]

**比較判断:** MPC基盤自体の再実装よりも、予約済み資産・資格・与信・注文順・決済権限をどのように一つの業務証跡へ束縛するかにZKFMIの開発理由を置く。Arcium上で同じ業務を構築する代替案も評価対象になる。

### Zama: 秘密演算と秘密入札は既に公開導入の段階

公式発表は、mainnet開始とFHEを使ったsealed-bid auctionの実施を説明する。2025年のtestnet記事だけを根拠に「FHEはまだ実用前」とは書けない。[Z2][z2]

鍵管理の公開仕様は、参加者数を`n`、許容する故障・悪意ある参加者数を`t`として、`t < n/3`のstrong honest majorityを前提に鍵生成・復号の完了を保証する。Arciumの少なくとも一者が正直なら秘密を保つdetect-and-abortモデルとは、結託条件と完了保証が違う。[Z3][z3] [A2][a2]

**比較判断:** 注文・担保計算をFHEに載せる案と、MPC＋証明を用いる案では、復号権限、計算可能な型、完了待ち、失敗・再試行、監査者が追える情報を揃えて比べる。本調査では両者の性能差を測定していない。

### Aztec: アプリ基盤としての価値と、その版の状態を分ける

private関数を端末で実行・証明する構成は、秘密の保有状態を使うアプリの代替になる。[X1][x1] ただし、2026-08-07の公式告知はV5 Alphaの重大な証明系脆弱性とV6での修正予定を記載する。本調査時に参照した告知では、修正完了を確認できなかった。[X2][x2]

これは全バージョンのAztecが恒久的に危険という結論ではない。採用検討では対象バージョンと修正完了・移行証跡を再確認する。ZKFMIにも独立監査が未完了という制約があり、競合のAlpha段階だけを理由に自分たちを本番品質と位置づけない。

### Fabric: 組織間の秘密共有と、演算者にも隠す秘密計算を分ける

Private Data Collectionは、許可されたpeer群へ実データを送り、他peerへhashを残す仕組みである。データを見せる組織を限定する用途には適合するが、そのままでは許可peer自身からも演算入力を隠すMPCの説明にはならない。[H1][h1]

**比較判断:** 全員から隠す必要がない業務なら、既存の組織間統治とFabricの方が導入理由を説明しやすい場合がある。ZKFMIを採る理由は、閲覧権限の制御だけでは満たせない秘密境界と外部検証要件で示す。

## 6. 差別化を主張する前に示すべきもの

以下は競合の欠点一覧ではなく、比較から導くZKFMI側の検証課題である。新規実装や実験の実施をこのサーベイで承認・着手したものではない。

| 訴求候補 | 必要な証拠 | 主に比較する対象 |
| --- | --- | --- |
| 運営者にも秘密の注文・価格ルール | 企業側でのshare生成からMPC・決済までの閲覧主体一覧、結託条件、平文復号点、鍵の管理者 | Renegade、Arcium、Prime Match |
| 約定ルールを後から検証できる | 対象注文集合・受付順・適格性・価格/時間優先・部分約定を束縛する命題と、独立検証手順 | Renegade、Penumbra、Zama上の市場アプリ |
| 計算結果と受渡しが一致する | 同じcommitmentを計算・指図・資産予約・両脚消費へ結び、再送と二重消費で状態が変わらない証跡 | Ownera、Corda、Chainlink |
| 金融機関が導入できる | 発行/保管/登録/現金脚の責任分担、障害回復、監督開示、参加者加入・退出、法的な決済完了 | Kinexys、Fnality、Partior、Swift、Progmat、ibet for Fin、Dusk |
| 長期間の機密性と検証可能性 | 署名、KEM、commitment、証明、TLS、保存データを分けた脅威モデルと移行・失効・再検証の経路 | Arcium、Zama、Aztecを含む採用構成全体 |

「暗号化されている」「分散している」「検証可能」という表現だけでは上表を満たさない。特に、正しく評価した回路の証明、対象集合内の最良約定、法令上の最良執行義務は同じ命題ではない。

### PQCの比較で保留すること

今回の資料確認だけでは、各製品の署名・鍵交換・commitment・証明・台帳合意を通したPQC状態を網羅できない。**未確認を非対応と採点しない。** FHEという方式名やPQC署名ライブラリの採用だけで、システム全体を耐量子と呼ばない。自分たちについても、独立P0の完了と既存決済スタック全体の移行を区別する。

## 7. 競争上の行動案

1. **技術比較の最初の対象をRenegadeとArciumに置く。** 注文内容を読める主体と、出力証明が保証する市場規則の違いを具体化する。暗号名による比較から始めない。
2. **日本の提案書ではProgmat・ibet for Finを必ず比較に含める。** 匿名性だけでなく、資格、資産登録、受渡し、開示と現行の業務責任を説明する。
3. **Ownera・Chainlink・Swiftとの重複を認めた接続案を検討する。** zkPIを追加することで既存指図の何が独立検証可能になるかを提示する。
4. **Kinexys・Fnality・Partiorは現金脚の競合であると同時に、接続先になり得ると考える。** 利用可能な接続契約/APIや法的受渡し条件は未確認であり、実装済みと紹介しない。
5. **外部向け表現は候補となる構成と現在の受入れ段階に限定する。** 「世界初の金融MPC」「唯一の秘密取引」「競合より高速」「全体が耐量子」といった主張はこの調査からは導けない。

本調査は17対象の比較であり、市場全体の網羅を主張しない。個別の既存取引所、カストディ、発行体向けSaaS、全てのFHE/TEE/MPCベンダーを列挙することより、ZKFMIの各層で何を購入・内製・接続するかの選択に直結する対象を優先した。

## 8. 一次資料と確認範囲

全リンクの確認日は2026-09-05。公開日は本文またはリンク名に明示された場合のみ記載した。日付のない仕様は版・確認日を基準にする。企業の発表は企業が公表した事実として扱い、当局・採用者・独立した運用監査と同一視しない。

| ID | 一次資料・公開日/版 | 主に確認したこと |
| --- | --- | --- |
| C1 | [R3: CSD PragueによるCorda採用、2024-11-07][c1] | 証券決済向け採用の公表 |
| C2 | [Corda 5.2: non-validating notary][c2] | UTXOの一意性、参加ノードとnotaryの開示差 |
| C3 | [R3: Corda protocol発表、2025-12-12][c3] | Solana向け別プロダクトと予定の区別 |
| K1 | [Kinexys milestones、2026-04-28][k1] | Base上のJPM Coin、Fund Flow初回取引等 |
| K2 | [Kinexys製品ページ][k2] | 銀行決済・private permissioned資産基盤 |
| F1 | [BoE: FMI Annual Report 2025–26][f1] | Fnalityの監督対象・限度付き運用の記載 |
| F2 | [Fnality: 初回Sterling payment公表、2023-12-14][f2] | 初期live paymentの位置づけ |
| T1 | [Partior公式][t1] | 支払/PvP、参加銀行の稼働事例、DvP PoCの区別 |
| O1 | [Ownera: Intent-Based Orchestration][o1] | Router間のproposal/approvalと台帳間調整 |
| O2 | [Ownera: Integration Use Case Guides][o2] | 取引・発行者・支払コネクタのAPI構成 |
| S1 | [Swift: 共有台帳の初期利用準備、2026-07-09][s1] | live pilotの準備と既存システムによる最終決済 |
| S2 | [Swift: March 2026 newsletter][s2] | Besu基盤、Swift運営、銀行側資産・資金管理 |
| L1 | [Chainlink: privacy構成、2026-05-21][l1] | TEE・DKG、秘密HTTP、private token、開発事例 |
| L2 | [Chainlink: CCIP、2026-04-22][l2] | 相互運用機能とメッセージプロトコルの境界 |
| J1 | [Progmat ST案件実績][j1] | 国内ST案件の継続的な一覧 |
| J2 | [Progmat: 国内籍トークン化投信の実証、原本2026-08-28・掲載09-03][j2] | 外部募集・販売をしない実証。原本だけでは利用チェーンを特定できない |
| J3 | [MUFG: JGBレポ実証、2026-08-13][j3] | Cantonを使うため非Canton実績から除外 |
| J4 | [Avalanche: Progmatの移行計画、2026-02-25][j4] | Cordaから専用Avalanche L1への移行計画 |
| J5 | [AvaCloud公式: Progmatの移行完了の投稿][j5] | 提供者による移行完了発表。相対日付から正確な日付を推定しない |
| B1 | [BOOSTRY: ibet for Finコンソーシアム説明][b1] | 公式検索索引の要約でネットワークとST標準を確認。本文抽出は不成功 |
| B2 | [SBIほか: ibet for Fin運営開始、2021-06-15][b2] | 参加者による運営開始発表。現在の規模はここから推定しない |
| R1 | [Renegade: relayerの役割][r1] | 接続walletの平文閲覧、pairwise MPC、自前relayer |
| R2 | [Renegade: collaborative zkSNARK][r2] | MPC出力としての証明とオンチェーン決済 |
| R3 | [Renegade公式][r3] | Arbitrum One mainnet開始の公表 |
| P1 | [Prime Match論文要旨・書誌、2023][p1] | 著者による秘密在庫照合と当時のlive運用報告 |
| N1 | [Penumbra: Batch Swaps仕様][n1] | V1と将来sealed-bid版の区別 |
| N2 | [Penumbra: Privacy Features][n2] | transfer・swap・claim・LPごとの開示範囲 |
| D1 | [Dusk公式][d1] | 資産業務と選択開示、NPEX等との関係 |
| D2 | [Dusk: Exchange integration][d2] | mainnet endpointとMoonlight public account指定 |
| A1 | [Arcium公式][a1] | Mainnet Alphaの表記 |
| A2 | [Arcium: MPC protocols][a2] | Cerberusの脅威モデルとabort・可用性 |
| Z1 | [Zama公式][z1] | FHEを使う機密金融アプリの構成と事例 |
| Z2 | [Zama: Mainnet Season 1、2026-02-11][z2] | mainnet開始日と秘密入札の実施 |
| Z3 | [Zama KMS: Threshold cryptography concepts][z3] | 分散鍵管理、strong honest majorityと鍵生成・復号の完了保証 |
| X1 | [Aztec公式][x1] | 端末側証明とprivate/publicアプリ |
| X2 | [Aztec: Alpha V5脆弱性告知、2026-08-07][x2] | 当該版の状態とV6修正予定。完了の確認は未取得 |
| H1 | [Hyperledger Fabric: Private data][h1] | 許可peerへの実データと全体へのhash、ordererの境界 |

Renegade白書の本文取得は502、Zama litepaperは閲覧ツールでcontent-typeエラーとなった箇所がある。上表の要約では、取得できた公式FAQ・仕様・発表を根拠とし、取得できなかった原本を読了扱いにしていない。BOOSTRYの本文抽出制約もB1に明示した。追加の実装監査・当局確認が必要な点は各節の未確認事項を引き継ぐ。

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
