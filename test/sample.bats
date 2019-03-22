load helpers

function setup() {
    make_tempdir
}

function teardown() {
    cleanup
}

@test "sample config parses correctly" {
    cp "${ROOT_DIR}/sample-usermode-helper.toml" "${TEMP_DIR}/usermode-helper.conf"

    # Now, let's append /bin/true so that there's something we can actually run
    # without consequence.
    cat >> "${TEMP_DIR}/usermode-helper.conf" <<EOF
[[helpers]]
path = "/bin/true"
EOF
    usermode-helper /bin/true
}
