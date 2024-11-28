VIRTLINT_BASE = $(shell realpath ../target/debug 2>/dev/null)
#some dummy fallback
ifeq ($(VIRTLINT_BASE),)
VIRTLINT_BASE := $(PWD)/target/debug/
endif

VIRT_LINT_VALIDATORS_PATH = $(shell realpath "../validators/" 2>/dev/null)

run:
	PYTHONPATH="${VIRTLINT_BASE}" VIRT_LINT_VALIDATORS_PATH="${VIRT_LINT_VALIDATORS_PATH}" ./virt_lint_test.py
