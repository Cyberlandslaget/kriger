ARG REGISTRY="r.o99.no"

# https://github.com/astral-sh/uv/pkgs/container/uv
FROM ghcr.io/astral-sh/uv:0.2.24 AS uv

FROM $REGISTRY/kriger/exploit-base:python

COPY requirements.txt .

RUN --mount=from=uv,source=/uv,target=/usr/bin/uv \
    /usr/bin/uv pip install --system -r requirements.txt

COPY . .

CMD ["python", "exploit.py"]
