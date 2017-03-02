#!/bin/bash
#
# Simple test wrapper which is not optimal as it currently does not download and start own EventStore inmemory instance.
# Also this is quite light-weight testing :)
#
# Run this with `bash test_with_inmemory_es.bash` while running an inmemory EventStore on localhost. You'll need to restart the server between the runs.
set -eu
set -o pipefail

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ES_URL="http://download.geteventstore.com/binaries/EventStore-OSS-Ubuntu-14.04-v3.9.3.tar.gz"
TMP_DIR="$DIR/target/es"
ES_PATH="EventStore-OSS-Ubuntu-14.04-v3.9.3"
ES_PID=""

first_message_orig='{ "this": "is", "test": "ing" }'
first_message_fmtd='{"data":{"this":"is","test":"ing"},"metadata":{}}'
second_message_orig='{ "another": "message" }'
second_message_fmtd='{"data":{"another":"message"},"metadata":{}}'

run () {
	RUST_BACKTRACE=1 cargo run --quiet -- --host 127.0.0.1 --port 1113 "$@"
}

main () {
	#download_es || { echo "Downloading EventStore failed"; return 1; }
	#"$TMP_DIR/$ES_PATH/run_node.sh" --mem-db & &> eventstore.log
	#ES_PID=$!

	run "write" --json hello-world created test_message "$first_message_orig"
	run "write" --json hello-world 0 test_message "$second_message_orig"
	diff <(run "read" --output json_oneline --mode forward-once --count 2 --position first hello-world) <(echo -e "$first_message_fmtd\n$second_message_fmtd")
	diff <(run "read" --output json_oneline --mode backward --count 2 --position last hello-world) <(echo -e "$second_message_fmtd\n$first_message_fmtd")
}

kill_server () {
	if [ "$ES_PID" ]; then
		echo "killing eventstore: $ES_PID" >&2
		kill "$ES_PID"
		echo "awaiting EventStore to stop..." >&2
		wait "$ES_PID"
		echo "done awaiting, exit code: $?" >&2
		ES_PID=""
	fi
}

download_es () {
	if [ -x "$TMP_DIR/$ES_PATH/run_node.sh" ]; then
		return 0
	fi
	if [ -d "$TMP_DIR/$ES_PATH" ]; then
		echo "removing previously partially unpacked EventStore under $TMP_DIR" >&2
		rm -rf "$TMP_DIR"
	fi
	mkdir -p "$TMP_DIR"
	echo "downloading and unpacking EventStore under $TMP_DIR ..." >&2
	cd "$DIR/target/es"
	curl --silent http://download.geteventstore.com/binaries/EventStore-OSS-Ubuntu-14.04-v3.9.3.tar.gz | tar zxf -
}

trap kill_server EXIT
main "$@"
