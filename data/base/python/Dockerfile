ARG REGISTRY="r.o99.no"

# https://github.com/astral-sh/uv/pkgs/container/uv
FROM ghcr.io/astral-sh/uv:0.2.24 as uv

FROM $REGISTRY/kriger/kriger as runner

# Note that we SHOULD NOT use the slim variant due to some Python packages relying on native compilation (eg. gcc)
FROM python:3.12-bookworm
WORKDIR /exploit

# Disable stdout/stderr buffering. See https://docs.python.org/3/using/cmdline.html#cmdoption-u
ENV PYTHONUNBUFFERED 1

COPY requirements.txt .

RUN --mount=from=uv,source=/uv,target=/usr/bin/uv \
    /usr/bin/uv pip install --system -r requirements.txt

COPY . .

COPY --from=runner /usr/bin/kriger /usr/bin/kriger
ENTRYPOINT ["/usr/bin/kriger", "runner"]