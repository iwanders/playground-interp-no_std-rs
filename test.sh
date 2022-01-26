#!/bin/bash
set -v


echo ${__CARGO_TEST_CHANNEL_OVERRIDE_DO_NOT_USE_THIS}
echo ${CARGO_UNSTABLE_TARGET_APPLIES_TO_HOST}
echo ${CARGO_TARGET_APPLIES_TO_HOST}

rm -rf target
cargo +nightly build --target x86_64-unknown-linux-gnu
objdump -T ./target/debug/build/compiler_builtins-1d5a76e82d9e8bdd/build-script-build  | grep main
cargo b -Zunstable-options --build-plan  --target x86_64-unknown-linux-gnu > /tmp/build_plan_with_target.json


rm -rf target
export __CARGO_TEST_CHANNEL_OVERRIDE_DO_NOT_USE_THIS="nightly"
export CARGO_UNSTABLE_TARGET_APPLIES_TO_HOST="true"
export CARGO_TARGET_APPLIES_TO_HOST="false"

echo ${__CARGO_TEST_CHANNEL_OVERRIDE_DO_NOT_USE_THIS}
echo ${CARGO_UNSTABLE_TARGET_APPLIES_TO_HOST}
echo ${CARGO_TARGET_APPLIES_TO_HOST}


echo "--"
cargo build
echo "--"
objdump -T ./target/debug/build/compiler_builtins-1d5a76e82d9e8bdd/build-script-build  | grep main
cargo b -Zunstable-options --build-plan > /tmp/build_plan.json
