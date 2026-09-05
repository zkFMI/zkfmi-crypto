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
