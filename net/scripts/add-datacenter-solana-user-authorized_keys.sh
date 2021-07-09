#!/usr/bin/env bash
set -ex

cd "$(dirname "$0")"

# shellcheck source=net/scripts/solana-user-authorized_keys.sh
source solana-user-authorized_keys.sh

# solana-user-authorized_keys.sh defines the public keys for users that should
# automatically be granted access to ALL datacenter nodes.
for i in "${!PANOPTES_USERS[@]}"; do
  echo "environment=\"PANOPTES_USER=${PANOPTES_USERS[i]}\" ${PANOPTES_PUBKEYS[i]}"
done

