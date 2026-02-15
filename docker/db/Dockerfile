FROM postgres:17

# アーキテクチャの自動検出
ARG TARGETARCH

# pg_textsearch のバージョン、ダウンロード URL
ARG PG_TEXTSEARCH_VERSION=0.5.0
ARG PG_TEXTSEARCH_BASE_URL=https://github.com/timescale/pg_textsearch/releases/download

# 必要なパッケージのインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    locales \
    curl \
    ca-certificates \
    postgresql-17-pgvector \
    && rm -rf /var/lib/apt/lists/*

# pg_textsearch のダウンロードとインストール
RUN if [ "$TARGETARCH" = "arm64" ]; then \
        arch="arm64"; \
    else \
        arch="amd64"; \
    fi && \
    curl -L -o /tmp/pg_textsearch.deb \
        "${PG_TEXTSEARCH_BASE_URL}/v${PG_TEXTSEARCH_VERSION}/pg-textsearch-postgresql-17_${PG_TEXTSEARCH_VERSION}-1_${arch}.deb" && \
    dpkg -i /tmp/pg_textsearch.deb && \
    rm /tmp/pg_textsearch.deb