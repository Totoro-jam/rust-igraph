# 开发笔记

## 快速命令

```bash
# 快速循环
cargo build --workspace
cargo test  --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# 完整循环 (oracle + proptest + conformance)
cargo test  --workspace --all-features

# WASM 检查
cargo check --target wasm32-unknown-unknown
```

## AWU 工作流

每个算法通过 9 步 SOP 完成：

1. Recon — 阅读 igraph C 源码
2. Interface — 设计 Rust API 签名
3. Skeleton — 创建模板文件
4. Implementation — 翻译算法逻辑
5. Unit tests — 单元测试 + proptest
6. Conformance — 三源一致性测试
7. Oracle — python-igraph 验证
8. Benchmark — criterion 基准测试
9. Documentation — rustdoc + doctest

详情参阅
[MASTER_PLAN.md §4](https://github.com/Totoro-jam/rust-igraph/blob/main/docs/plans/MASTER_PLAN.md)
