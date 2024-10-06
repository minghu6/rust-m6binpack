# Enable concat multiple command in one shell globally
# easy `cd` use
.ONESHELL:

test-armv7:
	@ cd example-armv7
	@ cargo run

test-std:
	@ cd example-std
	@ cargo run

integration-test: test-armv7 test-std
