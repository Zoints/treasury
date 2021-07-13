use borsh::{BorshDeserialize, BorshSerialize};

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum TreasuryInstruction {
    /// Initialize the Treasury Program
    ///
    /// This only needs to be done once to set the initial parameters
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer]` The account funding the instruction
    ///   1. `[]` The ZEE Token mint
    ///   2. `[writable]` The global settings program account
    ///   3. `[]` Rent sysvar
    ///   4. `[]` System Program
    Initialize,
    /// Create Simple Treasury
    ///
    /// Initializes a treasury for a specific user. SOL fees are paid by the funder
    /// and ZEE fees are paid by the creator
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer]` The account funding the instruction
    ///   1. `[signer]` The authority that controls the treasury
    ///   2. `[writable]` The treasury account for the authority
    ///   3. `[writable]` The treasury account's fund address
    ///   4. `[]` The ZEE token mint
    ///   5. `[]` The global settings program account
    ///   6. `[]` Rent sysvar
    ///   7. `[]` The SPL Token program
    ///   8. `[]` System Program
    CreateSimpleTreasury,
    CreatedVestedTreaury {
        amount: u64,
        period: u64,
        percentage: u16,
    },
    WithdrawVested,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test_serialize_instruction_init() {
        let data = vec![0];

        let instruction = TreasuryInstruction::Initialize;

        let serialized = instruction.try_to_vec().unwrap();
        assert_eq!(data, serialized);
        let decoded = TreasuryInstruction::try_from_slice(&serialized).unwrap();
        assert_eq!(instruction, decoded);
    }
}
