set -x

export CCACHE_BASEDIR="$(pwd)"
export CCACHE_DIR="$CCACHE_BASEDIR/ccache"
export CCACHE_MAXSIZE="500M"

if test -n "$CCACHE_WRAPPERSDIR"
then
    export PATH="$CCACHE_WRAPPERSDIR:$PATH"
fi

GIT_ROOT="$(git rev-parse --show-toplevel)"
run_cmd() {
    printf "\e[32m[RUN COMMAND]: '%s'\e[0m\n" "$*"
    "$@"
}

run_cmd_quiet() {
    printf "\e[32m[RUN COMMAND]: '%s'\e[0m\n" "$*"
    "$@" 1>/dev/null 2>&1
}

run_tests() {
    run_cmd make check
}

run_build() {
    run_cmd make
}

run_rpmbuild() {
    run_cmd make rpm
}
