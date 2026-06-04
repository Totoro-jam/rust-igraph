# 总体规划

本项目的长期规划文档维护在仓库中，是所有开发决策的唯一权威来源。

查看完整的总体规划文档：
[MASTER_PLAN.md on GitHub](https://github.com/Totoro-jam/rust-igraph/blob/main/docs/plans/MASTER_PLAN.md)

## 核心目标

- 纯 Rust 移植 igraph C v1.0.x 的约 850 个公共 API
- 零 `unsafe`，零系统依赖（除 `thiserror`）
- 原生支持 WASM (`wasm32-unknown-unknown`)
- 通过三源验证保证数值正确性
