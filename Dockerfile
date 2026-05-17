# ── QuantPilot v0.2.0 多阶段构建 ──
# Stage 1: Rust 后端构建
FROM rust:1.85-slim-bookworm AS backend-builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY qrpc_core/ ./qrpc_core/
COPY qrpc_core_ir/ ./qrpc_core_ir/
COPY qrpc_compiler/ ./qrpc_compiler/
COPY qrpc_runtime/ ./qrpc_runtime/
COPY quantscript/ ./quantscript/
COPY src/ ./src/
RUN cargo build --release && \
    cp target/release/quantpilot /app/quantpilot

# Stage 2: Node 前端构建
FROM node:22-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 3: 运行时
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend-builder /app/quantpilot /app/quantpilot
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist
COPY config/ /app/config/
RUN mkdir -p /app/storage
# v2.3.1: 非 root 用户运行
RUN useradd -m -u 1000 quantpilot && chown -R quantpilot:quantpilot /app
USER quantpilot

ENV QUANTPILOT_DEV=false
ENV QUANTPILOT_RATE_LIMIT_RPS=100
EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --retries=3 CMD curl -f http://localhost:3000/api/health || exit 1

CMD ["/app/quantpilot"]
