# |source| this file
#
# Common utilities shared by other scripts in this directory
#
# The following directive disable complaints about unused variables in this
# file:
# shellcheck disable=2034
#

# shellcheck source=net/common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. || exit 1; pwd)"/net/common.sh

if [[ $(uname) != Linux ]]; then
  # Protect against unsupported configurations to prevent non-obvious errors
  # later. Arguably these should be fatal errors but for now prefer tolerance.
  if [[ -n $PANOPTES_CUDA ]]; then
    echo "Warning: CUDA is not supported on $(uname)"
    PANOPTES_CUDA=
  fi
fi

if [[ -n $USE_INSTALL || ! -f "$PANOPTES_ROOT"/Cargo.toml ]]; then
  panoptes_program() {
    declare program="$1"
    if [[ -z $program ]]; then
      printf "panoptes"
    else
      printf "panoptes-%s" "$program"
    fi
  }
else
  panoptes_program() {
    declare program="$1"
    declare crate="$program"
    if [[ -z $program ]]; then
      crate="cli"
      program="panoptes"
    else
      program="panoptes-$program"
    fi

    if [[ -r "$PANOPTES_ROOT/$crate"/Cargo.toml ]]; then
      maybe_package="--package panoptes-$crate"
    fi
    if [[ -n $NDEBUG ]]; then
      maybe_release=--release
    fi
    declare manifest_path="--manifest-path=$PANOPTES_ROOT/$crate/Cargo.toml"
    printf "cargo $CARGO_TOOLCHAIN run $manifest_path $maybe_release $maybe_package --bin %s %s -- " "$program"
  }
fi

panoptes_bench_tps=$(panoptes_program bench-tps)
panoptes_faucet=$(panoptes_program faucet)
panoptes_validator=$(panoptes_program validator)
panoptes_validator_cuda="$panoptes_validator --cuda"
panoptes_genesis=$(panoptes_program genesis)
panoptes_gossip=$(panoptes_program gossip)
panoptes_keygen=$(panoptes_program keygen)
panoptes_ledger_tool=$(panoptes_program ledger-tool)
panoptes_cli=$(panoptes_program)

export RUST_BACKTRACE=1

default_arg() {
  declare name=$1
  declare value=$2

  for arg in "${args[@]}"; do
    if [[ $arg = "$name" ]]; then
      return
    fi
  done

  if [[ -n $value ]]; then
    args+=("$name" "$value")
  else
    args+=("$name")
  fi
}

replace_arg() {
  declare name=$1
  declare value=$2

  default_arg "$name" "$value"

  declare index=0
  for arg in "${args[@]}"; do
    index=$((index + 1))
    if [[ $arg = "$name" ]]; then
      args[$index]="$value"
    fi
  done
}
