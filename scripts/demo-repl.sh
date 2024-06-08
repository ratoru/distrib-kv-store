#!/bin/bash

# Copied from other shell script
rpc() {
    local uri=$1
    local body="$2"

    echo '---'" http(:$uri, $body)"

    {
        if [ ".$body" = "." ]; then
            time curl --silent "127.0.0.1:$uri"
        else
            time curl --silent "127.0.0.1:$uri" -H "Content-Type: application/json" -d "$body"
        fi
    } | {
        if type jq > /dev/null 2>&1; then
            jq
        else
            cat
        fi
    }

    echo
    echo
}

# Function to handle user commands
handle_command() {
    local command=$1

    case $command in
        "write")
            local key=$2
            local value=$3
            local cluster_num=$4
            local node_num=$5
            local port=$((31000 + (cluster_num * 10) + (node_num)))
            rpc "$port/api/write" "{\"Set\":{\"key\":\"$key\",\"value\":\"$value\"}}"
            ;;
        "read")
            local key=$2
            local cluster_num=$3
            local node_num=$4

            local port=$((31000 + (cluster_num * 10) + (node_num)))
            rpc "$port/api/read" "\"$key\""
            ;;
        "exit")
            echo "Exiting REPL..."
            exit 0
            ;;
        *)
            echo "Unknown command: $command"
            ;;
    esac
}

# REPL loop
while true; do
    echo -n "kvstore> "
    read -r command args
    handle_command "$command" $args
done


