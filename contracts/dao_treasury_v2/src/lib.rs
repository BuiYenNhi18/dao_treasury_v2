#![no_std]

//! # dao_treasury_v2
//!
//! A minimal on-chain treasury for a community DAO. Members deposit
//! funds into a shared pool, propose spend payouts to a recipient,
//! vote on those proposals, and an admin can pause / unpause the
//! contract in an emergency. No real native XLM transfer is executed
//! on-chain — the contract updates an internal ledger (mirroring what
//! a real `token.transfer` integration would do) so the full proposal
//! / vote / execute flow can be exercised end-to-end on Testnet.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, Symbol};

/// Storage keys for the treasury contract.
#[contracttype]
pub enum DataKey {
    /// Address of the admin that can pause / unpause the contract.
    Admin,
    /// Boolean flag indicating whether the contract is paused.
    Paused,
    /// Map of registered DAO members.
    Members,
    /// Map of per-member deposit balances.
    Balances,
    /// Aggregate treasury balance (in the same unit as deposits).
    Treasury,
    /// Proposal record keyed by numeric `proposal_id`.
    Proposal(u64),
    /// Map of voters that have voted on a given proposal.
    Voted(u64),
}

/// A spending proposal: who, how much, to whom, and the running tally.
#[contracttype]
pub struct Proposal {
    pub proposer: Address,
    pub recipient: Address,
    pub amount: i128,
    pub reason: Symbol,
    pub approvals: u32,
    pub rejections: u32,
    pub executed: bool,
}

#[contract]
pub struct DaoTreasuryV2;

#[contractimpl]
impl DaoTreasuryV2 {
    /// Initialize the treasury with an `admin` and a founding `member`.
    /// Can only be called once. The admin is granted pause authority and
    /// the founder is auto-registered as a DAO member.
    pub fn initialize(env: Env, admin: Address, founder: Address) {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);

        let mut members: Map<Address, bool> = Map::new(&env);
        members.set(founder, true);
        env.storage().instance().set(&DataKey::Members, &members);

