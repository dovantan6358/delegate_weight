# delegate_weight

## Project Title
delegate_weight

## Project Description
`delegate_weight` is a Soroban smart contract that brings **liquid democracy** to the Stellar network. In traditional one-person-one-vote DAOs, low engagement and information asymmetry lead to poor decisions. In rigid token-weighted voting, a small number of whales can dominate outcomes. Liquid democracy lets every voter choose: vote directly on a proposal with their own weight, or delegate that weight to a more knowledgeable or trusted participant (a "delegate") who will vote on their behalf. Delegations are not permanent: a voter can revoke at any time and instantly regain direct voting control. The contract stores voting weights, delegation edges, and per-proposal tallies entirely on-chain in Soroban's instance storage and does not move any native asset (XLM).

## Project Vision
Our vision is to make on-chain governance more participatory, more legitimate, and more resilient. By giving every voter a one-tx escape hatch (revoke) and a one-tx upgrade path (re-delegate), liquid democracy keeps power accountable to the people who actually showed up. Long-term we want `delegate_weight` to serve as a reusable governance primitive for Stellar-based DAOs, community treasuries, university student councils, and grant programs — anywhere a transparent, weight-delegated vote is needed without giving up revocability.

## Key Features
- **Self-sovereign voting weight** — every voter calls `set_weight` to declare their own base weight and is automatically self-delegated.
- **One-call delegation** — `delegate(voter, delegate, weight)` redirects the caller's voting power to a chosen delegate, atomically replacing any previous delegation.
- **Instant revocation** — `revoke(voter, delegate)` returns the caller's full base weight to self-delegation in a single transaction.
- **Aggregated delegate voting** — `cast_vote_with_delegation` lets a delegate vote once per proposal, with the cast weight equal to their own base weight plus every weight currently delegated to them.
- **Transparent on-chain tallies** — `get_tally(proposal_id)` returns a yes/no map of running totals, all queryable from any Stellar client.
- **No native-asset movement** — the contract is pure governance logic; no XLM or token transfer is required, making it cheap to run on Stellar.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** community dApp — see `contracts/delegate_weight/src/lib.rs` for the full delegate_weight business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CBSBSCKQMSOS2WRPGYODU36G36GT5Z2V6CMWCWP762HAWUKRBCYAG65E`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/d055b91c49db0137a7a224359eb1520965a0dcb0bd71d6b117dc2827b6a22766`


## Future Scope
- **Time-bounded delegations** — let a voter set an expiry block/ledger so delegations auto-revoke.
- **Partial delegations & split voting** — allow a single voter to split their weight across multiple delegates.
- **Quadratic & reputation weights** — plug in alternative weight curves (e.g. quadratic) and on-chain reputation scores.
- **Proposal registry** — add a `create_proposal` function so proposals are first-class on-chain objects with metadata, deadlines, and admin-controlled lifecycles.
- **Delegate profiles & analytics** — emit events (`set_weight`, `delegate`, `revoke`, `cast_vote_with_delegation`) so off-chain indexers can build delegate scorecards.
- **Token-gated weight** — read a SAC (Stellar Asset Contract) balance to derive weight from token holdings instead of a self-declared value.
- **Frontend dApp** — a React + Freighter UI that lets users connect, set weight, delegate, and watch live tallies.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `delegate_weight` (community)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
