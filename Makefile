# Enable concat multiple command in one shell globally
# easy `cd` use
.ONESHELL:

EXAMPLE_ARMV7="example-armv7"
EXAMPLE_STD="example-std"
MAIN="m6binpack"
PROC_MACRO="proc_macros"

test: doc-test integration-test

test-armv7:
	@ cd $(EXAMPLE_ARMV7)
	@ cargo run

clean-armv7:
	@ cd $(EXAMPLE_ARMV7)
	@ cargo clean

test-std:
	@ cd $(EXAMPLE_STD)
	@ cargo run

clean-std:
	@ cd $(EXAMPLE_STD)
	@ cargo clean

doc-test:
	@ cd $(MAIN)/$(PROC_MACRO)
	@ cargo test --doc

integration-test: test-armv7 test-std

clean: clean-armv7 clean-std
