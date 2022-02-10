# syntax = docker/dockerfile:experimental
FROM multiarch/ubuntu-core:armhf-focal

ARG NODE_VERSION=14

RUN apt-get update && \
  apt-get install -y ca-certificates gnupg2 curl apt-transport-https && \
  curl -sL https://deb.nodesource.com/setup_${NODE_VERSION}.x | bash - && \
  apt-get install -y nodejs && \
  npm install -g yarn pnpm

RUN --security=insecure mkdir -p /root/.cargo && chmod 777 /root/.cargo && mount -t tmpfs none /root/.cargo
