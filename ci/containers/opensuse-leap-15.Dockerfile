# THIS FILE WAS AUTO-GENERATED
#
#  $ lcitool manifest ci/manifest.yml
#
# https://gitlab.com/libvirt/libvirt-ci

FROM registry.opensuse.org/opensuse/leap:15.6

RUN zypper update -y && \
    zypper addrepo -fc https://download.opensuse.org/update/leap/15.6/backports/openSUSE:Backports:SLE-15-SP6:Update.repo && \
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