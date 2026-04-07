# memos-rs 执行计划

## 工作规则
1. 先读取 `codex-action/总指令.md`
2. 再读取 `codex-action/你应该给 Codex 的额外规则.md`
3. 再读取本文件
4. 如果存在 `codex-action/progress.md`，先读取它
5. 按下面的阶段顺序，找到第一个“未完成”的阶段
6. 只执行当前阶段
7. 完成后运行验证命令
8. 更新 `codex-action/progress.md`
9. 在本文件中将该阶段勾选为已完成
10. 若验证通过且没有 blocker，则继续执行下一个未完成阶段
11. 若遇到 blocker，则停止并在 `progress.md` 中写清楚原因

## 阶段顺序
- [ ] 第一阶段：项目骨架 + 健康检查 + CI
  - 文件：`codex-action/第一阶段：项目骨架 + 健康检查 + CI.md`

- [ ] 第二阶段：SQLite + migration + bootstrap admin
  - 文件：`codex-action/第二阶段：SQLite + migration + bootstrap admin.md`

- [ ] 第三阶段：认证
  - 文件：`codex-action/第三阶段：认证.md`

- [ ] 第四阶段：memo CRUD
  - 文件：`codex-action/第四阶段：memo CRUD.md`

- [ ] 第五阶段：静态前端托管
  - 文件：`codex-action/第五阶段：静态前端托管.md`

- [ ] 第六阶段：GitHub Actions 发布
  - 文件：`codex-action/第六阶段：GitHub Actions 发布.md`

- [ ] 第七阶段：systemd-部署打磨
  - 文件：`codex-action/第七阶段：systemd-部署打磨.md`

## 阶段完成判定
一个阶段只有在以下条件都满足时才算完成：
- 该阶段要求的代码/配置/文档已实现
- 项目仍可构建
- `cargo fmt --all --check` 通过
- `cargo clippy --all-targets --all-features -- -D warnings` 通过
- `cargo test` 通过
- 必要时 `cargo build --release` 通过
- `codex-action/progress.md` 已更新
- 本文件中的阶段复选框已被勾选

如果上述任一条件不满足，则继续修复当前阶段，不要进入下一阶段。

## 输出要求
每个阶段开始前：
- 说明当前阶段
- 说明预计修改哪些文件
- 说明如何验证成功

每个阶段结束后：
- 说明改了什么
- 说明跑了哪些命令
- 说明验证结果
- 说明是否进入下一阶段