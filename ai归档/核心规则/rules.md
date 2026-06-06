# new-SkillControl 核心规则与架构矩阵

## 1. 核心红线 (Core Rules)
- **前后端依赖**：前端必须保持为纯 Vanilla JS + CSS + HTML，禁止引入任何重量级 UI 框架或 TailwindCSS（除非用户显式要求）。
- **反序列化兼容**：若在 Rust 后端 `SkillStatus` 结构体中添加新字段，**必须**加 `#[serde(default)]` 属性，保证向下兼容旧 `config.json`。
- **冷启动防假死**：Git 同步与长耗时 I/O 必须通过 `tokio::spawn` 异步非阻塞执行，UI 层使用防抖 + RequestID 锁，严禁同步 await 阻塞主渲染线程。
- **同名技能隔离**：对于多仓库同名 ID 的技能，利用配置中 `skills_status` 记录的 `repo_id`（及文件内容对比）来区分并判断已下载/已安装状态。

## 2. API 路由速查 (API & Tauri Commands)
- `discover_all_skills`: 读取并同步所有技能，采用 OnceLock `SKILLS_CACHE` 内存缓存优化加载。
- `auto_update_and_cleanup_repo`: 拉取远程库，自动物理清理 staging、`.gemini`、`.reasonix` 和配置中不再存在的孤儿技能。
- `get_skill_diff`: 计算 staging 技能文件与仓库文件的 LCS 差异并按行返回比对结果。
- `toggle_skill_switch`: 触发技能分发状态（agy/agy2/reasonix）开关更新。

## 3. 环境与配置矩阵 (Environment & Configs)
- **配置文件**：`%APPDATA%/new-SkillControl/config.json` (或 `%APPDATA%/new-SkillControl/`)。
- **凭据管理**：通过 GCM (Git Credential Manager) 管理 GitHub Token，使用 `git credential fill` 提取凭据以安全地调用 GitHub API。
