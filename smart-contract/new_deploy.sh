#!/bin/bash

cargo build-bpf && solana program deploy ./target/deploy/betting_market.so
