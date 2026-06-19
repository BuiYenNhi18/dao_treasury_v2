# dao_treasury_v2

## Project Title
dao_treasury_v2 — Community DAO Treasury on Stellar / Soroban

## Project Description
`dao_treasury_v2` is a small, self-contained on-chain treasury for a community DAO, built as a Soroban smart contract. The contract lets registered members deposit funds into a shared pool, propose spend payouts to a recipient with a short reason, and vote on those proposals. When a proposal reaches a simple majority of the registered members and the treasury has enough funds, anyone can call `execute` to record the payout. The admin can pause and unpause the contract in an emergency. This is the second iteration of an internal "DAO treasury" experiment — the first version was a single-file demo, and `v2` adds explicit member registration, a separate `Voted` ledger per proposal, an admin-controlled pause, and read-only helpers for off-chain dashboards.

## Project Vision
The long-term vision is a transparent, low-cost community fund that any small group — a student club, a study group, a hackathon team, a co-op — can spin up in minutes. By encoding the rules of "who can deposit, who can propose, who can vote, and what counts as a passing proposal" directly into a Soroban contract, the treasury becomes auditable, censorship-resistant, and cheap to run. Future iterations will plug in a real Stellar asset (USDC or a custom SAC), add time-locked proposals, and provide a small web UI for non-technical members to interact with the contract through Freighter.

## Key Features
- **Member-gated deposits** — only registered members can top up the shared treasury via `deposit`; per-member balances are tracked on-chain.
- **Spending proposals with reasons** — any member can open a proposal via `propose_spend`, naming a `recipient`, an `amount`, and a short `reason` symbol; the proposer's own vote is recorded as an implicit approval.
- **One-member-one-vote tally** — `vote` lets a member approve or reject a proposal, and a per-proposal ledger prevents double voting.
- **Quorum-based execution** — `execute` succeeds only when approvals strictly exceed rejections and a simple majority of registered members has approved; on success the payout is recorded by reducing the treasury balance.
- **Emergency pause** — the admin can `pause` and `unpause` the contract, blocking all state-mutating calls until normal operation is resumed.
- **Read-only helpers** — `get_treasury_balance`, `get_proposal`, `get_member_balance`, and `is_paused` make it easy to build dashboards, explorers, and bots on top of the contract.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** community dApp — see `contracts/dao_treasury_v2/src/lib.rs` for the full dao_treasury_v2 business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CCTK2KUQX43QUZQOSDGUTY7VGYVTSI4ZLDJMHPVXS2OYON3FTG7GX4JP`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/4531bbe30d81a7b36a245e1613ccb026ca88ab62ce7d8d58ee0bc9f54a421eb7`


## Future Scope
- **Real asset integration** — replace the internal ledger with an actual Stellar asset contract (a SAC for USDC or a custom token) so `deposit` and `execute` move real on-chain value.
- **Time-locked proposals** — add a voting window (open / close timestamps) and a queue so proposals can only be executed after a delay.
- **Delegation and weighted voting** — let members delegate votes or weight votes by deposit amount.
- **On-chain proposal metadata** — store a longer description (e.g. a short text hash) and an optional external URL pointing to a forum thread.
- **Frontend dApp** — a small static web page using `@stellar/freighter-api` and `@stellar/stellar-sdk` so non-technical members can deposit, propose, vote, and execute from the browser.
- **Tests and formal verification** — full `cargo test` coverage of the happy path and every revert branch, plus property-based tests for the voting tally.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `dao_treasury_v2` (community)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
