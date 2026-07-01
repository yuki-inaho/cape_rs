# LLMオンボーディングサマリー

> このテンプレートは、新任LLMエージェントがプロジェクトに参加する際に共有する初期資料のたたき台です。各項目を埋め、参照元ドキュメントや補足リンクがあれば併記してください。

## 1. プロジェクト概要と目的
- **プロジェクト名称・領域:** CAPE の Rust/PyO3 非公式実装。深度画像から平面・円柱候補を抽出するネイティブ拡張と Python デモを扱う。
- **最終成果物:** `cape_pyo3` PyO3 拡張、公式 `Data/seq_example` を使う Jupyter notebook、uv ベースの検証コマンド、最小限の Python 補助 API。
- **ビジネス背景・価値:** CAPE の処理を Rust で実装し、Python/Jupyter から再現可能に呼び出せる形にすることで、実験・可視化・将来の高速化検証を進めやすくする。
- **現時点の進捗サマリ:** `data/seq_example/` に公式サンプルデータを配置済み。Python reference 実装や synthetic demo は持たず、notebook と test は PyO3 経由で動かす方針。

## 2. クリティカルな要求・制約
> 「壊してはいけない」品質・仕様ラインを箇条書きで列挙します。
- `cape_pyo3` を使う経路を主経路にする。Python reference backend や synthetic fallback を再導入しない。
- notebook は `data/seq_example/` の公式 CAPE サンプル画像を読む。任意のローカル絶対パスに依存させない。
- docs、notebooks、artifacts に個人名・ローカルホームディレクトリ・秘密情報を残さない。
- `cape_pyo3` import 前に `uv run maturin develop --release` でネイティブ拡張をビルドする必要がある。
- `.venv/`、`target/`、`temp/`、cache 類は成果物として扱わず、コミット対象にしない。

## 3. 参照すべき合意済み資料
> 新任エージェントが必ず確認すべき一次資料の一覧です。パスと役割を記載します。
| 種別 | ファイル/リンク | 概要・用途 |
|------|------------------|------------|
| 要求定義書 | `README.md` | 目的、セットアップ、主要コマンド、ネイティブ API の入口を確認する。 |
| 要件定義書 | `pyproject.toml`, `Cargo.toml`, `justfile` | Python/Rust 依存、PyO3 module 名、検証ワークフローを確認する。 |
| WBS / 進捗 | 未確認 | 専用の WBS/進捗表は現時点でリポジトリ内に見当たらない。必要なら別途作成する。 |
| テスト資産 | `tests/test_official_demo.py`, `notebooks/cape_pyo3_demo.ipynb` | 公式サンプル読み込み、PyO3 抽出、notebook 実行の確認に使う。 |
| 既知課題リスト | 未確認 | 専用の既知課題リストは現時点でリポジトリ内に見当たらない。 |
| 公式参照 | https://github.com/pedropro/CAPE/tree/master/Data/seq_example | notebook/demo で使う公式サンプルデータの参照元。 |

## 4. タスク境界（任せること / 任せないこと）
### 任せるタスク（例）
- PyO3 API の小さな整理、型や戻り値の扱いやすさの改善、対応する Rust/Python テストの追加。
- `data/seq_example/` を使う notebook、可視化、smoke check の保守。
- README、ONBOARDING、検証手順など、リポジトリ利用者向けドキュメントの更新。

### 任せないタスク（例）
- Python reference backend や synthetic demo を復活させること。
- 公式サンプルデータを出典不明の画像に差し替えること。
- 個人情報、ローカル絶対パス、認証情報を docs/notebooks/artifacts に残すこと。
- テストや検証コマンドを弱めて、PyO3 経路の失敗を見逃す変更を入れること。

## 5. インタラクション方針
- **回答スタイル:** 日本語で簡潔に、変更点・検証結果・残リスクを明確に書く。
- **回答手順:** 先に前提と対象ファイルを確認し、実装後に実行コマンドと結果を報告する。
- **禁止事項・注意:** 未確認事項を断定しない。推測は推測と明記する。ユーザーが消した変更を勝手に戻さない。
- **秘匿情報の扱い:** 秘密情報、個人名、ローカルホームディレクトリ、private remote 情報はドキュメントや notebook 出力に含めない。

## 6. 試行タスク（オンボーディング演習）
> 小さな検証タスクを2〜3件記載してください。理解度を確認するために実施します。
1. `just native-smoke` を実行し、公式サンプルで平面・円柱候補が検出されることを確認する。
2. `just notebook` を実行し、`artifacts/cape_pyo3_demo.executed.ipynb` に個人情報や絶対パスが出力されていないことを確認する。
3. `tests/test_official_demo.py` を読み、`cape_demo.extract.extract_frame()` が PyO3 経路だけを使っていることを説明する。

## 7. 運用ルール・変更管理
- **ドキュメント更新時の記載ルール:** 変更した仕様・コマンド・検証結果は、根拠となるファイルパスや実行コマンドと合わせて記載する。
- **TBDの扱い:** 未確認の項目は `未確認` と書き、次に確認すべきファイル・担当・コマンドを併記する。
- **レビュー/承認フロー:** 大きな API 変更、アルゴリズム変更、データ差し替えはユーザー確認後に進める。
- **その他の運用ルール:** code/notebook/docs を変更したら、可能な範囲で `just verify` を実行してから commit/push する。

---

### 付録: 参考情報
- **主要リポジトリ/ディレクトリ:** `src/` Rust/PyO3 実装、`python/cape_demo/` Python 補助 API、`data/seq_example/` 公式サンプル、`notebooks/` demo notebook、`tests/` 検証資産、`artifacts/` 実行結果。
- **代表的なコマンド:** `uv sync --all-extras`, `uv run maturin develop --release`, `just test`, `just native-smoke`, `just notebook`, `just verify`。
- **依存ライブラリ:** Rust/Cargo、PyO3、maturin、uv、numpy、Pillow、matplotlib、pytest、nbconvert。
- **連絡先/責任者:** 未確認。リポジトリ管理者または作業依頼者に確認する。

> ※テンプレートは必要に応じて拡張・縮退して構いません。記入済みのドキュメントはバージョン管理してください。
