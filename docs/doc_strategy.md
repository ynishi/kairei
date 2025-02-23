# KAIREIドキュメント戦略

## 概要
KAIREIのドキュメントは以下の2つの主要な場所で管理されます：

1. RustDoc: 「今のプロダクトがどうなっているか」
   - 実装と密接に結びついた技術仕様
   - APIドキュメント
   - コンポーネントの設計と構造

2. docs/: 「なぜそうなったのか、どう進めているのか」
   - 開発プロセス
   - 設計判断の記録
   - チュートリアルやユースケース
   - 図表やビジュアル資料

## RustDocの構成

### lib.rs - プロジェクト全体像
```rust
//! # KAIREI
//! 
//! AIエージェント実行基盤（AI Agent Orchestration Platform）
//! 
//! ## Architecture Overview
//! KAIREIは、LLMを活用したAIエージェントの実行環境を提供します...
//! 
//! ## Core Design
//! イベント駆動型アーキテクチャとMicroAgentモデルを採用し...
```

### dsl/mod.rs - DSL仕様
```rust
//! # KAIREI DSL
//! 
//! KAIREIのDSLは以下の主要コンポーネントで構成されています：
//! - World定義
//! - MicroAgent定義
//! - think構文
```

### agent/mod.rs - エージェント設計
```rust
//! # MicroAgent
//! 
//! MicroAgentは単一責務の原則に基づく実行単位で...
```

## docs/の構成
- design/: アーキテクチャと設計ドキュメント
- process/: 開発プロセスとワークフロー
- tutorials/: チュートリアルとガイド
- assets/: 図表とビジュアル資料

## 相互参照
- RustDocからdocs/への参照
  - 設計背景や詳細な説明が必要な場合
  - 図表やビジュアル資料の参照

- docs/からRustDocへの参照
  - API仕様やインターフェースの説明
  - 実装詳細の参照

## メンテナンス
- RustDoc: コードレビュー時に更新確認
- docs/: 設計変更や新機能追加時に更新
- 定期的なドキュメントレビューの実施
