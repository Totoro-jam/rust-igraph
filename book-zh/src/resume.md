# 中断后恢复

当你在一段时间后回到项目时，按以下步骤快速恢复上下文：

1. **检查当前状态**：`git status` + `git log --oneline -10`
2. **查看追踪器**：打开 `.codefuse/tracking/ALGORITHMS.md` 查看进度
3. **运行测试**：`cargo test --workspace` 确认一切正常
4. **查看规划**：阅读 `docs/plans/MASTER_PLAN.md` 了解下一步

详情参阅 [Resume (English)](/rust-igraph/book/resume.html)
