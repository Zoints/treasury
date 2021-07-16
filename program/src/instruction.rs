use borsh::{BorshDeserialize, BorshSerialize};

use crate::account::SimpleTreasuryMode;

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum TreasuryInstruction {
    /// Initialize the Treasury Program
    ///
    /// This only needs to be done once to set the initial parameters
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer, writable]` The account funding the instruction
    ///   1. `[]` The ZEE Token mint
    ///   2. `[writable]` The global settings program account
    ///   3. `[]` Rent sysvar
    ///   4. `[]` System Program
    Initialize,
    /// Create Simple Treasury
    ///
    /// Initializes a treasury for a specific user. SOL fees are paid by the funder.
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer, writable]` The account funding the instruction
    ///   1. `[signer]` The authority that controls the treasury
    ///   2. `[writable]` The treasury account for the authority
    ///   3. `[]` The ZEE token mint
    ///   4. `[]` The global settings program account
    ///   5. `[]` Rent sysvar
    ///   6. `[]` System Program
    CreateSimpleTreasury { mode: SimpleTreasuryMode },
    /// Created Vested Treasury
    ///
    /// Initializes a vested treasury. SOL fees are paid by the funder.
    //
    /// Accounts expected by this instruction:
    ///   0. `[signer, writable]` The account funding the instruction
    ///   1. `[]` The authority that controls the treasury
    ///   2. `[writable]` The treasury account for the authority
    ///   3. `[]` The ZEE token mint
    ///   4. `[]` The global settings program account
    ///   5. `[]` Rent sysvar
    ///   6. `[]` Clock sysvar
    ///   7. `[]` System Program
    CreatedVestedTreaury {
        amount: u64,
        period: u64,
        percentage: u16,
    },
    /// Withdraw from a Vested Treasury
    ///
    /// Withdraw everything that is possible to currently withdraw from the vested treasury.
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer, writable]` The account funding the instruction
    ///   1. `[signer]` The authority that controls the treasury
    ///   2. `[writable]` The recipient token address (must be owned by authority)
    ///   3. `[writable]` The treasury account
    ///   4. `[]` The treasury's fund authority
    ///   5. `[]` The treasury's fund associated account
    ///   6. `[]` The ZEE token mint
    ///   7. `[]` The global settings program account
    ///   8. `[]` Clock sysvar
    ///   9. `[]` SPL Token Program
    ///  10. `[]` System Program
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
