# THIS FILE WAS AUTO-GENERATED
#
#  $ lcitool manifest ci/manifest.yml
#
# https://gitlab.com/libvirt/libvirt-ci

function install_buildenv() {
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get dist-upgrade -y
    apt-get install --no-install-recommends -y \
            ca-certificates \
            cargo \
            cargo-c \
            ccache \
            git \
            golang \
            libvirt-dev \
            libxml2-dev \
            locales \
            lua5.4 \
            openssl-devel \
            perl-base \
            python3-dev \
            python3-libvirt
    sed -Ei 's,^# (en_US\.UTF-8 .*)$,\1,' /etc/locale.gen
    dpkg-reconfigure locales
    dpkg-query --showformat '${Package}_${Version}_${Architecture}\n' --show > /packages.txt
}

export CCACHE_WRAPPERSDIR="/usr/libexec/ccache-wrappers"
export LANG="en_US.UTF-8"