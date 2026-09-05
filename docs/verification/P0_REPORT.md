# 独立P0実装・検収報告

発注されたS1〜S5をローカルGitで完了した。暗号実装は既存サービスへ未配線。
softbank-l40sの公式Rust Dockerで、コミット済みの実装に対してfmt、clippy、
45テスト、棚卸し照合が成功した。GitHub認証が無効だったため、リモート作成と
pushは未実施。これは発注で認められたローカル完結の経路である。

## コミット

| スライス | リポジトリ | コミット | 成果 |
| --- | --- | --- | --- |
| S1 | zkfmi-crypto | `7bf7a69bb45b0ce37cd7838518d498308a8f7dae` | MIT、Rust workspace、日本語README、発注書全文、project-memory、remote-gate |
| S2 | zkfmi-crypto | `bd43f598a9a78b7f74d16eafb02e10be90d9c95c` | suite、鍵管理、正規化、バイト列トレイト、実バックエンド、ハイブリッド署名/KEM |
| S3 | zkfmi-crypto | `7efd7ffa6eadc89495ed801813c4cb02fb088c6c` | 固定NIST ACVP 8件、RFC 8032 1件、出典とハッシュ |
| S2補完 | zkfmi-crypto | `c6ece07210a7f0f85ae4e548bd296fc35e630a7e` | Transport用途の認証署名鍵を許可し、KEM鍵との用途分離を回帰確認 |
| S4 | zkfmi-crypto | `3dfb937b966601e01e39602306f91ece609140b6` | 7リポジトリの凍結棚卸し、独立grep照合、生成Markdown、件数照合 |
| S5 | qomm | `61596e523a4249031ae2471c78bd2249fea8f49b` | 許可された計画書1ファイルのみ、`docs:`コミット |

以下の実装検収は、上記S4コミットのcleanな作業ツリーを同期して行った。
本報告と検収証跡を追加するコミットの後にも同じゲートを実行し、その最終HEADと
ログをタスクの最終応答に記載する。先行スライスのログに記載されたHEADは
各スライスの基点であり、当時の未コミット候補を検証したログと区別する。

## 実行環境と結果

| 項目 | 実測・証跡 |
| --- | --- |
| 呼出コマンド | `RUN_INVENTORY_CHECK=1 make remote-gate` |
| SSHホスト / hostname | `softbank-l40s` / `ngi-external022-vm1` |
| リモート作業パス | `/home/ubuntu/work/zkfmi-crypto/` |
| Docker | 公式 `rust:1.97-bookworm`、CPU上限4 |
| イメージdigest | `sha256:0e2bcaef56d041a486784e54104a81aebe0da44bd03019bd70bc0401e42e4a97` |
| rustc | `1.97.1 (8bab26f4f 2026-07-14)` |
| cargo | `1.97.1 (c980f4866 2026-06-30)` |
| 検証したコミット | `3dfb937b966601e01e39602306f91ece609140b6`、`worktree_dirty=0` |
| 実行時刻 | 2026-09-05 05:58:33〜05:58:37 UTC（14:58:33〜14:58:37 JST） |
| 同期入力manifest SHA-256 | `b07d223ff64d342dc1adf4c54a98caab5c7ac69128c9960ee8b742abd396387c` |

実行順序と結果:

1. `cargo fmt --all -- --check`: 成功。
2. `cargo clippy --all-targets -- -D warnings`: 成功。
3. `cargo test --release`: **45成功、0失敗、0無視**。
4. `cargo tree -d`: 成功。`rand_core` 0.6.4 / 0.10.1、`signature` 2.2.0 / 3.0.0。
5. `scripts/inventory_check.sh`: 終了0、下記の完全一致。

```text
grep_files=507 inventory_files=507 primitive_entries=1032 missing=0 stale=0
source_hashes=825 markdown_from_json=PASS inventory_check=PASS
```

実行全文は [final-code-gate.log](final-code-gate.log)、同期した入力の一覧とハッシュは
[final-code-input.sha256](final-code-input.sha256)。ログ上の所要時間はキャッシュ済み
検証ゲートの時間であり、署名/KEMの性能値や初回ビルド時間ではない。
`rust-version = "1.85"` はMSRV宣言であり、rustc 1.85での実行検証は行っていない。

### 45テストの内容

| 区分 | 件数 | 実行した対象 |
| --- | ---: | --- |
| バックエンド既知解 | 9 | NIST ML-DSA-65 4件、ML-KEM-768 4件、RFC 8032 Ed25519 1件 |
| canonical / suite / memory | 9 | 固定hex golden 3件、FIPSサイズ、用途・再利用境界、長さ曖昧性、未知フィールド/方式/版の拒否、project-memory形式 |
| hybrid / provider | 14 | 署名4象限、欠落/入替/長さ異常、用途・suiteの束縛、実KEM往復、片側秘密/暗号文差替え、非寄与X25519拒否、未登録操作拒否 |
| 鍵ライフサイクル | 13 | 双方向更新、原子性、未知鍵世代、失効/期限、用途、PQから古典のみへの逆移行拒否、Transport署名鍵とKEM鍵の分離 |

