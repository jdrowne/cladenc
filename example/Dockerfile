FROM debian:13.7-slim

RUN apt-get update \
 && apt-get install --yes --no-install-recommends python3 diffutils grep sed gawk procps \
 && rm -rf /var/lib/apt/lists/*

ENV HOME=/tmp \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1
