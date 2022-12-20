FROM bitnami/minideb:bullseye

# Install dependencies
RUN install_packages \
    ca-certificates \
    curl \
    gnupg \
    lsb-release \
    procps \
    sudo \
    unzip

RUN set -eux; \
    groupadd -r rustbase --gid=999; \
    useradd -r -g rustbase --uid=999 --home-dir=/var/lib/rustbase --shell=/bin/bash rustbase; \
    mkdir -p /var/lib/rustbase; \
    chown -R rustbase:rustbase /var/lib/rustbase

ENV RUSTBASE_INSTALL_PATH /var/lib/rustbase
ENV PATH $PATH:/var/lib/rustbase

RUN curl -sS https://raw.githubusercontent.com/rustbase/rustbase-install/main/install.sh | bash -s -- --no-cli --no-service

RUN mkdir -p /var/lib/rustbase/data && \
    chown -R rustbase:rustbase /var/lib/rustbase

VOLUME /var/lib/rustbase/data

# Expose the default port
EXPOSE 23561

CMD ["rustbase_server"]