# Firmware for STM32U5AZJ to test error handling PR in embassy

This is run on the [STM32 Nucleo-144](https://www.st.com/en/evaluation-tools/nucleo-u5a5zj-q.html#documentation) dev board. Simply plug it in using the micro-usb port and run `cargo run`. It builds, flashes and runs the firmware. If all the tests in `main` pass you should get a `ALL TESTS PASS` log in your terminal. Otherwise the firmware should panic and you'd get another, much nastier, log.
