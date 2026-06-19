#![no_std]

//! # DelegateWeight
//!
//! A Soroban smart contract implementing **liquid democracy**: voters can
//! delegate their voting power to a delegate of their choice. The delegate
//! then exercises that aggregated weight when voting on proposals. Voters
//! may revoke their delegation at any time, restoring their own voting
//! power. The contract is self-contained: it stores weights, delegation
//! edges, and per-proposal tallies in on-chain storage and does not move
//! any native asset.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, Symbol};

/// Storage keys used by the contract. Using a `#[contracttype]` enum lets
/// us keep all per-address and per-proposal state in a single, typed
/// namespace backed by the contract's instance storage.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// A voter's own base voting weight (set via `set_weight`).
    Weight(Address),
    /// The address that a voter is currently delegating to. When a voter
    /// has no explicit delegate this is set to the voter themselves
    /// (self-delegation).
    Delegate(Address),
    /// The total voting weight currently delegated to a given delegate.
    /// This does NOT include the delegate's own base weight.
    DelegatedTo(Address),
    /// Whether a given delegate has already cast a vote on a proposal.
    Voted(Symbol, Address),
    /// Running "yes" tally for a proposal.
    TallyYes(Symbol),
    /// Running "no" tally for a proposal.
    TallyNo(Symbol),
}

#[contract]
pub struct DelegateWeight;

#[contractimpl]
impl DelegateWeight {
    // -----------------------------------------------------------------
    // State-mutating functions
    // -----------------------------------------------------------------

