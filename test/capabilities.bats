load helpers

function setup() {
    make_tempdir
}

function teardown() {
    cleanup
}

@test "caps are dropped correctly" {
    cat >> "${TEMP_DIR}/usermode-helper.conf" <<EOF
[[helpers]]
path = "/bin/hostname"
capabilities = "="
EOF
    usermode-helper-fail /bin/hostname foo
}

@test "caps are kept correctly" {
    cat >> "${TEMP_DIR}/usermode-helper.conf" <<EOF
[[helpers]]
path = "/bin/hostname"
capabilities = "cap_sys_admin+eip"
EOF
    usermode-helper /bin/hostname foo
}
