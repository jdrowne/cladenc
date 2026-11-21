FROM debian:stable-slim

RUN apt-get update \
 && apt-get install --yes --no-install-recommends ca-certificates curl git ripgrep unzip \
 && rm -rf /var/lib/apt/lists/*

ENV BUN_INSTALL_CACHE_DIR=/var/cache/bun \
    HOME=/tmp \
    MODELS_DEV_API_JSON=/usr/local/share/models.dev/api.json

RUN --mount=type=bind,target=/usr/local/src/opencode \
    bun=$(sed -n 's/.*"packageManager": *"bun@\([^"]*\)".*/\1/p' /usr/local/src/opencode/package.json) \
 && curl --fail --location --silent --show-error --output /tmp/bun.zip \
      "https://github.com/oven-sh/bun/releases/download/bun-v$bun/bun-linux-aarch64.zip" \
 && unzip -j /tmp/bun.zip bun-linux-aarch64/bun -d /usr/local/bin \
 && rm /tmp/bun.zip \
 && mkdir -p /usr/local/share/models.dev \
 && curl --fail --location --silent --show-error --output "$MODELS_DEV_API_JSON" \
      https://models.dev/api.json \
 && cp -R /usr/local/src/opencode /tmp/opencode \
 && cd /tmp/opencode \
 && bun install --frozen-lockfile --ignore-scripts \
 && cd / \
 && rm -rf /tmp/opencode /tmp/.bun \
 && chmod -R a+rwX "$BUN_INSTALL_CACHE_DIR"

ENV GIT_ALLOW_PROTOCOL=file \
    NPM_CONFIG_OFFLINE=true \
    OPENCODE_CHANNEL=local \
    OPENCODE_DISABLE_AUTOUPDATE=1 \
    OPENCODE_DISABLE_LSP_DOWNLOAD=1 \
    OPENCODE_DISABLE_MODELS_FETCH=1 \
    OPENCODE_DISABLE_PROJECT_CONFIG=1 \
    OPENCODE_VERSION=0.0.0-local
