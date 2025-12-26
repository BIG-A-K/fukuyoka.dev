# FROM nvidia/cuda:12.9.1-cudnn-devel-ubuntu24.04
ARG PYTHON="3.12"
FROM ubuntu:latest AS builder
ARG PYTHON
ENV TZ=Asia/Tokyo
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

USER root

ENV DEBIAN_FRONTEND=noninteractive \
    LANG=ja_JP.UTF-8 \
    TZ=Asia/Tokyo \
    UV_PYTHON=${PYTHON} \
    UV_PYTHON_INSTALL_DIR="/opt/python" \
    # インストール後にソースファイルをバイトコードにコンパイルする
    UV_COMPILE_BYTECODE=1 \
    # wheelからパッケージをコピーしてインストールする
    UV_LINK_MODE=copy \
    # 標準出力・標準エラーのストリームのバッファリングを行わない
    PYTHONUNBUFFERED=1 \
    # .pycファイルを生成しない
    PYTHONDONTWRITEBYTECODE=1 \
    PATH="/app/.venv/bin:~/.local/bin:/root/.cargo/bin:$PATH"

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    build-essential \
    unzip \
    curl \
    ca-certificates \
    && apt-get -y clean \
    && rm -rf /var/lib/apt/lists/*

# Install Python and packages using uv
COPY --from=ghcr.io/astral-sh/uv:latest /uv /bin/uv
WORKDIR /app
COPY scripts/pyproject.toml scripts/uv.lock ./
RUN --mount=type=cache,target=/root/.cache/uv \
    uv sync --frozen


FROM ubuntu:24.04 AS runner
ARG PYTHON

ARG UID
ARG USER
ARG GROUP
ARG GID

ENV TZ=Asia/Tokyo
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

USER root

ENV DEBIAN_FRONTEND=noninteractive \
    LANG=ja_JP.UTF-8 \
    TZ=Asia/Tokyo \
    UV_PYTHON=${PYTHON} \
    UV_PYTHON_INSTALL_DIR="/opt/python" \
    # インストール後にソースファイルをバイトコードにコンパイルする
    UV_COMPILE_BYTECODE=1 \
    # wheelからパッケージをコピーしてインストールする
    UV_LINK_MODE=copy \
    # 標準出力・標準エラーのストリームのバッファリングを行わない
    PYTHONUNBUFFERED=1 \
    # .pycファイルを生成しない
    PYTHONDONTWRITEBYTECODE=1 \
    PATH="/app/.venv/bin:~/.local/bin:/root/.cargo/bin:$PATH"

RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
USER root

COPY --from=builder /opt/python /opt/python
COPY --from=builder /app /app


SHELL ["/bin/bash", "-o", "pipefail", "-c"]

RUN rm -f /etc/apt/apt.conf.d/docker-clean \
    && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && \
    apt-get upgrade -y&&\
    apt-get install -y \
        vim \
        git \
        tmux \
        build-essential \
        sudo \
        wget \
        curl 

RUN deluser ubuntu

RUN addgroup --gid ${GID} ${GROUP} 

RUN useradd -ms /bin/bash -u ${UID} -g ${GID} -G sudo ${USER} &&\
    echo ${USER}:${USER} | chpasswd 

USER ${USER}
WORKDIR /home/${USER}

CMD ["/bin/bash"]
