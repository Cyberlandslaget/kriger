ARG REGISTRY="r.o99.no"

# https://github.com/astral-sh/uv/pkgs/container/uv
FROM ghcr.io/astral-sh/uv:0.3.3 AS uv

FROM $REGISTRY/kriger/kriger AS runner

FROM python:3.12-slim-bookworm
COPY --from=uv /uv /usr/bin/uv
COPY --from=runner /usr/bin/kriger /usr/bin/kriger

WORKDIR /exploit

# Disable stdout/stderr buffering. See https://docs.python.org/3/using/cmdline.html#cmdoption-u
ENV PYTHONUNBUFFERED 1

COPY requirements.txt .
RUN /usr/bin/uv pip install --system -r requirements.txt
COPY . .

ENTRYPOINT ["/usr/bin/kriger", "runner"]