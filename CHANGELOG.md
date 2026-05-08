# Changelog

## 0.4.1

安全审计全量修复。12 项发现中 11 项已修复，1 项已文档化。

### 安全修复
- credential_vault: SecretString wrapper (Drop 时 Zeroize) / 原子写入 (tmp + rename + bak) / AAD 绑定 / OnceLock 去竞态 / 删除静默密钥降级
- CLI: 交互式 stdin 输入替代命令行参数传递凭证明文
- 日志: 动态注册 vault 字段值到脱敏模块 / while let 全量替换
- 环境变量: 删除 set_var("HTTPS_PROXY") 全局副作用
- 前端: 凭证面板关闭时清零 React state

### 新增
- /api/credentials 路由: GET list / POST set / DELETE delete
- CredentialInput 组件: 动态字段渲染 / OkxCredentialInput 预设
- TopToolbar 凭证管理面板
- storage/.machine_key 随机机器密钥

### 文档
- api安全方案.md: 本地凭证存储设计 + 安全边界与已知限制
- v0.4.1/01-03 三个里程碑文档

## 0.4.0

审计驱动收口。三条工作线 10 项全量完成。

### UI 简洁化
- 字段裁剪: 面板默认 ≤4 字段 + 折叠展开
- 面板重整: ResearchConsole + EventStreamPanel 去重叠
- 仪表盘布局: 默认首页替代 6 标签页
- CSS 变量统一

### 引导教程
- TutorialOverlay: 5 步分步引导 + 前进/后退/退出
- createTutorialSteps(t) i18n 支持

### 凭证安全
- CredentialVault: AES-256-GCM 加密本地凭证文件
- CredentialInput 组件: type=password / autocomplete=off
- Zeroizing 内存保护 / safe_log 日志脱敏

## 0.3.0

全量合规优化与基础设施补强。22 项 P0-P3 + 10 个新信号 + I-1/I-2 双路径合并。

## 0.2.0

测试自动化全链路。@test/@step/@assert 指令 + TestRunner + 前端测试桥 + Playwright + CI。

## 0.1.0

初始私有基线。Paper 运行时沙箱 / 图编辑器 / QS 编译管道 / 6 类 K 线 Intent / 回测。
