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
- [暗号棚卸し](inventory/CRYPTO_INVENTORY.md): 7リポジトリのJSONから生成した一覧。
- [独立P0の検収報告](docs/verification/P0_REPORT.md): S1〜S5のコミット、実行結果、変更範囲と保留事項。

GitHub認証は着手時に無効だったため、GitHub上のリポジトリ作成とpushは未実施です。
ローカルGitリポジトリとして管理し、将来作成する場合は発注どおりprivateを初期値にします。

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
