#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
# SPDX-License-Identifier: GPL-3.0-or-later

set -eu -o pipefail

: "${TEST_DIR:=target/tests}"
#BORGREPORT='target/debug/borgreport --env-dir tests'
BORGREPORT="cargo run -- --env-dir ${TEST_DIR}"
BASE_DIR="${TEST_DIR}/borg"

# Unset all BORG_* variables
unset_env() {
	unset "${!BORG_@}"
	unset "${!BORGREPORT_@}"
}

# Read ENV from stdin and write to <FILE>.
# Source <FILE> and export all variables
# Usage: make_env <FILE>
make_env() {
	local REPLY IFS=''
	read -r -d '' || printf '%s' "${REPLY}" >"${1}"

	set -o allexport
	# shellcheck disable=SC1090
	source "${1}"
	set +o allexport

	export BORG_EXIT_CODES=modern
}

# Test 1: Repo not initialized
# Prepare:	test1 prepare
# Run:			test1 run <REPORT>
# Returns 0 on success
test1() {
	local name="${FUNCNAME[0]}-noinit"
	case ${1} in
	prepare)
		make_env "${TEST_DIR}/${name}.env" <<-EOF
			BORG_BASE_DIR=${BASE_DIR}
			BORG_REPO=${TEST_DIR}/${name}
		EOF
		mkdir "${TEST_DIR}/${name}"
		;;
	run)
		grep --silent -F "${name} is not a valid repository." <<<"${2}"
		;;
	*) echo "Wrong call to ${FUNCNAME[0]}. Unkown action." >&2 && exit 1 ;;
	esac
}

# Test 2: Repo is empty
# Prepare:	test2 prepare
# Run:			test2 run <REPORT>
# Returns 0 on success
test2() {
	local name="${FUNCNAME[0]}-nobackups"
	case ${1} in
	prepare)
		make_env "${TEST_DIR}/${name}.env" <<-EOF
			BORG_BASE_DIR=${BASE_DIR}
			BORG_REPO=${TEST_DIR}/${name}
			BORG_PASSPHRASE=${name}
			BORGREPORT_CHECK=true
		EOF
		borg init --encryption=repokey &>/dev/null
		;;
	run)
		grep --silent -F -e "${name}: Repository is empty" <<<"${2}" &&
			grep --silent -E -e "^\| ${name}(.*)yes \|$" <<<"${2}"
		;;
	*) echo "Wrong call to ${FUNCNAME[0]}. Unknown action." >&2 && exit 1 ;;
	esac
}

# Test 3: Check OK
# Prepare:	test3 prepare
# Run:			test3 run <REPORT>
# Returns 0 on success
test3() {
	local name="${FUNCNAME[0]}-checkok"
	case ${1} in
	prepare)
		make_env "${TEST_DIR}/${name}.env" <<-EOF
			BORG_BASE_DIR=${BASE_DIR}
			BORG_REPO=${TEST_DIR}/${name}
			BORG_PASSPHRASE=${name}
			BORGREPORT_CHECK=true
		EOF
		borg init --encryption=repokey &>/dev/null
		borg create '::{utcnow}Z' "${BASH_SOURCE[0]}"
		;;
	run)
		grep --silent -E -e "^\| ${name}(.*)yes \|$" <<<"${2}"
		;;
	*) echo "Wrong call to ${FUNCNAME[0]}. Unknown action." >&2 && exit 1 ;;
	esac
}

# Test 4: Check NOT OK
# Prepare:	test4 prepare
# Run:			test4 run <REPORT>
# Returns 0 on success
test4() {
	local name="${FUNCNAME[0]}-checknotok"
	case ${1} in
	prepare)
		make_env "${TEST_DIR}/${name}.env" <<-EOF
			BORG_BASE_DIR=${BASE_DIR}
			BORG_REPO=${TEST_DIR}/${name}
			BORG_PASSPHRASE=${name}
			BORGREPORT_CHECK=true
		EOF
		borg init --encryption=repokey &>/dev/null
		borg create '::{utcnow}Z' "${BASH_SOURCE[0]}"
		echo 'FOOBAR' >>"${TEST_DIR}/${name}/data/0/0"
		borg delete --cache-only ::
		;;
	run)
		grep --silent -F -i "Finished full repository check, errors found." <<<"${2}" &&
			grep --silent -E -e "^\| ${name}(.*)no \|$" <<<"${2}"
		;;
	*) echo "Wrong call to ${FUNCNAME[0]}. Unknown action." >&2 && exit 1 ;;
	esac
}

# Test 5: twoarchivesok
# Prepare:	test5 prepare
# Run:			test5 run <REPORT>
# Returns 0 on success
test5() {
	local name="${FUNCNAME[0]}-twoarchivesok"
	case ${1} in
	prepare)
		make_env "${TEST_DIR}/${name}.env" <<-EOF
			BORG_BASE_DIR=${BASE_DIR}
			BORG_REPO=${TEST_DIR}/${name}
			BORG_PASSPHRASE=${name}
			BORGREPORT_CHECK=true
			BORGREPORT_GLOB_ARCHIVES="etc-* srv-*"
		EOF
		borg init --encryption=repokey &>/dev/null
		borg create '::etc-{utcnow}Z' "${BASH_SOURCE[0]}"
		borg create '::srv-{utcnow}Z' "${BASH_SOURCE[0]}"
		;;
	run)
		grep --silent -E -e "^\| ${name}(.*)\| etc-(.*)yes \|$" <<<"${2}" &&
			grep --silent -E -e "^\| ${name}(.*)\| srv-(.*)yes \|$" <<<"${2}"
		;;
	*) echo "Wrong call to ${FUNCNAME[0]}. Unknown action." >&2 && exit 1 ;;
	esac
}

# Prepare test <NUMBER> for execution
# Usage: test <NUMBER>
prepare_test() {
	unset_env
	"test${i}" prepare
}

# Run test <NUMBER> and print result.
# Returns 0 on success and 1 on failure.
# - If no [REPORT] is provided, a test specific one is generated.
# Usage: test <NUMBER> [REPORT]
run_test() {
	unset_env #clean up exported values from prepare
	report="${2:-$(${BORGREPORT})}"
	[ -z "${2:-}" ] && printf '%s' "${report}" >>"${TEST_DIR}/test${1}_report.txt"
	set +e
	{
		local -i result=0
		if "test${1}" run "${report}"; then
			printf 'Test%-2s: OK\n' "${1}"
			result=0
		else
			printf 'Test%-2s: Failed\n' "${1}"
			result=1
		fi
	}
	set -e
	return ${result}
}

# Print all test numbers
all_test_num() {
	declare -F | grep -E -i --only-matching '^declare -F test([[:digit:]]*)$' | grep --only-matching '[[:digit:]]*$'
}

# Run all tests against a single report
# Returns 0 on success and 1 on failure.
run_combined() {
	tests=$(all_test_num)
	for i in ${tests}; do
		prepare_test "${i}"
	done

	unset_env #clean up exported values from prepare
	report=$(${BORGREPORT})
	printf '%s' "${report}" >>"${TEST_DIR}/combined_report.txt"

	local -i result=0
	for i in ${tests}; do
		if ! run_test "${i}" "${report}"; then
			result=1
		fi
	done
	return ${result}
}

bootstrap() {
	mkdir -p "${TEST_DIR}"
}

clean() {
	rm -rf "${TEST_DIR}"
}

clean
bootstrap
run_combined
