script {
    use windfall::governance;

    /// Called by backend when a member votes on a proposal
    public entry fun submit_vote(
        voter: &signer,
        proposal_id: u64,
        approve: bool
    ) {
        governance::vote(voter, proposal_id, approve);
    }

    /// Called by backend when members initiate emergency veto
    public entry fun trigger_emergency_veto(
        member: &signer,
        proposal_id: u64
    ) {
        governance::emergency_veto(member, proposal_id);
    }

    /// Called by backend to execute an approved proposal
    public entry fun execute_approved_proposal(
        executor: &signer,
        proposal_id: u64
    ) {
        governance::execute_proposal(executor, proposal_id);
    }
} 