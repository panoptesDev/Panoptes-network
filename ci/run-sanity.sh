#!/usr/bin/env bash
set -e

cd "$(dirname "$0")/.."
# shellcheck source=multinode-demo/common.sh
source multinode-demo/common.sh

rm -rf config/run/init-completed config/ledger config/snapshot-ledger

timeout 120 ./run.sh &
pid=$!

attempts=20
while [[ ! -f config/run/init-completed ]]; do
  sleep 1
  if ((--attempts == 0)); then
     echo "Error: validator failed to boot"
     exit 1
  fi
done

snapshot_slot=1

# wait a bit longer than snapshot_slot
while [[ $($solana_cli --url http://localhost:8328 slot --commitment recent) -le $((snapshot_slot + 1)) ]]; do
  sleep 1
done
curl -X POST -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1, "method":"validatorExit"}' http://localhost:8328

wait $pid

$panoptes_ledger_tool create-snapshot --ledger config/ledger "$snapshot_slot" config/snapshot-ledger
cp config/ledger/genesis.tar.bz2 config/snapshot-ledger
$panoptes_ledger_tool verify --ledger config/snapshot-ledger