暗号操作には実際の指定クレートを使い、既知解の期待値は上流クレートの内蔵試験で
代用していない。NIST ACVPの固定コミットは
`975de31eb83d87039ec88934fdc47d8c312b892d`。
ML-DSAのtcIdは31/33/42/43、ML-KEMは26/27/86/88で、改ざん拒否とimplicit rejectionも
含む。原本URL、取得原本とJSON抜粋のSHA-256、ライセンスは
[SOURCES.md](../../tests/vectors/SOURCES.md)に保存した。

秘密鍵のzeroize、署名context、HKDFの入力順序、時刻の端点、管理操作の認可責任は
[CRYPTO_CONTRACT.md](../CRYPTO_CONTRACT.md)に定義した。ハイブリッド署名は両成分が
成功した場合だけを受理する。正しい長さの改ざんML-KEM暗号文には別の秘密を返す
標準のimplicit rejectionを用い、古典暗号だけでの受理へ切り替えない。

新しい正規化ドメイン `ZKFMI:CANONICAL:v1` が7リポジトリの凍結Rustソースに
存在しないこともリモートで確認した（[canonical-domain-check.log](canonical-domain-check.log)）。

## 棚卸しと件数差

2026-09-05T05:22:20Zに開始した凍結スナップショットに対し、指定された暗号クレート
識別子を含むRustファイルを数えた。原本は読み取り専用で、実際に採取した825個の
Rust/CargoファイルのSHA-256と各リポジトリのHEAD・dirtyパスを
[source_manifest.json](../../inventory/source_manifest.json)に保存した。
並走中の作業ツリーを含むため、以後の原本HEADが同じとは主張しない。

| リポジトリ | 暗号利用ファイル数 |
| --- | ---: |
| qomm | 201 |
| zkpi | 59 |
| defmi | 206 |
| oclob | 25 |
| dekyx | 3 |
| deccp | 4 |
| aethel | 9 |
| 合計 | 507 |

1,032件のprimitive単位レコードをJSONの正本として保存し、MarkdownはそのJSONから
生成した。ownerは `repo/crate` の担当コンポーネントであり、未確認の担当者名を
作っていない。quantum_statusは指定の3分類、移行フェーズはP1〜P6である。

発注件数との照合は全22クレートについて
[count_reconciliation.json](../../inventory/count_reconciliation.json)に記録した。
今回の集合はtests、hash-only、存在するexamples/benches等も含む。たとえば
qomm-transportは全Rust/src+testsで58、srcで42、srcの非ハッシュ集合で34となり、
最後の集合が発注の34件と一致する。qomm-defmiも全Rust54、src+tests46、src28、
src非ハッシュ25となり、発注の25件を再現する。

qomm-zkpiは発注7に対しsrc非ハッシュ8（src+tests13）、oclob-mpcは発注1に対し
src+tests2、oclob-settlementは発注3に対しsrc+tests5。これら3件の残差は
発注時のファイル一覧が提供されていないため原因を確定できない。漏れ検査は発注の
集計値に合わせて除外する方式ではなく、採取した実ファイルに対する独立grepとの
集合一致、primitive再生成、原本ハッシュ、JSON由来Markdownを確認する。

検査自身の陰性確認として、実在する
`aethel/crates/aethel/tests/end_to_end.rs` のレコードを一時コピーから除去すると、
`grep_files=507 inventory_files=506 primitive_entries=1031 missing=1 stale=0` となり
**期待どおり終了1**。その一時コピーは削除済み。証跡は
[s4-negative.log](s4-negative.log)。これは45テストの失敗数には含めない。

## 計画書

qommの変更は `/Users/shukob/Research/DeFMI/qomm/doc/ja/PQC_MIGRATION_PLAN.md` のみ。
43行追加・1行削除（既存のサイズ基準1行を置換）で、既存の全見出し
（レベル1〜3）の順序と文字列を比較し一致した。
コミットのファイル一覧と `git diff --check` も確認した。証跡は
[qomm-plan-check.log](qomm-plan-check.log)。

開始条件の未達と独立P0の先行根拠、複製クレート統合後のP1配線、候補一覧と
decisions.jsonlと同じ選定理由、P1のOpenSSL条件とTLS認証の保留事項を追記した。
16KB/32KBは「現行package_bytes 57,971 Bの定義を確認したうえで再設定」に修正し、
FIPSの固定サイズと3署名分の算術を表で追加した。現行package/proofサイズは
発注書からの引継ぎ値であり、今回の再計測結果ではない。

## 変更パス

