# P0 暗号境界

## 実装範囲

実装は `ed25519-dalek =2.2.0`、`ml-dsa =0.1.1`、`ml-kem =0.3.2`、
`x25519-dalek =2.0.1` に固定する。曲線の解決版は `curve25519-dalek 4.1.3`。
`Cargo.lock` を同梱し、間接依存も固定する。新規暗号コアの自作はせず、用途別の
バイト列API、鍵メタデータ、正規化、指定のハイブリッド合成だけを実装する。

登録済み操作は Ed25519 / ML-DSA-65 / Ed25519+ML-DSA-65 の署名・検証と、
ML-KEM-768 / X25519+ML-KEM-768 のカプセル化・復号。FROST、Pedersen、
Bulletproofs、ML-DSA-44、SLH-DSA、従来TLS、ハッシュの SuiteId は識別用であり、
未登録の暗号操作は `UnsupportedSuite` で拒否する。既存サービスへは未配線。

## バックエンド選定と出典

RustCrypto の ml-dsa 0.1.1 / ml-kem 0.3.2 を P0 の第一バックエンドとする。
純RustでCツールチェーンが不要、Apache-2.0 OR MIT、MSRV宣言1.85である。
自前トレイトはバイト列を使い、将来 aws-lc-rs へ差し替えられる境界にする。
この選定は本番採用、外部監査、FIPS認証を意味しない。両クレートの上流は
独立監査未実施と記載している。aws-lc-rs の採用は別途判断する。

| 上流 | 発行物のVCS commit | crates.io tarball SHA-256 |
| --- | --- | --- |
| [RustCrypto/signatures ml-dsa](https://github.com/RustCrypto/signatures/tree/f75d5b829948988f18d9463f286805fb9410bcdd/ml-dsa) | `f75d5b829948988f18d9463f286805fb9410bcdd` | `add6b9d92e496f16f4526d68ff29da1483aba4b119baeab8bed3b9e3544a6f3d` |
| [RustCrypto/KEMs ml-kem](https://github.com/RustCrypto/KEMs/tree/440768245bba59784b504269cb3087a6c21af45c/ml-kem) | `440768245bba59784b504269cb3087a6c21af45c` | `5e15f3e5b957493873e396a66914e83e616b6afe335cdef7efe5c6e1216aba66` |

依存をCargoから利用し、上流実装のコピーや改変はしていない。
古典側の乱数は rand_core 0.6、PQ側は独立したOS乱数の取得または rand_core 0.10。
Ed25519側の signature 2 と ML-DSA側の signature 3 を共通トレイトとして扱わない。
秘密鍵には各依存の zeroize 機能を有効にし、所有するseedと共有秘密は
`Zeroizing` で保持する。seedインポート呼び出し側が所有する元バッファの消去は
呼び出し側の責任。秘密鍵・Signer・共有秘密の保存用serdeは提供しない。

## 署名と正規化

`SigningPreimage` の先頭は固定の `ZKFMI:CANONICAL:v1`。可変長値は
u32のバイト長、配列はu32要素数、整数は固定幅big-endianでエンコードする。
文字列はUTF-8の完全一致で扱い、暗黙のUnicode正規化や並べ替えはしない。
object_idsはプロトコルで決めた順番。本文ハッシュはSHA-256の32バイトで固定し、
本文自体の正規化は呼び出し側のプロトコルが定義する。

署名コンテキストは `ZKFMI:SIGNATURE:v1 || KeyPurpose(u16) || Suite(u16,u16)`。
ML-DSAにはこれをFIPS 204 contextとして渡す。Ed25519には独立した
`ZKFMI:ED25519-CONTEXT:v1` と長さ前置したcontext・messageを署名させる。
ハイブリッドの両成分ともハイブリッドsuiteを束縛し、単独署名の転用を拒否する。
両方の検証成功だけを受理し、片側失敗・欠落・入替では失敗する。

## KEM合成

公開鍵は X25519 の32バイトと ML-KEM の1,184バイト。暗号文は送信側の一時
X25519公開鍵32バイトと ML-KEM暗号文1,088バイトを固定順に連結する。
HKDF-SHA256のsaltは `ZKFMI:HYBRID-KEM:v1`、infoは
`ZKFMI:HYBRID-KEM:SESSION:v1`、出力は32バイト。
ikmは `ss_x25519 || ss_mlkem || ct_x25519 || ct_mlkem || suite_id || suite_version`。
suite_idとsuite_versionはそれぞれu16 big-endian。片側の秘密・暗号文の変更が
出力へ反映されることを試験する。これは形式的安全性証明を代替しない。

X25519の非寄与な共有値と不正な長さを拒否する。ML-KEMはFIPS 203の
implicit rejectionに従い、正しい長さの改ざん暗号文には異なる共有秘密を返す。
古典側だけへのフォールバックはない。このKEM自体は相手の認証や鍵確認を行わず、
TLSハンドシェイクやP1の設定変更も実装していない。

## 鍵管理の責任境界

参加者IDは公開鍵と独立し、値を変更するAPIを持たない。鍵の有効期間は
`[not_before, not_after)`、`revoked_at` の時点から失効する。時刻単位はUnix秒。
`key_version` は1から増える鍵世代で、既知の鍵世代と一致しない利用を拒否する。
一方、プロトコル・suite・更新証明の版は現在V1だけを受理する閉じた型。

初期登録と失効は認可済み管理操作として呼び出す。DeKYX bindingは不透明な参照で、
資格の正当性をこのcrateが確認したとは扱わない。時計、期待するnetwork/deployment/
contract/protocol、nonceの再利用検査、業務上の登録権限は呼び出し側が管理する。

更新では旧鍵と新鍵の両方で、参加者・鍵ID・suite・世代・用途・公開鍵・有効期間・
DeKYX参照を含む遷移に署名する。承認と応答のドメインは別で、応答も旧鍵IDに
束縛する。両方向の検証を完了するまで状態は変更せず、成功時に旧鍵を失効させる。
参加者・用途の変更、世代飛ばし、同一鍵の再利用、PQから古典のみへの逆移行は拒否する。
更新証明は署名可能な鍵に適用する。KEM鍵の更新承認ポリシーはP1の接続側で定義する。

公開DTOはserde対応で未知フィールドを拒否する。`RegistrySnapshot` は公開記録の
保存形を提供するが、その読み込みを自動的に信頼済みレジストリへ変換しない。
既存 `PublicManifest` / `EncryptedKeyStore` の認証・保存責任を置き換えない。

メタデータの `post_quantum` は方式の分類であり、実装認証や全用途での安全性宣言ではない。
ハッシュにはGrover等による安全性余裕の減少があり、Pedersenの完全秘匿性と
量子攻撃で破られる束縛性も別の保証として扱う。
