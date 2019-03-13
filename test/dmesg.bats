load helpers

@test "outputting to dmesg works" {
    if [ "$(id -u)" != "0" ]; then
        skip "not root, can't test dmesg"
    fi

    # don't use our helper: we want to test the dmesg logging bits, and our
    # helper turns off dmesg logging.
    run bash -c "exec -a /bin/true \"${ROOT_DIR}/target/debug/usermode-helper\""
    echo "$output"
    [ "$status" -eq 1 ]

    dmesg | tail | grep "couldn't read config file ./usermode-helper.conf"
}
