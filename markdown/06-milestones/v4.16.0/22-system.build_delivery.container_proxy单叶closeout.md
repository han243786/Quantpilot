# v4.16.0 system.build_delivery.container_proxy 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 基准: `12-system十叶模块等价基线.md`、`19-system.build_delivery.desktop_build_scripts单叶closeout.md`。
> 执行档位: 标准。
> 判定: S8 `system.build_delivery.container_proxy` 完成静态白箱 closeout；容器与代理配置已登记，不作为当前桌面默认路径，不改 Docker/nginx 语义。Docker runtime smoke 只有在开发者明确决定进入版本发布或发布验收时才执行。

---

## 目标

本文件确认容器构建与反向代理配置的当前边界。

本批次只登记:

1. `Dockerfile` 的后端构建、前端构建和 runtime stage。
2. `docker-compose.yml` 的 backend 与 frontend-dev service。
3. `nginx.conf` 的 80 -> 443 redirect 和 443 TLS proxy。
4. 本批次不运行容器、不改变端口、不改变镜像构建阶段、不改变 nginx 代理路径。

门禁标记: `Docker runtime smoke requires developer release decision`。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 system 单叶 closeout、S8 完成判定 | 落地 |
| 规范矩阵 | 容器构建 owner、反向代理边界、桌面默认路径隔离 | 扩展 |
| 引导矩阵 | 全量树、模块树、真实文件、container/proxy 门禁坐标 | 扩展 |
| 模块树 | `system.build_delivery.container_proxy` | 完成 S8 静态基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.build_delivery.container_proxy` |
| 真实文件 | `Dockerfile`、`docker-compose.yml`、`nginx.conf` |
| public 方法 | Docker build context、compose services、nginx proxy config |
| 关键内部实现 | Rust backend release build、Node frontend build、runtime image、3000 backend port、5173 frontend-dev profile、443 TLS proxy、80 redirect |
| 测试/门禁 | Docker/compose static review、proxy route 对照、发布验收触发的 `docker compose config` / runtime smoke、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 静态等价验证证据

| 核查项 | 结果 | 证据 |
| --- | :--: | --- |
| Docker backend build stage | 通过 | `FROM rust:1.85-slim-bookworm`、`cargo build --release`、复制 `target/release/quantpilot` |
| Docker frontend build stage | 通过 | `FROM node:22-alpine`、`npm ci`、`npm run build` |
| Docker runtime stage | 通过 | `debian:bookworm-slim`、`QUANTPILOT_DEV=false`、`QUANTPILOT_BIND_ADDR=0.0.0.0`、`EXPOSE 3000`、healthcheck `/api/health` |
| compose backend service | 通过 | `backend` service build 当前目录，映射 `3000:3000`，挂载 `storage` 与 `config` |
| compose frontend-dev service | 通过 | `frontend-dev` 使用 `node:22-alpine`，映射 `5173:5173`，依赖 `backend`，归入 `dev` profile |
| nginx proxy | 通过 | 443 TLS server proxy 到 `http://quantpilot:3000`，80 server redirect 到 HTTPS |
| runtime compose config | 未执行 | 本批次不是版本发布或发布验收；本机未安装 Docker CLI，`docker compose config` 无法作为本批次证据 |

---

## 白箱 closeout 判定

| 项 | 判定 | 说明 |
| --- | --- | --- |
| public 入口 | 完成 | Docker build、compose services、nginx proxy config 已登记 |
| 兼容入口 | 完成 | 文件名、服务名、端口和 proxy 路径不变 |
| 静态边界 | 完成 | Dockerfile、compose、nginx config 已静态核查 |
| 外部边界 | 完成 | 不拥有桌面默认运行路径、后端 handler、前端 route 或 release workflow |
| runtime 证据 | 暂缺 | 本机无 Docker CLI；后续只有在开发者明确决定版本发布/发布验收，或明确重新打开 S8 runtime 验收时，才补 Docker runtime smoke |
| 继续细分 | 停止 | 当前只有容器构建与代理配置，继续拆会变成配置字段级文档 |

---

## 父子通信规则

`system.build_delivery.container_proxy` 只能经 `system.build_delivery` 提供容器构建和代理配置。它不得直接拥有 `system.entry.launch_scripts`、`system.desktop_shell`、后端 API handler、前端路由、CI/release workflow 或发布版本过渡决策。

后续如果改变暴露端口、服务名、镜像阶段、volume、环境变量、nginx upstream 或 TLS 规则，必须重新打开 S8。Docker runtime smoke 不由 AI 主动触发；只有开发者明确决定进入版本发布/发布验收，或明确要求重新打开 S8 runtime 验收时，才补充 `docker compose config` 或容器 smoke 证据。

---

## 不继续细分理由

| 候选子叶 | 不继续拆的原因 |
| --- | --- |
| Dockerfile build stages | 当前是单一镜像构建链，未拆成独立发布 owner |
| compose services | 只服务容器本地编排，不是桌面默认运行路径 |
| nginx proxy | 单一代理配置，未接入当前桌面 smoke |

---

## 禁止事项

- 不把容器路径宣称为当前桌面默认运行路径。
- 不在没有 Docker runtime 证据时宣称容器启动已验证。
- 不在开发者未明确决定版本发布/发布验收时主动要求 Docker runtime smoke。
- 不改 `3000`、`5173`、443、80 或 nginx upstream。
- 不把 container proxy 变更混入 CI/release 或 desktop build scripts。
- 不主动提出发布版本过渡或横向连接。

---

## 验收标准

1. S8 的真实文件、public 入口、关键内部实现和父级通信规则已登记。
2. Dockerfile、compose 和 nginx 关键路径已静态核查。
3. 文档明确 Docker runtime smoke 只能由开发者版本发布/发布验收决策触发；当前未执行 runtime compose config。
4. `system.build_delivery.container_proxy` 模块树节点标记为静态单叶 closeout 完成。
5. 本批次不改容器或代理配置，不继续细分，不进入整理或重构。
