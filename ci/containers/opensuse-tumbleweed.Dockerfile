# THIS FILE WAS AUTO-GENERATED
#
#  $ lcitool manifest ci/manifest.yml
#
# https://gitlab.com/libvirt/libvirt-ci

FROM registry.opensuse.org/opensuse/tumbleweed:latest

RUN zypper dist-upgrade -y && \
    zypper install -y \
           ca-certificates \
           cargo \
           cargo-c \
           cargo-rpm-macros \
           ccache \
           git \
           glibc-locale \
           go \
           libvirt-devel \
           libxml2-devel \
           lua-devel \
           openssl-devel \
           perl-base \
           python3-devel \
           python3-libvirt \
           rpm-build && \
    zypper clean --all && \
    rpm -qa | sort > /packages.txt

ENV CCACHE_WRAPPERSDIR "/usr/libexec/ccache-wrappers"
ENV LANG "en_US.UTF-8"
