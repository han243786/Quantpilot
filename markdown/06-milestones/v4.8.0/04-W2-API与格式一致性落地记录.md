# v4.8.0 W2 API 与格式一致性落地记录

> 日期: 2026-05-25
> 范围: W2-1 至 W2-4

## 落地结果

| 项 | 已落地内容 | 回归命令 |
|---|---|---|
| W2-1 | auth register/login/refresh 错误响应统一为 `error/error_code/message/details` | `cargo test --test api_auth` |
| W2-2 | executor SSE 改为命名 `event:` 字段; v4 证据面板和策略图面板同步读取命名事件, 同时保留旧 `payload.type` fallback | `rg -n "Event::default\\(\\)\\.data\\(.*\\\"type\\\"" src-executor/main.rs` |
| W2-3 | 主后端部署入口从 `/api/executor/deploy` 对齐为 `/api/executor/strategies`; 执行端错误提示同步 | `rg -n "executor/deploy" src src-executor contracts` |
| W2-4 | CHANGELOG 拆出 `## v4.5.0` 与 `## v4.6.0` 独立章节 | `rg -n "^## v4\\.(5|6)\\.0" CHANGELOG.md` |

## 边界说明

- W2 只统一 API 与事件格式, 不新增真实资金执行能力。
- SSE 仍保留 `onmessage` fallback, 避免旧 payload 型事件消费者立即失效。
- `/api/executor/strategies` 是 OpenAPI 与执行端现有路径的统一口径。
