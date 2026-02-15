FROM ubuntu:24.04 AS installer
LABEL maintainer="yukijya"


ENV TZ=Asia/Tokyo
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

USER root

RUN rm -f /etc/apt/apt.conf.d/docker-clean \
    && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && \
    apt-get upgrade -y&&\
    apt-get install -y \
    ca-certificates \
    curl \
    build-essential \
    unzip

# install awscli
RUN curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip" \
    && unzip awscliv2.zip \
    && ./aws/install \
    && rm -rf awscliv2.zip aws


FROM rust:latest
LABEL maintainer="yukijya"

ARG UID
ARG USER
ARG GROUP
ARG GID

ENV TZ=Asia/Tokyo
ENV PATH="/usr/local/aws-cli/v2/current/bin:${PATH}"
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

COPY --from=installer /usr/local/aws-cli/ /usr/local/aws-cli/

USER root

# Minimal packages; rust toolchain is preinstalled in base image
RUN rm -f /etc/apt/apt.conf.d/docker-clean \
    && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache \
    && apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y --no-install-recommends ca-certificates curl pkg-config libssl-dev sudo libimage-exiftool-perl \
    && rm -rf /var/lib/apt/lists/*


RUN groupadd -f -g ${GID} ${GROUP}
RUN useradd -ms /bin/zsh -u ${UID} -g ${GID} -G sudo ${USER} &&\
    echo ${USER}:${USER} | chpasswd

USER ${USER}
WORKDIR /home/${USER}

CMD ["bash"]
