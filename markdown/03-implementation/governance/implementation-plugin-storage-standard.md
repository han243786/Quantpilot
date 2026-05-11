# 插件存储与存放管理标准

> v1.0.0 | 所有插件必须遵守本标准的目录结构、命名规范和生命周期规则

---

## 一、目录结构

```
plugins/
├── builtin/                  # 内置插件 (随 QuantPilot 发布, 只读, 不可删除)
│   ├── data/
│   │   └── kline/
│   │       └── plugin.json
│   ├── intent/
│   │   ├── ma_cross/
│   │   │   └── plugin.json
│   │   ├── rsi/
│   │   │   └── plugin.json
│   │   └── ...
│   ├── agent/
│   │   └── weighted/
│   │       └── plugin.json
│   ├── risk/
│   │   └── global/
│   │       └── plugin.json
│   └── execution/
│       └── paper/
│           └── plugin.json
│
├── installed/                # 用户安装的第三方插件 (可增删)
│   └── {plugin-id}/
│       ├── plugin.json       # Manifest (必填, 文件名固定)
│       ├── README.md         # 插件说明 (可选)
│       └── LICENSE           # 许可证 (可选)
│
├── suites/                   # 套件 (纯打包, 不含逻辑代码)
│   └── {suite-id}/
│       └── plugin.json
│
├── cache/                    # 插件市场索引缓存
│   └── market-index.json     # 最近一次 fetch_index() 的缓存
│
└── disabled/                 # 已禁用但未删除的插件
    └── {plugin-id}/
        └── plugin.json
```

## 二、基本规则

### 2.1 一插件一目录

每个插件独占一个以 `plugin_id` 命名的子目录。`plugin_id` 必须是合法的文件系统目录名：仅允许 ASCII 小写字母、数字、下划线、连字符和点号。正则: `^[a-z0-9._-]+$`。

### 2.2 Manifest 文件名固定为 `plugin.json`

无论插件类型 (Atom/Suite)、无论来源 (内置/用户安装)，Manifest 文件名统一为 `plugin.json`。注册表通过目录结构区分插件，不通过文件名区分。

### 2.3 内置与用户安装严格分离

| 特征 | `builtin/` | `installed/` |
|------|-----------|-------------|
| 随版本发布 | ✅ | ❌ |
| 用户可删除 | ❌ | ✅ |
| 用户可修改 | ❌ | ✅ |
| 启动时自动注册 | ✅ | ✅ |
| 版本升级 | 随 QuantPilot 升级 | 用户手动或市场拉取 |
| 安全审计 | QuantPilot 维护者负责 | 用户自行承担 |

### 2.4 套件独立存放

套件是纯打包层，不含策略逻辑。套件存放在 `suites/` 目录，与原子插件分离。套件的 `plugin.json` 中 `plugin_type` 字段为 `"suite"`，`atoms` 字段列出引用的原子 ID。

### 2.5 禁用机制

用户可通过将插件目录从 `installed/` 移动到 `disabled/` 来禁用插件。`disabled/` 中的插件不会被注册表扫描加载。用户可随时将其移回 `installed/` 以重新启用。

已禁用的插件如果被某个套件引用，套件加载时校验失败。

## 三、Manifest 规范

### 3.1 文件名与编码

- 文件名: `plugin.json`
- 编码: UTF-8 (无 BOM)
- 格式: JSON (严格模式, 不允许注释和尾逗号)

### 3.2 必填字段

```json
{
  "api_version": "quantpilot/plugin-manifest/v1",
  "id": "custom.atr_v2",
  "version": "0.1.0",
  "kind": "intent",
  "display": {
    "name": "ATR v2",
    "summary": "改进版真实波幅, 支持 Wilder 和 EMA 平滑"
  },
  "capability_declarations": [
    {
      "id": "qrpc.intent_module_provider",
      "version": "v1"
    }
  ],
  "extension_points": ["intent_module_provider"],
  "execution": {
    "engine": "native",
    "entrypoint": "atr_v2.dll"
  },
  "compatibility": {
    "core_ir_version": "quantpilot/core-ir/v1",
    "capability_api_version": "quantpilot-capabilities/v1"
  },
  "security": {
    "max_compute_ms": 100,
    "max_memory_mb": 64,
    "allow_network": false
  }
}
```

### 3.3 Atom 扩展字段

Atom 类型 (`plugin_type: "atom"` 或未设置) 可选声明:

```json
{
  "plugin_type": "atom",
  "hot_handoff": false,
  "asset_management": false
}
```

### 3.4 Suite 扩展字段

Suite 类型 (`plugin_type: "suite"`) 必须声明:

```json
{
  "plugin_type": "suite",
  "atoms": [
    { "atom_id": "builtin.data.kline", "version": "0.1.0", "kind": "data" },
    { "atom_id": "builtin.intent.rsi",   "version": "0.1.0", "kind": "intent" }
  ],
  "hot_handoff": false,
  "asset_management": false
}
```

Suite 不声明 `extension_points` 和 `capability_declarations`——这些由组成它的 Atom 各自声明。

## 四、生命周期管理

### 4.1 安装

**方式一: 手动安装**

1. 在 `plugins/installed/` 下创建以 `plugin_id` 命名的目录
2. 将 `plugin.json` 放入该目录
3. 重启 QuantPilot 或调用 `RuntimePluginRegistry::scan_atoms()` 触发重新扫描
4. 验证: `GET /api/capabilities` 返回的 `strategy_ir.indicator_support` 中应出现新插件

**方式二: 市场安装**

