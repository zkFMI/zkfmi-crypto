# zkfmi-crypto

ZKFMI の耐量子移行 P0 用の独立 Rust crate。暗号方式の識別、正規署名対象、
用途別の鍵管理、ハイブリッド暗号の共通境界を提供します。

QOMM / zkPI / DeFMI / OCLOB / DeKYX / DeCCP / Aethel の現行コードへの配線は
まだ行いません。既存の鍵管理を置き換えず、将来各サービスが実装できる境界を
定義します。将来の共有は Git URL と `rev` の固定を用い、複製された共通 crate の
統合を P1 配線の前提にします。P1 の TLS 設定と P2 の zkPI 格納変更は対象外です。

## 検証

ローカル Mac でのビルド・テストは禁止です。

```sh
make remote-gate
```

softbank-l40s の `~/work/zkfmi-crypto/` だけに同期し、公式
`rust:1.97-bookworm` の Docker で fmt、clippy (`-D warnings`)、release test を
順に実行します。ログは `.artifacts/` に取得します。他プロジェクトや既存 Docker
イメージを変更しません。ビルド並列数は既定で4です。

発注書全文は [docs/orders/2026-09-05-p0-handoff.md](docs/orders/2026-09-05-p0-handoff.md)。
計測していない性能値や、本番導入・FIPS 認証・外部監査の完了は主張しません。

## 現在の成果物

- [暗号契約と責任境界](docs/CRYPTO_CONTRACT.md): 登録済み方式、正規化、鍵更新、zeroize。
- [NIST / RFC既知解の出典](tests/vectors/SOURCES.md): 固定コミットと原本・抜粋ハッシュ。
- [暗号棚卸し](inventory/CRYPTO_INVENTORY.md): P0検収時点の7リポジトリの固定snapshotから生成。後続のAethel分離・OCLOB更新は含まない。
- [独立P0の検収報告](docs/verification/P0_REPORT.md): S1〜S5のコミット、実行結果、変更範囲と保留事項。
- [Canton以外の初回競合サーベイ](docs/research/COMPETITORS_EX_CANTON_2026-09-05.md): 歴史的原文として保持する17対象の秘密境界、金融業務、成熟度とZKFMIの比較課題。
- [Canton Network追加調査](docs/research/CANTON_NETWORK_2026-09-05.md): Daml、validator / synchronizer、秘密・検証・DvPの境界、商用事例とZKFMIとの差。
- [Claude Fable 5.1 Maxレビュー](docs/research/CANTON_FABLE_5_1_MAX_REVIEW_2026-09-05.md): Cantonの一次資料、予約・DvP、秘密境界と戦略の独立照合。
- [比較判断 v1.2](docs/research/COMPETITIVE_DECISIONS_2026-09-05.md): Cantonを含む18対象の扱い、確定した差、主張の限界と実装証拠。
- [ZKFMI戦略 v1.2](docs/strategy/ZKFMI_STRATEGY_2026-09-05.md): Cantonを競合・実装先・接続先候補に加えた初期顧客、提供単位、採用と開発の順序、指標と見直し条件。

公開範囲は2026-09-05のユーザー指示でpublicに決定しました。
GitHub認証が無効のため、GitHub上のリポジトリ作成とpushは未実施です。
認証復旧後はpublicで作成します。初回発注書のprivate指定はこの決定で更新されています。

## 棚卸しの再現

原本リポジトリは読み取りだけで使用します。Mac上で行うのはコピーだけです。

```sh
scripts/capture-inventory.sh /Users/shukob/Research/DeFMI "$PWD/.cache/inventory-source"
```

生成されたスナップショットをsoftbank-l40sの専用ディレクトリ内
`.cache/inventory-source/` へ同期した後、同じRust Docker環境で実行します。

```sh
cargo run --release --bin crypto-inventory -- generate .cache/inventory-source inventory
scripts/inventory_check.sh .cache/inventory-source
```

既存スナップショットの上書きは拒否します。再採取するときは新しい名前を指定します。
検出集合はGNU grepで独立に比較し、ファイル漏れ・余分なファイル・primitive漏れ・
原本ハッシュ相違・JSONとMarkdownのずれを非ゼロ終了にします。
固定スナップショットを配置済みの今回の環境では、全ゲートを次で再実行できます。

```sh
RUN_INVENTORY_CHECK=1 make remote-gate
```