新規リポジトリのルートは `/Users/shukob/Research/DeFMI/zkfmi-crypto/`。
以下はこのルートからの相対パスで、すべて今回の新規ファイルである。

| パス | 内容 |
| --- | --- |
| `.gitignore`, `Cargo.toml`, `Cargo.lock`, `LICENSE`, `Makefile`, `README.md` | 骨格、依存固定、MIT、説明、遠隔ゲート |
| `.codex/project-memory/project.toml`, `facts.jsonl`, `decisions.jsonl`, `worklog.jsonl` | 境界、確認済み事実、選定、スライス記録 |
| `docs/orders/2026-09-05-p0-handoff.md` | 添付発注書とbyte単位で一致する全文コピー |
| `docs/CRYPTO_CONTRACT.md` | 仕様、上流出典、保存/認可責任、実装限界 |
| `src/lib.rs`, `error.rs`, `suite.rs`, `canonical.rs`, `key.rs`, `traits.rs`, `backend.rs`, `backend_tests.rs` | 暗号API、DTO、ライフサイクル、実バックエンド、既知解テスト |
| `src/hybrid/mod.rs`, `signature.rs`, `kem.rs` | ハイブリッド署名/KEM |
| `src/bin/crypto-inventory.rs` | Rust製棚卸し生成・検査 |
| `tests/canonical_and_suite.rs`, `hybrid.rs`, `key_lifecycle.rs` | 36件の統合テスト |
| `tests/vectors/ml-dsa-65-sigver.json`, `ml-kem-768-encapdecap.json`, `rfc8032-ed25519-1.json`, `SOURCES.md`, `NIST-NOTICE.md` | 独立ベクトル、URL/commit/hash、原本ライセンス |
| `scripts/remote-gate.sh`, `capture-inventory.sh`, `inventory_check.sh` | 隔離同期、読取専用採取、独立grep照合 |
| `inventory/crypto_inventory.json`, `source_manifest.json`, `count_reconciliation.json`, `CRYPTO_INVENTORY.md` | 正本JSON、入力証跡、件数差、生成文書 |
| `docs/verification/s1-gate.log`, `s2-gate.log`, `s3-gate.log`, `s3-input.sha256`, `s4-gate.log`, `s4-input.sha256`, `s4-negative.log` | 各スライスの検証証跡 |
| `docs/verification/P0_REPORT.md`, `final-code-gate.log`, `final-code-input.sha256`, `qomm-plan-check.log` | 本報告とコミット済み状態の検収証跡 |
| `docs/verification/canonical-domain-check.log` | 凍結した既存ソースとの正規化ドメイン非衝突確認 |

加えて、新規リポジトリ内の `.git/`、無視対象の `.artifacts/` と `.cache/`、リモートの
`/home/ubuntu/work/zkfmi-crypto/` にGit管理情報、検証ログ、採取スナップショット、
依存・ツールチェーン・ビルドキャッシュを作成した。追跡対象ファイルの削除はない。
qommでは上記計画書1ファイルのみを編集し、コミットに伴うGit管理情報を更新した。

**当タスクの書込みはこの新規リポジトリ、専用リモート作業領域、qommの許可された
計画書とそのコミット操作に限定した。** defmi、oclob、zkpi、dekyx、deccp、aethel、
zkfmi-siteのファイル、既存Cargo.lock、pin、Dockerfile、composeは編集していない。
他のリモート作業領域と既存Dockerイメージは変更していない。ローカルMacでの
ビルド/テスト、Python実装、`/tmp`への一時ファイル作成は行っていない。

## 未実施・保留

- GitHub作成/push: 着手時の `gh auth status` が無効。`gh auth login`、GitHub書込みAPI、
  リポジトリ作成、pushはいずれも実行していない。ローカルの全スライスはコミット済み。
- 既存クレートへの依存追加、共通クレート統合、P1のTLS変更、P2の格納変更: 明示的な対象外。
- 性能ベンチマークと計画全体のP0性能基準確定: 今回の発注スライスに含まれない。
  暗号処理時間や本番性能は未計測。独立S1〜S5の完了を移行計画全体のP0完了とは扱わない。
- ML-DSA-44、SLH-DSA、FROST、証明系などの実演算: 今回はSuiteIdと必要なサイズ定義のみ。
  未登録の操作は拒否する。秘密鍵保存、KEM鍵更新の認可、TLS鍵確認は既存基盤/P1の責任。
- 外部監査、本番採用、FIPS認証: 実施・取得を主張しない。

## 判断を要する事項

- リポジトリ公開範囲: 初回発注時はprivateを既定としていたが、2026-09-05の追加指示でpublicに決定済み。GitHub認証が無効のため、GitHub上の作成・pushは引き続き未実施。
- aws-lc-rs: P0はRustCryptoを採用済み。aws-lc-rsを本番候補として採用するかは別途判断。
- TLS認証: ML-DSA証明書単独か二重証明書かは未選択。P1実装前に決定する。
