# syntax=docker/dockerfile:1.7

# マスタメンテ Web の本番イメージ。ランタイムには Node.js を含めず nginx で静的配信する。
# ビルドコンテキストは src/frontend/ をルートとし、shared と master のワークスペースを同時にコピーする。

FROM node:22-slim AS builder
WORKDIR /app
ENV CI=1
COPY package.json package-lock.json* ./
COPY tsconfig.base.json ./
COPY shared/package.json ./shared/
COPY master/package.json ./master/
RUN npm ci --workspace=@wnav/shared --workspace=@wnav/master --include-workspace-root || npm install --workspaces --include-workspace-root
COPY shared ./shared
COPY master ./master
RUN npm run build --workspace=@wnav/master

FROM nginx:1.27-alpine AS runtime
COPY --from=builder /app/master/dist /usr/share/nginx/html
COPY master/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -qO- http://127.0.0.1/ >/dev/null 2>&1 || exit 1
CMD ["nginx", "-g", "daemon off;"]
