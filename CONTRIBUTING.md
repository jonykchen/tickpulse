# 贡献指南

感谢您考虑为 TickPulse 做出贡献！

## 目录

- [行为准则](#行为准则)
- [如何贡献](#如何贡献)
- [开发指南](#开发指南)
- [提交规范](#提交规范)
- [Pull Request 流程](#pull-request-流程)

---

## 行为准则

本项目采用贡献者公约作为行为准则。参与此项目即表示您同意遵守其条款。请阅读 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) 了解详情。

---

## 如何贡献

### 报告 Bug

如果您发现了 bug，请通过 [GitHub Issues](https://github.com/jonykchen/tickpulse/issues) 提交报告。

提交 Bug 报告时，请包含：

1. **问题描述**: 清晰简洁地描述问题
2. **复现步骤**: 详细说明如何复现问题
3. **预期行为**: 描述您期望发生的情况
4. **实际行为**: 描述实际发生的情况
5. **环境信息**:
   - 操作系统及版本
   - 应用版本
   - Node.js 版本
   - Rust 版本
6. **截图**: 如果适用，添加截图帮助解释问题

### 建议新功能

我们欢迎新功能建议！请通过 [GitHub Issues](https://github.com/jonykchen/tickpulse/issues) 提交，并包含：

1. **功能描述**: 清晰描述您希望添加的功能
2. **使用场景**: 说明该功能解决什么问题
3. **实现思路**: 如果有想法，可以简要说明实现方案

---

## 开发指南

### 环境准备

请参考 [README.md](README.md#快速开始) 设置开发环境。

### 代码风格

#### Rust

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 遵循 [Rust API 指南](https://rust-lang.github.io/api-guidelines/)

#### TypeScript/Vue

- 使用 ESLint + Prettier 格式化代码
- 遵循 Vue 3 Composition API 风格
- 使用 `<script setup>` 语法

### 项目结构

```
tickpulse/
├── src/                    # Vue 前端源码
│   ├── views/              # 路由视图
│   ├── components/         # Vue 组件
│   ├── stores/             # Pinia 状态管理
│   ├── lib/                # 工具函数
│   └── types/              # TypeScript 类型
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── market/         # 行情模块
│   │   ├── analysis/       # AI 分析引擎
│   │   ├── db/             # 数据库层
│   │   ├── alert/          # 预警系统
│   │   ├── anomaly/        # 异动检测
│   │   └── system/         # 系统功能
│   └── Cargo.toml
└── doc/                    # 文档
```

### 添加新功能

1. **Tauri 命令**: 在 `src-tauri/src/lib.rs` 中定义命令
2. **前端调用**: 在 `src/lib/tauri.ts` 中封装 API
3. **类型定义**: 在 `src/types/` 中添加 TypeScript 类型
4. **UI 组件**: 在 `src/components/` 中实现界面

### 测试

```bash
# Rust 测试
cd src-tauri
cargo test

# 前端类型检查
npm run build
```

---

## 提交规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/zh-hans/v1.0.0/) 规范：

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### 类型 (type)

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具变更

### 示例

```
feat(market): 添加北向资金流向图表

- 新增 NorthFlowChart 组件
- 集成东方财富北向资金 API
- 支持日/周/月维度切换

Closes #123
```

---

## Pull Request 流程

1. **Fork 项目** 并克隆到本地

2. **创建分支**
   ```bash
   git checkout -b feat/your-feature-name
   ```

3. **编写代码** 并确保通过测试

4. **提交变更**
   ```bash
   git add .
   git commit -m "feat(scope): 描述"
   ```

5. **推送到 Fork**
   ```bash
   git push origin feat/your-feature-name
   ```

6. **创建 Pull Request**
   - 填写 PR 模板
   - 关联相关 Issue
   - 等待代码审查

### PR 检查清单

- [ ] 代码通过 `cargo clippy` 检查
- [ ] 代码通过 `cargo fmt` 格式化
- [ ] 前端代码通过类型检查
- [ ] 提交信息符合规范
- [ ] 更新相关文档（如有必要）

---

## 获取帮助

如果您在贡献过程中遇到问题，可以：

- 查看 [文档](doc/QUICKSTART.md)
- 在 [Discussions](https://github.com/jonykchen/tickpulse/discussions) 中提问
- 加入社区交流群（如有）

再次感谢您的贡献！
