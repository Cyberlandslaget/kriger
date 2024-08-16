FROM node:20-slim AS build
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable pnpm
WORKDIR /app
COPY package.json .
COPY pnpm-lock.yaml .
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
COPY . .
RUN pnpm run build

FROM cgr.dev/chainguard/nginx
COPY --from=build /app/dist /usr/share/nginx/html
COPY ./config/nginx.conf /etc/nginx/conf.d/nginx.default.conf
