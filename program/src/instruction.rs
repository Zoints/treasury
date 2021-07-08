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
    ///   2. `[signer]` The authority that sets fees
    ///   3. `[]` The address that receives fees (must be valid ZEE associated account)
    ///   4. `[writable]` The global settings program account
    ///   5. `[]` Rent sysvar
    Initialize { fee_user: u64, fee_zoints: u64 },
    /// Create Simple Treasury
    ///
    /// Initializes a treasury for a specific user. SOL fees are paid by the funder
    /// and ZEE fees are paid by the creator
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer]` The account funding the instruction
    ///   1. `[signer]` The creator
    ///   2. `[writable]` The creator's ZEE associated token account
    ///   3. `[]` The ZEE token mint
    ///   4. `[]` The global settings program account
    ///   5. `[writable]` The fee recipient address
    ///   6. `[]` Rent sysvar
    ///   7. `[]` The SPL Token program
    ///   8. `[]` System Program
    CreateSimpleTreasury,
    /// Update Fees and Fee Recipient
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer]` The account funding the instruction
    ///   1. `[signer]` The authority that sets fees
    ///   2. `[]` The address that receives fees (must be valid ZEE associated account)
    ///   3. `[writable]` The global settings program account
    UpdateFees { fee_user: u64, fee_zoints: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test_serialize_instruction_init() {
        let data = vec![
            0, 0x5F, 0xCA, 0x12, 0, 0, 0, 0, 0, 0x96, 0xAD, 0x1D, 0x14, 0x2, 0, 0, 0,
        ];

        let instruction = TreasuryInstruction::Initialize {
            fee_user: 1231455,
            fee_zoints: 8927423894,
        };

        let serialized = instruction.try_to_vec().unwrap();
        assert_eq!(data, serialized);
        let decoded = TreasuryInstruction::try_from_slice(&serialized).unwrap();
        assert_eq!(instruction, decoded);
    }
}
