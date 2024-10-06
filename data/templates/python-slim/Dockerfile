ARG REGISTRY="r.o99.no"

FROM $REGISTRY/kriger/exploit-base:python-slim

COPY requirements.txt .
RUN /usr/bin/uv pip install --system -r requirements.txt
COPY . .

CMD ["python3", "exploit.py"]