    /// Set a voter's own base voting weight. The caller (`voter`) must
    /// authorize the transaction. If the voter has not previously
    /// registered, this also initialises a self-delegation so the voter
    /// can vote on proposals with their own weight immediately.
    pub fn set_weight(env: Env, voter: Address, weight: u32) {
        voter.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Weight(voter.clone()), &weight);
        if env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::Delegate(voter.clone()))
            .is_none()
        {
            env.storage()
                .instance()
                .set(&DataKey::Delegate(voter), &voter);
        }
    }

    /// Delegate `weight` units of the caller's voting power to
    /// `delegate`. Any previous delegation made by the same voter is
    /// replaced. The delegated amount cannot exceed the voter's own
    /// base weight and must be strictly greater than zero. The voter
    /// must authorize the transaction.
    pub fn delegate(env: Env, voter: Address, delegate: Address, weight: u32) {
        voter.require_auth();

        if weight == 0 {
            panic!("delegated weight must be greater than zero");
        }

        let own_weight: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Weight(voter.clone()))
            .unwrap_or(0);
        if weight > own_weight {
            panic!("delegated weight exceeds your own voting weight");
        }

        // Remove the weight from the voter's previous delegate (if any).
        if let Some(prev) = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::Delegate(voter.clone()))
        {
            if prev != voter {
                let prev_total: u32 = env
                    .storage()
                    .instance()
                    .get(&DataKey::DelegatedTo(prev.clone()))
                    .unwrap_or(0);
                let new_total = prev_total.saturating_sub(weight);
                env.storage()
                    .instance()
                    .set(&DataKey::DelegatedTo(prev), &new_total);
            }
        }

        // Add the weight to the new delegate (if different from voter).
        if delegate != voter {
            let curr_total: u32 = env
                .storage()
                .instance()
                .get(&DataKey::DelegatedTo(delegate.clone()))
                .unwrap_or(0);
            let new_total = curr_total.saturating_add(weight);
            env.storage()
                .instance()
                .set(&DataKey::DelegatedTo(delegate.clone()), &new_total);
        }

        env.storage()
            .instance()
            .set(&DataKey::Delegate(voter), &delegate);
    }

    /// Revoke the active delegation from `voter` to `delegate`. The full
    /// base weight of the voter is subtracted from the delegate's
    /// running tally and the voter reverts to self-delegation. The
    /// voter must authorize the transaction.
    pub fn revoke(env: Env, voter: Address, delegate: Address) {
        voter.require_auth();

        if voter == delegate {
            panic!("cannot revoke a self-delegation");
        }

        let current: Address = env
            .storage()
            .instance()
            .get(&DataKey::Delegate(voter.clone()))
            .unwrap_or_else(|| panic!("voter has not registered a delegate"));
        if current != delegate {
            panic!("voter is not currently delegated to this address");
        }

        let own_weight: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Weight(voter.clone()))
            .unwrap_or(0);

        let delegate_total: u32 = env
            .storage()
            .instance()
            .get(&DataKey::DelegatedTo(delegate.clone()))
            .unwrap_or(0);
        let new_total = delegate_total.saturating_sub(own_weight);
        env.storage()
            .instance()
            .set(&DataKey::DelegatedTo(delegate), &new_total);

        // Reset to self-delegation so the voter can vote with their
        // own weight again.
        env.storage()
            .instance()
            .set(&DataKey::Delegate(voter), &voter);
    }

    /// `delegate` casts a vote on `proposal_id` with `choice` (the
    /// symbols `"yes"` or `"no"`). The vote's weight equals the
    /// delegate's own base weight plus the sum of all weights currently
    /// delegated to them. A delegate may only vote once per proposal.
    /// The delegate must authorize the transaction.
    pub fn cast_vote_with_delegation(
        env: Env,
        delegate: Address,
        proposal_id: Symbol,
        choice: Symbol,
    ) {
        delegate.require_auth();

        let voted_key = DataKey::Voted(proposal_id.clone(), delegate.clone());
        if env
            .storage()
            .instance()
            .get::<_, bool>(&voted_key)
            .unwrap_or(false)
        {
            panic!("delegate has already voted on this proposal");
        }

        let own_weight: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Weight(delegate.clone()))
            .unwrap_or(0);
        let delegated: u32 = env
            .storage()
            .instance()
            .get(&DataKey::DelegatedTo(delegate.clone()))
            .unwrap_or(0);
        let power = own_weight + delegated;
        if power == 0 {
            panic!("delegate has no voting power on this proposal");
        }

        let yes_sym = Symbol::new(&env, "yes");
        let no_sym = Symbol::new(&env, "no");
        if choice == yes_sym {
            let curr: u32 = env
                .storage()
                .instance()
                .get(&DataKey::TallyYes(proposal_id.clone()))
                .unwrap_or(0);
            env.storage()
                .instance()
                .set(&DataKey::TallyYes(proposal_id.clone()), &curr.saturating_add(power));
        } else if choice == no_sym {
            let curr: u32 = env
                .storage()
                .instance()
                .get(&DataKey::TallyNo(proposal_id.clone()))
                .unwrap_or(0);
            env.storage()
                .instance()
                .set(&DataKey::TallyNo(proposal_id.clone()), &curr.saturating_add(power));
        } else {
            panic!("invalid choice: must be the symbol 'yes' or 'no'");
        }

        env.storage().instance().set(&voted_key, &true);
    }

    // -----------------------------------------------------------------
    // View (read-only) functions
    // -----------------------------------------------------------------

    /// Return the total weight currently delegated to `delegate`. This
    /// excludes the delegate's own base weight; callers that want the
    /// full voting power used at vote-time should add their own weight
    /// (see `get_weight`).
    pub fn get_delegated_to(env: Env, delegate: Address) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::DelegatedTo(delegate))
            .unwrap_or(0)
    }

    /// Return the address that `voter` is currently delegating to. If
    /// the voter has never registered (no `set_weight` or `delegate`
    /// call), the function panics because no delegate exists.
    pub fn get_delegate_of(env: Env, voter: Address) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Delegate(voter))
            .unwrap_or_else(|| panic!("voter has not registered a delegate"))
    }

    /// Return a voter's own base voting weight (defaults to 0 if the
    /// voter has not called `set_weight`).
    pub fn get_weight(env: Env, voter: Address) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Weight(voter))
            .unwrap_or(0)
    }

    /// Return the running yes/no tally for `proposal_id` as a map
    /// keyed by the symbols `"yes"` and `"no"`.
    pub fn get_tally(env: Env, proposal_id: Symbol) -> Map<Symbol, u32> {
        let yes: u32 = env
            .storage()
            .instance()
            .get(&DataKey::TallyYes(proposal_id.clone()))
            .unwrap_or(0);
        let no: u32 = env
            .storage()
            .instance()
            .get(&DataKey::TallyNo(proposal_id.clone()))
            .unwrap_or(0);
        let mut result = Map::new(&env);
        result.set(Symbol::new(&env, "yes"), yes);
        result.set(Symbol::new(&env, "no"), no);
        result
    }
}
