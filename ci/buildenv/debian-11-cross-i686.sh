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
            locales \
            lua5.4 \
            openssl-devel \
            perl-base \
            python3-dev \
            python3-libvirt
    sed -Ei 's,^# (en_US\.UTF-8 .*)$,\1,' /etc/locale.gen
    dpkg-reconfigure locales
    export DEBIAN_FRONTEND=noninteractive
    dpkg --add-architecture i386
    apt-get update
    apt-get dist-upgrade -y
    apt-get install --no-install-recommends -y dpkg-dev
    apt-get install --no-install-recommends -y \
            libvirt-dev:i386 \
            libxml2-dev:i386
    mkdir -p /usr/local/share/meson/cross
    printf "[binaries]\n\
c = '/usr/bin/i686-linux-gnu-gcc'\n\
ar = '/usr/bin/i686-linux-gnu-gcc-ar'\n\
strip = '/usr/bin/i686-linux-gnu-strip'\n\
pkgconfig = '/usr/bin/i686-linux-gnu-pkg-config'\n\
\n\
[host_machine]\n\
system = 'linux'\n\
cpu_family = 'x86'\n\
cpu = 'i686'\n\
endian = 'little'\n" > /usr/local/share/meson/cross/i686-linux-gnu
    dpkg-query --showformat '${Package}_${Version}_${Architecture}\n' --show > /packages.txt
}

export CCACHE_WRAPPERSDIR="/usr/libexec/ccache-wrappers"
export LANG="en_US.UTF-8"

export ABI="i686-linux-gnu"
