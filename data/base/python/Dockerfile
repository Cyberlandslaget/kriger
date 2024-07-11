# https://github.com/astral-sh/uv/pkgs/container/uv
FROM ghcr.io/astral-sh/uv:0.2.24 as uv

FROM python:3.12-slim-bullseye
WORKDIR /exploit

# Disable stdout/stderr buffering. See https://docs.python.org/3/using/cmdline.html#cmdoption-u
ENV PYTHONUNBUFFERED 1

COPY requirements.txt .

RUN --mount=from=uv,source=/uv,target=/usr/bin/uv \
    /usr/bin/uv pip install --system -r requirements.txt

COPY . .
