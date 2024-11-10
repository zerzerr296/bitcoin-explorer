# Stage 1: Build backend
# syntax=docker/dockerfile:1.4
FROM rust:latest AS backend-builder-stage  # 修改了阶段的名称
WORKDIR /app
COPY . .
RUN cargo build --release

# 安装 ca-certificates 以支持安全连接
RUN apt-get update && apt-get install -y ca-certificates

# Stage 2: Final image
FROM debian:bookworm

# 安装必要的依赖项
RUN apt-get update && apt-get install -y libssl-dev

# 确保在 `backend-builder-stage` 阶段构建的二进制文件存在
# 复制最终的二进制文件到目标镜像中
COPY --from=backend-builder-stage /app/target/release/bitcoin-explorer /usr/local/bin/bitcoin-explorer

# 暴露后端的端口
EXPOSE 8080

# 启动后端程序
CMD ["bitcoin-explorer"]
