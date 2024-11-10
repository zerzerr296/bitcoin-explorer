# Stage 1: Build backend
# syntax=docker/dockerfile:1.4
FROM rust:latest AS backend-builder
WORKDIR /app
COPY . .
RUN cargo build --release

RUN apt-get update && apt-get install -y ca-certificates

# Stage 2: Final image
FROM debian:bookworm

# 安装必要的依赖项
RUN apt-get update && apt-get install -y libssl-dev

# 将后端可执行文件复制到系统路径
COPY --from=backend-builder /app/target/release/bitcoin-explorer /usr/local/bin/bitcoin-explorer

# 暴露后端的端口
EXPOSE 8080

# 启动后端程序
CMD ["bitcoin-explorer"]
