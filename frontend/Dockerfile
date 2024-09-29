# Stage 1: Build the front-end application
FROM node:18 AS frontend-builder

# 设置工作目录
WORKDIR /app

# 复制 package.json 和 package-lock.json
COPY package*.json ./

# 安装依赖
RUN npm install

# 复制源代码
COPY . .

# 构建应用
RUN npm run build

# Stage 2: Serve with NGINX
FROM nginx:latest

# 将前端构建产物复制到 NGINX 的默认静态资源路径
COPY --from=frontend-builder /app/build /usr/share/nginx/html

# 替换默认的 NGINX 配置文件
COPY nginx.conf /etc/nginx/nginx.conf
