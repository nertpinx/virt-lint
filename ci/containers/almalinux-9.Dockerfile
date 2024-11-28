# THIS FILE WAS AUTO-GENERATED
#
#  $ lcitool manifest ci/manifest.yml
#
# https://gitlab.com/libvirt/libvirt-ci

FROM docker.io/library/almalinux:9

RUN dnf update -y && \
    dnf install 'dnf-command(config-manager)' -y && \
    dnf config-manager --set-enabled -y crb && \
    dnf install -y epel-release && \
    dnf install -y \
        ca-certificates \
        cargo \
        cargo-c \
        cargo-rpm-macros \
        ccache \
        git \
        glibc-langpack-en \
        golang \
        libvirt-devel \
        libxml2-devel \
        lua-devel \
        openssl-devel \
        perl-base \
        python3-devel \
        python3-libvirt \
        rpm-build && \
    dnf autoremove -y && \
    dnf clean all -y && \
    rpm -qa | sort > /packages.txt

ENV CCACHE_WRAPPERSDIR "/usr/libexec/ccache-wrappers"
ENV LANG "en_US.UTF-8"
