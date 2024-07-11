# https://github.com/astral-sh/uv/pkgs/container/uv
FROM ghcr.io/astral-sh/uv:0.2.24 as uv

FROM kriger-base:python

COPY requirements.txt .

RUN --mount=from=uv,source=/uv,target=/usr/bin/uv \
    /usr/bin/uv pip install --system -r requirements.txt

COPY . .
