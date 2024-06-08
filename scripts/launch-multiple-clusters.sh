#!/bin/bash

set -o errexit

cargo build

kill_all() {
    SERVICE='raft-kv'
    if [ "$(uname)" = "Darwin" ]; then
        if pgrep -xq -- "${SERVICE}"; then
            pkill -f "${SERVICE}"
        fi
        rm -r 127.0.0.1:*-db || echo "no db to clean"
    else
        set +e # killall will error if finds no process to kill
        killall "${SERVICE}"
        set -e
    fi
}

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

export RUST_LOG=trace
export RUST_BACKTRACE=full
bin=./target/debug/raft-kv

# Function to start a node
start_node() {
    local id=$1
    local http_port=$2
    local rpc_port=$3
    local log_file=$4

    nohup ${bin} --id ${id} --http-addr 127.0.0.1:${http_port} --rpc-addr 127.0.0.1:${rpc_port} > ${log_file} 2>&1 &
    sleep 1
    echo "Node ${id} started"
}

# Function to initialize a cluster
change_membership() {
    local init_port=$1
    local members=$2
    rpc ${init_port}/cluster/change-membership "${members}"
    sleep 1
    echo "Cluster initialized with members: ${members}"
}

# Main function to setup clusters
setup_clusters() {
    local num_clusters=3
    local nodes_per_cluster=3
    local base_http_port=31000
    local base_rpc_port=32000

    for (( c=1; c<=num_clusters; c++ ))
    do
        local member_ids='['
        local first_port=""

        for (( n=1; n<=nodes_per_cluster; n++ ))
        do
            local http_port=$((base_http_port + c * 10 + n))
            local rpc_port=$((base_rpc_port + c * 10 + n))
            local log_file="n${n}.log"

            start_node ${n} ${http_port} ${rpc_port} ${log_file}

            if [ "$n" -eq 1 ]; then
                first_port=${http_port}
            fi

            member_ids+="${n}"
            if [ "$n" -lt "${nodes_per_cluster}" ]; then
                member_ids+=", "
            fi
        done

        sleep 2
        rpc ${first_port}/cluster/init '{}'
        sleep 2

        for (( n=2; n<=nodes_per_cluster; n++ ))
        do
            local http_port=$((base_http_port + c * 10 + n))
            local rpc_port=$((base_rpc_port + c * 10 + n))

            sleep 1
            rpc ${first_port}/cluster/add-learner "[${n}, \"127.0.0.1:${http_port}\", \"127.0.0.1:${rpc_port}\"]"
            sleep 1
        done

        member_ids+=']'

        sleep 1
        change_membership ${first_port} "${member_ids}"
        sleep 1
        rpc ${first_port}/cluster/update-hash-ring "{\"version\":1.0,\"config_id\":0,\"list_ttl\":600,\"nodes\":[{\"addr\":\"127.0.0.1:31011\",\"relative_load\":0.33333334},{\"addr\":\"127.0.0.1:31021\",\"relative_load\":0.33333334},{\"addr\":\"127.0.0.1:31031\",\"relative_load\":0.33333334}]}"
        sleep 1

        echo "Cluster ${c} started with nodes ${member_ids}"
    done
}

echo "Killing all running raft-kv and cleaning up old data"

kill_all
sleep 1

if ls 127.0.0.1:*-db
then
    rm -r 127.0.0.1:*-db || echo "no db to clean"
fi

# Setup 3 clusters, each with 3 nodes
setup_clusters

trap 'echo "Killing all nodes..."; kill_all' INT

while true
do
    sleep 1
done