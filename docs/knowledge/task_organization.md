# タスク組織化と範囲拡大検知

## 概要
複数コンポーネントへの変更やスコープ拡大を検知し、タスクの分割を提案する機能。

## 検知対象
- 複数コンポーネントへの同時変更
  - tokenizer
  - analyzer/parser
  - type_checker
  - その他コアコンポーネント
- 設計ドキュメント参照の確認

## 動作
1. PRの変更を分析:
   - 影響を受けるコンポーネントの特定
   - docs/design参照の確認
   - 変更量の分析

2. 複数コンポーネント変更時:
   - タスク分割の提案
   - ナレッジベースへの記録
   - PRへのラベル付与とコメント

## 推奨プラクティス
1. コンポーネントごとに独立したPRを作成
2. 設計ドキュメントの参照を明示
3. 依存関係の明確化

## 実装詳細
- `.github/workflows/task-organization.yml`で実装
- `.github/workflows/scope-expansion-detection.yml`で実装
- GitHub Actionsのpull_requestトリガーを使用
