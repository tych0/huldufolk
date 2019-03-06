ROOT_DIR=$(git rev-parse --show-toplevel)

function make_tempdir()
{
    declare -g TEMP_DIR=$(mktemp -d huldufolk-test.XXXXXXXX)
}

function usermode-helper {
    if [ "$#" -gt 1 ]; then
        args="${@:2}"
    else
        args=""
    fi
    pushd "${TEMP_DIR}"
    run bash -c "exec -a $1 \"${ROOT_DIR}/target/debug/usermode-helper\" $args"
    popd
    echo "$output"
    [ "$status" -eq 0 ]
}

function cleanup {
    rm -rf "$TEMP_DIR"
}
