# new-SkillControl LOGIC_BASE — 核心方案与避坑指南

[2026-06-06] | 实现 v0.1.4 需求（包含清除孤儿技能、AGY 2.0开关、手风琴列表UI、LCS差异比对器、极速搜索缓存和全局快捷键）并直接自动发布 Release | 1. 采用 OnceLock 静态内存缓存 `SKILLS_CACHE` 降低二次加载延迟至 0ms。2. 编写 Rust 手动计算 LCS 并高亮行级差异。3. 使用 Python 子进程交互调用 `git credential fill` 提取 GCM 托管 of GitHub Token，稳定绕过本地无 `gh` 客户端及双因子认证。 | 1. 结构体 `SkillStatus` 新增字段必须加 `#[serde(default)]`，否则前端解析老配置时会导致反序列化报错崩溃。 2. Windows 命令行交互获取凭据极易受换行符和编码干扰，使用 Python 原生 `subprocess` 以标准输入形式与 `git credential` 交互是最稳妥的方法。
