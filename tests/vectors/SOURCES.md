# 独立した既知解の出典

取得日: 2026-09-05。暗号クレートに同梱されたテストの転用ではなく、NIST / RFCの
原本から必要な公開テストデータを抽出した。これらのseed・dkは公開の試験値であり、
実運用の鍵ではない。ACVPサービスの認証取得やFIPS実装認証を意味しない。

## NIST ACVP

リポジトリ: [usnistgov/ACVP-Server](https://github.com/usnistgov/ACVP-Server)

固定コミット: `975de31eb83d87039ec88934fdc47d8c312b892d`

| 保存先 | 原本 | 選択したケース |
| --- | --- | --- |
| `ml-dsa-65-sigver.json` | [ML-DSA-sigVer-FIPS204/internalProjection.json](https://github.com/usnistgov/ACVP-Server/blob/975de31eb83d87039ec88934fdc47d8c312b892d/gen-val/json-files/ML-DSA-sigVer-FIPS204/internalProjection.json) | tgId 3、external/pure、tcId 31/33/42/43。正常2件、改ざん2件、255バイトcontextを含む |
| `ml-kem-768-encapdecap.json` | [ML-KEM-encapDecap-FIPS203/internalProjection.json](https://github.com/usnistgov/ACVP-Server/blob/975de31eb83d87039ec88934fdc47d8c312b892d/gen-val/json-files/ML-KEM-encapDecap-FIPS203/internalProjection.json) | tgId 2のencapsulation tcId 26/27、tgId 5のdecapsulation tcId 86/88。88は改ざん暗号文のimplicit rejection |

変更内容: 2026-09-05に対象パラメータの上記ケースを抜粋し、検証に不要なフィールドを
除いた。暗号入力と期待値は変更していない。`m` / `reason` が原本に無いケースは
抽出JSONでnull。出典は米国 National Institute of Standards and Technology。
原本READMEの許諾・免責通知は [NIST-NOTICE.md](NIST-NOTICE.md) に全文保持する。

SHA-256:

| 対象 | SHA-256 |
| --- | --- |
| ML-DSA原本 | `47cdd6314c7f746d02421ffcba89d4dbc7bb875ac49e07a029fdfc26fba55437` |
| ML-KEM原本 | `a556952ce869bb89c3a3196a701dad89647c193a34c86eafb61a9d710d5b810f` |
| ML-DSA抜粋JSON | `4f189e858d6c5959a41296fd1c49c5395a15bef49dd4625e548c36921efb4c21` |
| ML-KEM抜粋JSON | `23497b3f39b4cd83b3a50c458b21abff3adea309563dacb1d1d1c616cbd32b2b` |

## Ed25519

原本: [RFC 8032 section 7.1, TEST 1](https://www.rfc-editor.org/rfc/rfc8032#section-7.1)。
保存先は `rfc8032-ed25519-1.json`。空メッセージの公開鍵・署名・seedを保持し、
生成結果の一致と検証を実行する。用途コンテキストのないRFC入力を、P0の用途別署名と
混同しない。検証はP0バックエンド内部の同じEd25519 raw検証経路で行う。