        let balances: Map<Address, i128> = Map::new(&env);
        env.storage().instance().set(&DataKey::Balances, &balances);
        env.storage().instance().set(&DataKey::Treasury, &0i128);
    }

    /// Register a new DAO member. Admin only.
    pub fn add_member(env: Env, admin: Address, member: Address) {
        admin.require_auth();
        let stored: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        if stored != admin {
            panic!("not admin");
        }
        let mut members: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Members)
            .unwrap_or_else(|| Map::new(&env));
        members.set(member, true);
        env.storage().instance().set(&DataKey::Members, &members);
    }

    /// Member deposits `amount` into the shared treasury. The member's
    /// per-account balance and the aggregate treasury are both updated.
    /// Requires the member's authorization.
    pub fn deposit(env: Env, member: Address, amount: i128) {
        Self::assert_not_paused(&env);
        member.require_auth();
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let members: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Members)
            .unwrap_or_else(|| Map::new(&env));
        if !members.get(member.clone()).unwrap_or(false) {
            panic!("not a member");
        }

        let mut balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&DataKey::Balances)
            .unwrap_or_else(|| Map::new(&env));
        let current = balances.get(member.clone()).unwrap_or(0);
        balances.set(member, current + amount);
        env.storage().instance().set(&DataKey::Balances, &balances);

        let treasury: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Treasury)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::Treasury, &(treasury + amount));
    }

    /// Open a new spending proposal. The proposer's own vote is
    /// recorded automatically as an approval. `reason` is a short
    /// symbolic label describing the payout.
    pub fn propose_spend(
        env: Env,
        proposer: Address,
        proposal_id: u64,
        recipient: Address,
        amount: i128,
        reason: Symbol,
    ) {
        Self::assert_not_paused(&env);
        proposer.require_auth();

        if env
            .storage()
            .instance()
            .has(&DataKey::Proposal(proposal_id))
        {
            panic!("proposal already exists");
        }
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let members: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Members)
            .unwrap_or_else(|| Map::new(&env));
        if !members.get(proposer.clone()).unwrap_or(false) {
            panic!("proposer is not a member");
        }

        let proposal = Proposal {
            proposer: proposer.clone(),
            recipient,
            amount,
            reason,
            approvals: 1,
            rejections: 0,
            executed: false,
        };
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        let mut voted: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Voted(proposal_id))
            .unwrap_or_else(|| Map::new(&env));
        voted.set(proposer, true);
        env.storage()
            .instance()
            .set(&DataKey::Voted(proposal_id), &voted);
    }

    /// Cast a vote on `proposal_id`. `approve = true` counts as a yes,
    /// `false` counts as a no. Each member may vote at most once.
    pub fn vote(env: Env, voter: Address, proposal_id: u64, approve: bool) {
        Self::assert_not_paused(&env);
        voter.require_auth();

        let members: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Members)
            .unwrap_or_else(|| Map::new(&env));
        if !members.get(voter.clone()).unwrap_or(false) {
            panic!("not a member");
        }

        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        let mut voted: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Voted(proposal_id))
            .unwrap_or_else(|| Map::new(&env));
        if voted.get(voter.clone()).unwrap_or(false) {
            panic!("already voted");
        }
        voted.set(voter, true);
        env.storage()
            .instance()
            .set(&DataKey::Voted(proposal_id), &voted);

        if approve {
            proposal.approvals = proposal.approvals.saturating_add(1);
        } else {
            proposal.rejections = proposal.rejections.saturating_add(1);
        }
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);
    }

    /// Execute `proposal_id` if it has reached a simple majority of the
    /// registered members and the treasury has enough funds. On success
    /// the payout is recorded by deducting the amount from the treasury
    /// balance; the recipient is published in the proposal state. No
    /// real native asset transfer is performed.
    pub fn execute(env: Env, _anyone: Address, proposal_id: u64) {
        Self::assert_not_paused(&env);

        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");
        if proposal.executed {
            panic!("already executed");
        }

        let members: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::Members)
            .unwrap_or_else(|| Map::new(&env));
        let total_members: u32 = members.len();

        if proposal.approvals <= proposal.rejections {
            panic!("proposal did not pass");
        }
        let quorum: u32 = (total_members / 2) + 1;
        if proposal.approvals < quorum {
            panic!("quorum not reached");
        }

        let treasury: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Treasury)
            .unwrap_or(0);
        if proposal.amount > treasury {
            panic!("insufficient treasury");
        }
        env.storage()
            .instance()
            .set(&DataKey::Treasury, &(treasury - proposal.amount));

        proposal.executed = true;
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);
    }

    /// Emergency pause. Blocks all state-mutating calls. Admin only.
    pub fn pause(env: Env, admin: Address) {
        admin.require_auth();
        let stored: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        if stored != admin {
            panic!("not admin");
        }
        env.storage().instance().set(&DataKey::Paused, &true);
    }

    /// Resume normal operation. Admin only.
    pub fn unpause(env: Env, admin: Address) {
        admin.require_auth();
        let stored: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        if stored != admin {
            panic!("not admin");
        }
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    /// Return the current treasury balance encoded as a `u32`. Useful
    /// for off-chain explorers and dashboards.
    pub fn get_treasury_balance(env: Env) -> u32 {
        let treasury: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Treasury)
            .unwrap_or(0);
        if treasury < 0 {
            panic!("treasury underflow");
        }
        if treasury > u32::MAX as i128 {
            panic!("treasury overflow u32");
        }
        treasury as u32
    }

    /// Read-only helper that returns a proposal by id.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        env.storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found")
    }

    /// Read-only helper that returns the aggregate of deposit balances
    /// for a single member. Returns 0 if the member has no deposits.
    pub fn get_member_balance(env: Env, member: Address) -> i128 {
        let balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&DataKey::Balances)
            .unwrap_or_else(|| Map::new(&env));
        balances.get(member).unwrap_or(0)
    }

    /// Read-only helper: `true` if the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    // --- internal helpers -------------------------------------------------

    /// Reverts the transaction if the contract is currently paused.
    fn assert_not_paused(env: &Env) {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            panic!("contract is paused");
        }
    }
}
