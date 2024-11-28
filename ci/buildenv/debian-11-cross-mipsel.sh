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
    dpkg --add-architecture mipsel
    apt-get update
    apt-get dist-upgrade -y
    apt-get install --no-install-recommends -y dpkg-dev
    apt-get install --no-install-recommends -y \
            libvirt-dev:mipsel \
            libxml2-dev:mipsel
    mkdir -p /usr/local/share/meson/cross
    printf "[binaries]\n\
c = '/usr/bin/mipsel-linux-gnu-gcc'\n\
ar = '/usr/bin/mipsel-linux-gnu-gcc-ar'\n\
strip = '/usr/bin/mipsel-linux-gnu-strip'\n\
pkgconfig = '/usr/bin/mipsel-linux-gnu-pkg-config'\n\
\n\
[host_machine]\n\
system = 'linux'\n\
cpu_family = 'mips'\n\
cpu = 'mipsel'\n\
endian = 'little'\n" > /usr/local/share/meson/cross/mipsel-linux-gnu
    dpkg-query --showformat '${Package}_${Version}_${Architecture}\n' --show > /packages.txt
}

export CCACHE_WRAPPERSDIR="/usr/libexec/ccache-wrappers"
export LANG="en_US.UTF-8"

export ABI="mipsel-linux-gnu"
