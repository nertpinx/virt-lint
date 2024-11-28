# THIS FILE WAS AUTO-GENERATED
#
#  $ lcitool manifest ci/manifest.yml
#
# https://gitlab.com/libvirt/libvirt-ci

FROM docker.io/library/alpine:edge

RUN apk update && \
    apk upgrade && \
    apk add \
        ca-certificates \
        cargo \
        cargo-c \
        ccache \
        git \
        go \
        libvirt-dev \
        libxml2-dev \
        lua5.4 \
        openssl-devel \
        perl \
        py3-libvirt \
        python3-dev && \
    apk list --installed | sort > /packages.txt

ENV CCACHE_WRAPPERSDIR "/usr/libexec/ccache-wrappers"
ENV LANG "en_US.UTF-8"
