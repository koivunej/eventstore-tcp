#!/bin/bash
#
# Simple test wrapper which is not optimal as it currently does not download and start own EventStore inmemory instance.
# Also this is quite light-weight testing :)
#
# Run this with `bash test_with_inmemory_es.bash` while running an inmemory EventStore on localhost. You'll need to restart the server between the runs.
set -eu
set -o pipefail

readonly DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
readonly ES_URL="http://download.geteventstore.com/binaries/EventStore-OSS-Ubuntu-14.04-v3.9.3.tar.gz"
readonly TMP_DIR="$DIR/target/es"
readonly ES_PATH="EventStore-OSS-Ubuntu-14.04-v3.9.3"
ES_PID=""
readonly self="$(basename "$0")"

# the events to write
readonly messages_in=(
	'{ "this": "is", "test": "ing" }'
	'{ "another": "message" }'
	'{ "blah": "blah" }'
	'{ "blahblah": "blah" }'
	'{ "fifth": "any" }'
)

# how written messages are expected to be formatted by --output json_oneline
readonly messages_out=(
	'{"data":{"this":"is","test":"ing"},"metadata":{}}'
	'{"data":{"another":"message"},"metadata":{}}'
	'{"data":{"blah":"blah"},"metadata":{}}'
	'{"data":{"blahblah":"blah"},"metadata":{}}'
	'{"data":{"fifth":"any"},"metadata":{}}'
)

run () {
	RUST_BACKTRACE=1 cargo run --quiet -- --host 127.0.0.1 --port 1113 "$@"
}

main () {
	# TODO: this would be easier to do in rust if the testclient was a bit less adhoc
	stream_name="${1:-}"
	if [ -z "$stream_name" ]; then
		stream_name="testclient-$(uuidgen -r)" || {
			echo "You need to give stream id as first argument or install 'uuidgen'." >&2
			echo "'uuidgen' is part of util-linux (debian package: uuid-runtime)" >&2
			return 2
		}
	fi

	# ES is not yet downloaded and executed as:
	#  1. no https link
	#  2. I do not know how to use seccomp to jail it for only localhost networking

	#download_es || { echo "Downloading EventStore failed"; return 1; }
	#"$TMP_DIR/$ES_PATH/run_node.sh" --mem-db & &> eventstore.log
	#ES_PID=$!

	echo "$self: using stream name $stream_name" >&2

	run "write" --json "$stream_name" created test_message "${messages_in[0]}"

	diff \
		<(run "read" --output json_oneline --mode forward-once --count 1 --position first "$stream_name") \
		<(echo -e "${messages_out[0]}")
	diff \
		<(run "read" --output json_oneline --mode backward --count 1 --position last "$stream_name") \
		<(echo -e "${messages_out[0]}")

	run "write" --json "$stream_name" 0 test_message "${messages_in[1]}"

	diff \
		<(run "read" --output json_oneline --mode forward-once --count 2 --position first "$stream_name") \
		<(echo -e "${messages_out[0]}\n${messages_out[1]}")
	diff \
		<(run "read" --output json_oneline --mode backward --count 2 --position last "$stream_name") \
		<(echo -e "${messages_out[1]}\n${messages_out[0]}")

	run "write" --json "$stream_name" 1 test_message "${messages_in[2]}"
	run "write" --json "$stream_name" 2 test_message "${messages_in[3]}"

	diff \
		<(run "read" --output json_oneline --mode forward-once --count 2 --position first "$stream_name") \
		<(echo -e "${messages_out[0]}\n${messages_out[1]}")
	diff \
		<(run "read" --output json_oneline --mode forward-once --count 3 --position first "$stream_name") \
		<(echo -e "${messages_out[0]}\n${messages_out[1]}\n${messages_out[2]}")
	diff \
		<(run "read" --output json_oneline --mode forward-once --count 3 --position 1 "$stream_name") \
		<(echo -e "${messages_out[1]}\n${messages_out[2]}\n${messages_out[3]}")
	diff \
		<(run "read" --output json_oneline --mode backward --count 2 --position last "$stream_name") \
		<(echo -e "${messages_out[3]}\n${messages_out[2]}")
	diff \
		<(run "read" --output json_oneline --mode backward --count 2 --position 3 "$stream_name") \
		<(echo -e "${messages_out[3]}\n${messages_out[2]}")
	diff \
		<(run "read" --output json_oneline --mode backward --count 2 --position 2 "$stream_name") \
		<(echo -e "${messages_out[2]}\n${messages_out[1]}")
	diff \
		<(run "read" --output json_oneline --mode backward --count 2 --position 1 "$stream_name") \
		<(echo -e "${messages_out[1]}\n${messages_out[0]}")
	diff \
		<(run "read" --output json_oneline --mode backward --count 2 --position first "$stream_name") \
		<(echo -e "${messages_out[0]}")

	run "write" --json "$stream_name" "any" test_message "${messages_in[4]}"

	diff \
		<(run "read" --output json_oneline --mode forward-once --count 10 --position first "$stream_name") \
		<(echo -e "${messages_out[0]}\n${messages_out[1]}\n${messages_out[2]}\n${messages_out[3]}\n${messages_out[4]}")

	diff \
		<(run "read" --output json_oneline --mode backward --count 10 --position last "$stream_name") \
		<(echo -e "${messages_out[4]}\n${messages_out[3]}\n${messages_out[2]}\n${messages_out[1]}\n${messages_out[0]}")
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
