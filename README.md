### Betting market demo
This project is still very much a WIP, but if you wanted to just take a look at the code, the processing logic is all in `smart-contract/src/process.rs` and the instruction API is defined in `smart-contract/src/instruction.rs`.
If you want to deploy it, just run `./smart-contract/new_deploy.sh` (make sure you're on the devnet or testnet!!!) with a valid solana account keypair setup through the solana cli.

There's a lot of stuff hardcoded to the frontend, so it's probly too annoying to set up right now cuz you need to replace a bunch of the keypairs and public keys in `frontend/src/common.tsx` with your own and also make a fake USDC token that you can mint to yourself and also associated token accounts for the different tokens for each user, but I will update this with how to set up the frontend after I add in some wallet adapter code to make things easier.

If you really want to get it up and running just to play around with, feel free to message me and I can walk you through the setup for the frontend.
