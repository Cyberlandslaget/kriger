ARG REPOSITORY="ghcr.io/cyberlandslaget/kriger-exploit-base"
FROM $REPOSITORY:python-slim

COPY requirements.txt .
RUN /usr/bin/uv pip install --system -r requirements.txt
COPY . .

CMD ["python3", "exploit.py"]
