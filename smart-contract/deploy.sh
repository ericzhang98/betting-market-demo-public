#!/bin/bash

cargo build-bpf && solana program deploy ./target/deploy/betting_market.so --program-id GWQx5TcgfjYukS8tk6ZyL4Gz5Q9itjzeTKpD3UfLJyNr