1. 通过 `PluginMarketClient::fetch_index()` 浏览可用插件
2. 调用 `PluginMarketClient::fetch_manifest()` 下载并校验 manifest
3. 自动写入 `plugins/installed/{plugin_id}/plugin.json`
4. 自动注册到 `RuntimePluginRegistry`

### 4.2 升级

1. 备份当前插件目录: `mv plugins/installed/{id} plugins/installed/{id}.bak`
2. 安装新版本到 `plugins/installed/{id}/`
3. 重启。如果新版本校验失败，恢复备份
4. 如果该插件被套件引用，套件校验时会检查版本兼容性

### 4.3 禁用

1. 将插件目录移动到 `plugins/disabled/`: 
   ```
   mv plugins/installed/{id} plugins/disabled/{id}
   ```
2. 重启。注册表不再加载该插件
3. 任何引用该插件的套件在 `validate_suite()` 时报错

### 4.4 删除

1. 删除插件目录: `rm -rf plugins/installed/{id}`
2. 如果该插件被套件引用，必须先更新或删除套件
3. 内置插件 (`builtin/`) 不可删除

### 4.5 启用已禁用插件

1. 将插件目录移回: `mv plugins/disabled/{id} plugins/installed/{id}`
2. 重启

## 五、发现与注册流程

启动时，`RuntimePluginRegistry` 按以下顺序扫描:

```
1. 扫描 plugins/builtin/   → 按 kind 分组注册
     data/     → DataModuleProvider
     intent/   → IntentModuleProvider
     agent/    → AgentModuleProvider
     risk/     → RiskCheckerProvider
     execution/ → ExecutionModuleProvider

2. 扫描 plugins/installed/ → 逐目录注册
     每个子目录:
       - 读取 plugin.json
       - PluginManifest::validate()
       - 根据 kind 注册到对应 provider Map
       - 记录 lifecycle: Registered

3. 扫描 plugins/suites/    → 校验但不立即展开
     每个套件:
       - 读取 plugin.json
       - validate_suite() (所有 atom 已注册)
       - 记录 lifecycle: Registered

4. plugins/disabled/       → 跳过 (不扫描)
```

## 六、校验规则

### 6.1 目录级校验

- 子目录名必须与 `plugin.json` 中的 `id` 字段匹配
- 不匹配时注册表拒绝加载，记录错误日志

### 6.2 Manifest 校验

- 使用 `PluginManifest::validate()` (详见 RFC-020)
- 必须通过所有必填字段检查
- api_version 必须为 `"quantpilot/plugin-manifest/v1"`

### 6.3 冲突检测

- 如果 `installed/` 中有与 `builtin/` 相同 `id` 的插件，`installed/` 优先 (用户覆盖)
- 如果 `installed/` 中有重复 `id`，后扫描的覆盖先扫描的
- 冲突情况记录 warning 日志

### 6.4 Suite 完整性校验

- Suite 引用的每个 atom 必须在 `builtin/` 或 `installed/` 中已注册
- Suite 引用的 atom 版本必须兼容 (semver 主版本号一致)
- Suite 中的 atom 如果有 exchange/symbol 声明，必须全部一致

## 七、文件系统权限

| 目录 | 用户权限 | QuantPilot 权限 |
|------|:--:|:--:|
| `builtin/` | 只读 | 只读 |
| `installed/` | 读写 | 只读 (启动时) / 读写 (市场安装时) |
| `suites/` | 读写 | 只读 (启动时) |
| `disabled/` | 读写 | 不访问 |
| `cache/` | 读写 | 读写 |

## 八、插件 ID 命名规范

```
{namespace}.{category}.{name}

示例:
  builtin.data.kline        — 内置 K 线数据模块
  builtin.intent.rsi        — 内置 RSI 指标
  custom.atr_v2             — 用户自定义 ATR v2
  suite.okx_btc_ma_trend    — 套件: OKX BTC 均线趋势
  community.ml.lstm_predict — 社区: LSTM 预测
```

命名空间:
- `builtin` — QuantPilot 官方内置
- `custom` — 用户自行开发
- `suite` — 套件
- `community` — 社区贡献 (未来)

## 九、与存储生命周期的关系

插件文件分类为 **Permanent** (长期存储):

| 目录 | 存储生命周期 | TTL | 清理触发 |
|------|:--:|:--:|------|
| `plugins/builtin/` | Permanent | 无上限 | 随版本升级替换 |
| `plugins/installed/` | Permanent | 无上限 | 仅用户显式删除 |
| `plugins/suites/` | Permanent | 无上限 | 仅用户显式删除 |
| `plugins/cache/` | Temporary | 7 天 | 启动清理 |
| `plugins/disabled/` | Permanent | 无上限 | 仅用户显式删除 |

插件目录不受 `storage/` 全局 500MB 配额限制 (Permanent 豁免)，但应在 UI 中展示占用空间供用户参考。

## 十、迁移与兼容性

### 10.1 版本迁移

插件升级时，如果新版本与旧版本的 `capability_declarations` 不兼容:

1. 注册表将旧版本标记为 `Stopped`
2. 新版本注册为 `Registered`
3. 如果旧版本有活跃运行引用，运行继续使用旧版本直到完成
4. 新策略使用新版本

### 10.2 回滚

如果新版本插件导致运行错误:

1. 将新版本目录移到 `disabled/`
2. 将备份的旧版本恢复到 `installed/`
3. 重启
4. 注册表自动加载旧版本

### 10.3 QuantPilot 升级

QuantPilot 主版本升级时:

1. `builtin/` 目录随安装包整体替换
2. `installed/` 和 `suites/` 中的用户插件保留不变
3. 启动时校验所有用户插件的 `compatibility.core_ir_version` 是否与新版本兼容
4. 不兼容的插件自动标记为 `Faulted`，不会激活
