import {
    AccountMeta,
    PublicKey,
    SYSVAR_RENT_PUBKEY,
    TransactionInstruction
} from '@solana/web3.js';
import { Treasury } from './treasury';
import borsh from 'borsh';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

export enum TreasuryInstructions {
    Initialize,
    CreateSimpleTreasury
}

export class SimpleSchema {
    instructionId: number;

    static schema: borsh.Schema = new Map([
        [
            SimpleSchema,
            {
                kind: 'struct',
                fields: [['instructionId', 'u8']]
            }
        ]
    ]);

    constructor(id: number) {
        this.instructionId = id;
    }
}

export class TreasuryInstruction {
    public static async Initialize(
        programId: PublicKey,
        funder: PublicKey,
        mint: PublicKey
    ): Promise<TransactionInstruction> {
        const settingsId = await Treasury.settingsId(programId);

        const keys: AccountMeta[] = [
            am(funder, true, true),
            am(mint, false, false),
            am(settingsId, false, true),
            am(SYSVAR_RENT_PUBKEY, false, false)
            //            am(SystemProgram.programId, false, false)
        ];

        const instruction = new SimpleSchema(TreasuryInstructions.Initialize);
        const instructionData = borsh.serialize(
            SimpleSchema.schema,
            instruction
        );

        return new TransactionInstruction({
            keys: keys,
            programId,
            data: Buffer.from(instructionData)
        });
    }

    public static async CreateSimpleTreasury(
        programId: PublicKey,
        funder: PublicKey,
        authority: PublicKey,
        mint: PublicKey
    ): Promise<TransactionInstruction> {
        const settingsId = await Treasury.settingsId(programId);
        const treasury = await Treasury.simpleTreasuryId(authority, programId);
        const fund = await Treasury.simpleTreasuryFundId(authority, programId);

        const keys: AccountMeta[] = [
            am(funder, true, true),
            am(authority, true, false),
            am(treasury, false, true),
            am(fund, false, true),
            am(mint, false, false),
            am(settingsId, false, true),
            am(SYSVAR_RENT_PUBKEY, false, false),
            am(TOKEN_PROGRAM_ID, false, false)
            //            am(SystemProgram.programId, false, false)
        ];

        const instruction = new SimpleSchema(TreasuryInstructions.Initialize);
        const instructionData = borsh.serialize(
            SimpleSchema.schema,
            instruction
        );

        return new TransactionInstruction({
            keys: keys,
            programId,
            data: Buffer.from(instructionData)
        });
    }
}

function am(
    pubkey: PublicKey,
    isSigner: boolean,
    isWritable: boolean
): AccountMeta {
    return { pubkey, isSigner, isWritable };
}
