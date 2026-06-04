# 架构决策

架构决策记录 (ADR) 记录了 rust-igraph 的关键技术选型和约束。

查看完整文档：
[ARCHITECTURE.md on GitHub](https://github.com/Totoro-jam/rust-igraph/blob/main/.codefuse/tracking/ARCHITECTURE.md)

## 核心约束

1. **禁止 `unsafe`**——除非有明确的 ADR 批准
2. **禁止 `unwrap()` / `expect()`**——非测试代码中不允许
3. **禁止新依赖**——除非 ARCHITECTURE.md 明确批准
4. **浮点比较**——必须使用容差辅助函数
5. **整数运算**——使用 `checked_*` / `try_from` 防止溢出
