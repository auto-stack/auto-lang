# Website 部署指南

本文档说明如何将 `website/` 目录（VitePress 站点）构建并部署到远程云服务器，通过 Nginx 在 80 端口提供静态文件服务。

## 为什么使用 Nginx？

- **Vite / VitePress 的 `preview` 模式仅供临时预览**，关闭终端即停止，不适合生产环境。
- **Nginx** 是专业的静态资源服务器，支持持久运行、并发处理、gzip 压缩、缓存策略以及后续 HTTPS（443 端口）扩展。

## 前置条件

- 一台可通过 SSH 访问的 Linux 云服务器（示例：`visus@112.74.45.241`）
- 服务器已开放 80 端口（安全组/防火墙）
- 本地已安装 Node.js、npm、PowerShell（Windows）或 Bash（Linux/macOS）

## 服务器端：安装与配置 Nginx

SSH 登录服务器后执行以下步骤（仅需执行一次）：

### 1. 安装 Nginx

```bash
sudo apt update && sudo apt install -y nginx
```

### 2. 创建站点配置

创建文件 `/etc/nginx/sites-available/auto-website`：

```nginx
server {
    listen 80;
    server_name 112.74.45.241;  # 如有域名请替换

    root /home/visus/auto-website;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # 静态资源缓存
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
        expires 1M;
        add_header Cache-Control "public, immutable";
    }

    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml;
}
```

> **注意**：将 `server_name` 和 `root` 路径替换为你的实际 IP/域名和服务器用户名。

### 3. 启用配置

```bash
sudo ln -sf /etc/nginx/sites-available/auto-website /etc/nginx/sites-enabled/
sudo rm -f /etc/nginx/sites-enabled/default
sudo nginx -t && sudo systemctl restart nginx
```

### 4. 确保家目录可被 Nginx 访问

Nginx worker 进程通常以 `www-data` 用户运行，需要能够进入你的家目录：

```bash
chmod 755 /home/visus
mkdir -p /home/visus/auto-website
```

## 本地：构建与部署

### 方式一：使用 deploy-website.ps1（推荐）

项目根目录已提供 `deploy-website.ps1`，它会自动完成构建、打包、上传和解压。

```powershell
.\deploy-website.ps1
```

脚本流程：
1. 在本地执行 `npm run build` 生成静态文件（输出到 `website/.vitepress/dist`）
2. 将 `dist` 打包为 tar.gz
3. 通过 SCP 上传到服务器的 `/tmp`
4. 通过 SSH 解压到 `/home/visus/auto-website`

### 方式二：手动部署

```powershell
# 1. 构建
cd website
npm run build

# 2. 打包上传
tar -czf auto-website-dist.tar.gz -C .vitepress/dist .
scp auto-website-dist.tar.gz visus@112.74.45.241:/tmp/

# 3. 服务器端解压
ssh visus@112.74.45.241 "rm -rf /home/visus/auto-website/* && tar -xzf /tmp/auto-website-dist.tar.gz -C /home/visus/auto-website && rm /tmp/auto-website-dist.tar.gz"
```

## 故障排查

### 403 Forbidden

- **原因**：`auto-website` 目录为空，或 Nginx 没有权限读取文件。
- **解决**：检查 `/home/visus/auto-website` 是否包含 `index.html`；确认家目录权限为 `755`（`chmod 755 /home/visus`）。

### 500 Internal Server Error

- **原因**：Nginx 配置中的 `root` 指向的目录不存在，或上级目录权限不足导致 Nginx 无法进入。
- **解决**：确保 `mkdir -p /home/visus/auto-website` 已执行，且 `chmod 755 /home/visus` 已设置。

### 修改后不生效

- 浏览器可能缓存了旧页面，尝试 **Ctrl + F5** 强制刷新。
- 若修改了 Nginx 配置，需在服务器执行 `sudo nginx -s reload`。

## 后续扩展：启用 HTTPS

若后续要为网站添加 SSL 证书，只需在 Nginx 配置中增加 443 端口监听和证书路径，无需改动应用代码。推荐使用 [Certbot](https://certbot.eff.org/) 自动申请 Let's Encrypt 证书。
