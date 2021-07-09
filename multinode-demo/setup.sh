#!/usr/bin/env bash

here=$(dirname "$0")
# shellcheck source=multinode-demo/common.sh
source "$here"/common.sh

set -e

rm -rf "$PANOPTES_CONFIG_DIR"/bootstrap-validator
mkdir -p "$PANOPTES_CONFIG_DIR"/bootstrap-validator

# Create genesis ledger
if [[ -r $FAUCET_KEYPAIR ]]; then
  cp -f "$FAUCET_KEYPAIR" "$PANOPTES_CONFIG_DIR"/faucet.json
else
  $panoptes_keygen new --word-count 24 --no-passphrase -fso "$PANOPTES_CONFIG_DIR"/faucet.json
fi

if [[ -f $BOOTSTRAP_VALIDATOR_IDENTITY_KEYPAIR ]]; then
  cp -f "$BOOTSTRAP_VALIDATOR_IDENTITY_KEYPAIR" "$PANOPTES_CONFIG_DIR"/bootstrap-validator/identity.json
else
  $panoptes_keygen new --word-count 24 --no-passphrase -so "$PANOPTES_CONFIG_DIR"/bootstrap-validator/identity.json
fi

$panoptes_keygen new --word-count 24 --no-passphrase -so "$PANOPTES_CONFIG_DIR"/bootstrap-validator/vote-account.json
$panoptes_keygen new --word-count 24 --no-passphrase -so "$PANOPTES_CONFIG_DIR"/bootstrap-validator/stake-account.json

args=(
  "$@"
  --max-genesis-archive-unpacked-size 1073741824
  --enable-warmup-epochs
  --bootstrap-validator "$PANOPTES_CONFIG_DIR"/bootstrap-validator/identity.json
                        "$PANOPTES_CONFIG_DIR"/bootstrap-validator/vote-account.json
                        "$PANOPTES_CONFIG_DIR"/bootstrap-validator/stake-account.json
)

"$PANOPTES_ROOT"/fetch-spl.sh
if [[ -r spl-genesis-args.sh ]]; then
  SPL_GENESIS_ARGS=$(cat "$PANOPTES_ROOT"/spl-genesis-args.sh)
  #shellcheck disable=SC2207
  #shellcheck disable=SC2206
  args+=($SPL_GENESIS_ARGS)
fi

default_arg --ledger "$PANOPTES_CONFIG_DIR"/bootstrap-validator
default_arg --faucet-pubkey "$PANOPTES_CONFIG_DIR"/faucet.json
default_arg --faucet-lamports 500000000000000000
default_arg --hashes-per-tick auto
default_arg --cluster-type development

$panoptes_genesis "${args[@]}"
