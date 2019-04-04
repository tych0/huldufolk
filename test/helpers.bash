ROOT_DIR=$(git rev-parse --show-toplevel)

function make_tempdir()
{
    declare -g TEMP_DIR=$(mktemp -d huldufolk-test.XXXXXXXX)
}

function real-usermode-helper {
    if [ "$#" -gt 1 ]; then
        args="${@:2}"
    else
        args=""
    fi
    pushd "${TEMP_DIR}"
    run bash -c "HULDUFOLK_DEBUG=1 exec -a $1 \"${ROOT_DIR}/target/debug/usermode-helper\" $args"
    popd
    echo "$output"
}

function usermode-helper {
    real-usermode-helper "$@"
    [ "$status" -eq 0 ]
}

function usermode-helper-deny {
    real-usermode-helper "$@"
    if ! echo "$output" | grep 'invalid usermode helper'; then
        echo "failed, but didn't deny" && false
    fi
    [ "$status" -eq 1 ]
}

function usermode-helper-fail {
    real-usermode-helper "$@"
    if echo "$output" | grep 'invalid usermode helper'; then
        echo "failed, but was denied" && false
    fi
    [ "$status" -eq 1 ]
}

function cleanup {
    rm -rf "$TEMP_DIR"
}
