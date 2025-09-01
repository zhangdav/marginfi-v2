#!/usr/bin/env sh
ROOT=$(git rev-parse --show-toplevel)
cd $ROOT

program_lib_name=$1
cluster=$2

if [ -z "$program_lib_name" ] || [ -z "$cluster" ]; then
    echo "Usage: $0 <program_lib_name> <cluster>"
    exit 1
fi

if [ "$cluster" = "devnet" ]; then
    features="--features devnet --no-default-features"
elif [ "$cluster" = "localnet" ]; then
    features=""
else
    echo "Error: Unknown cluster: $cluster. Supported: devnet, localnet"
    exit 1
fi

cmd="anchor build -p $program_lib_name -- $features"
echo "Running: $cmd"
eval "$cmd"