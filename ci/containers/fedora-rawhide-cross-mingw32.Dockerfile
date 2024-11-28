# THIS FILE WAS AUTO-GENERATED
#
#  $ lcitool manifest ci/manifest.yml
#
# https://gitlab.com/libvirt/libvirt-ci

FROM registry.fedoraproject.org/fedora:rawhide

RUN dnf update -y --nogpgcheck fedora-gpg-keys && \
    dnf install -y nosync && \
    printf '#!/bin/sh\n\
if test -d /usr/lib64\n\
then\n\
    export LD_PRELOAD=/usr/lib64/nosync/nosync.so\n\
else\n\
    export LD_PRELOAD=/usr/lib/nosync/nosync.so\n\
fi\n\
exec "$@"\n' > /usr/bin/nosync && \
    chmod +x /usr/bin/nosync && \
    nosync dnf distro-sync -y && \
    nosync dnf install -y \
               ca-certificates \
               cargo \
               cargo-c \
               cargo-rpm-macros \
               ccache \
               git \
               glibc-langpack-en \
               golang \
               lua-devel \
               openssl-devel \
               perl-base \
               python3-devel \
               python3-libvirt \
               rpm-build && \
    nosync dnf autoremove -y && \
    nosync dnf clean all -y

ENV CCACHE_WRAPPERSDIR "/usr/libexec/ccache-wrappers"
ENV LANG "en_US.UTF-8"

RUN nosync dnf install -y \
               mingw32-libvirt \
               mingw32-libxml2 && \
    nosync dnf clean all -y && \
    rpm -qa | sort > /packages.txt

ENV ABI "i686-w64-mingw32"
