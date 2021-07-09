#!/usr/bin/env bash

set -e
cd "$(dirname "$0")"
PANOPTES_ROOT="$(cd ../..; pwd)"

logDir="$PWD"/logs
rm -rf "$logDir"
mkdir "$logDir"

solanaInstallDataDir=$PWD/releases
solanaInstallGlobalOpts=(
  --data-dir "$solanaInstallDataDir"
  --config "$solanaInstallDataDir"/config.yml
  --no-modify-path
)

# Install all the panoptes versions
bootstrapInstall() {
  declare v=$1
  if [[ ! -h $solanaInstallDataDir/active_release ]]; then
    sh "$PANOPTES_ROOT"/install/panoptes-install-init.sh "$v" "${solanaInstallGlobalOpts[@]}"
  fi
  export PATH="$solanaInstallDataDir/active_release/bin/:$PATH"
}

bootstrapInstall "edge"
panoptes-install-init --version
panoptes-install-init edge
panoptes-gossip --version
panoptes-dos --version

killall panoptes-gossip || true
panoptes-gossip spy --gossip-port 10015 > "$logDir"/gossip.log 2>&1 &
solanaGossipPid=$!
echo "panoptes-gossip pid: $solanaGossipPid"
sleep 5
panoptes-dos --mode gossip --data-type random --data-size 1232 &
dosPid=$!
echo "panoptes-dos pid: $dosPid"

pass=true

SECONDS=
while ((SECONDS < 600)); do
  if ! kill -0 $solanaGossipPid; then
    echo "panoptes-gossip is no longer running after $SECONDS seconds"
    pass=false
    break
  fi
  if ! kill -0 $dosPid; then
    echo "panoptes-dos is no longer running after $SECONDS seconds"
    pass=false
    break
  fi
  sleep 1
done

kill $solanaGossipPid || true
kill $dosPid || true
wait || true

$pass && echo Pass
