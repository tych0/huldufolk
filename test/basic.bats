load helpers

function setup() {
    make_tempdir
}

function teardown() {
    cleanup
}

@test "basic /bin/true helper" {
    cat <<EOF > "${TEMP_DIR}/usermode-helper.conf"
[[helpers]]
path = "/bin/true"
EOF
    usermode-helper /bin/true
}

@test "basic /bin/ls denial" {
    cat <<EOF > "${TEMP_DIR}/usermode-helper.conf"
[[helpers]]
path = "/bin/true"
EOF
    usermode-helper-deny /bin/ls
}

@test "argc=1 denial" {
    cat <<EOF > "${TEMP_DIR}/usermode-helper.conf"
[[helpers]]
path = "/bin/true"
argc = 1
EOF
    usermode-helper-deny /bin/true another-arg
}

@test "argc=1 allowed" {
    cat <<EOF > "${TEMP_DIR}/usermode-helper.conf"
[[helpers]]
path = "/bin/true"
argc = 1
EOF
    usermode-helper /bin/true
}

@test "argc=5 denial" {
    cat <<EOF > "${TEMP_DIR}/usermode-helper.conf"
[[helpers]]
path = "/bin/true"
argc = 5
EOF
    usermode-helper-deny /bin/true another-arg
}

@test "argc=5 allowed" {
    cat <<EOF > "${TEMP_DIR}/usermode-helper.conf"
[[helpers]]
path = "/bin/true"
argc = 5
EOF
    usermode-helper /bin/true one two three four
}
