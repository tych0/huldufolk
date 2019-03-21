load helpers

function setup() {
    make_tempdir

    cat <<EOF > "${TEMP_DIR}/usermode-helper.conf"
[[helpers]]
path = "/bin/true"
EOF
}

function teardown() {
    cleanup
}

@test "basic /bin/true helper" {
    usermode-helper /bin/true
}

@test "basic /bin/ls denial" {
    usermode-helper-deny /bin/ls
}
