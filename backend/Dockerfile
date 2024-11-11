# Stage 1: Build backend
# syntax=docker/dockerfile:1.4
FROM rust:latest AS backend-builder
WORKDIR /app

# 复制项目文件
COPY . .

# 确保安装编译所需的依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    pkg-config \
    libssl-dev

# 使用 cargo 构建项目
RUN cargo build --release

# 创建最终镜像
FROM debian:bookworm

# 将后端可执行文件复制到最终镜像中
COPY --from=backend-builder /app/target/release/backend /usr/local/bin/bitcoin-explorer

# 安装运行时所需的依赖项
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libgcc-s1

# 暴露后端的端口
EXPOSE 8080

# 启动后端程序
CMD ["bitcoin-explorer"]
