# THIS FILE WAS AUTO-GENERATED
#
#  $ lcitool manifest ci/manifest.yml
#
# https://gitlab.com/libvirt/libvirt-ci

function install_buildenv() {
    zypper dist-upgrade -y
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
           rpm-build
    rpm -qa | sort > /packages.txt
}

export CCACHE_WRAPPERSDIR="/usr/libexec/ccache-wrappers"
export LANG="en_US.UTF-8"
