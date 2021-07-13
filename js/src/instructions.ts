import {
    AccountMeta,
    PublicKey,
    SystemProgram,
    SYSVAR_CLOCK_PUBKEY,
    SYSVAR_RENT_PUBKEY,
    TransactionInstruction
} from '@solana/web3.js';
import { Treasury } from './treasury';
import * as borsh from 'borsh';
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    Token,
    TOKEN_PROGRAM_ID
} from '@solana/spl-token';

export enum TreasuryInstructions {
    Initialize,
    CreateSimpleTreasury,
    CreatedVestedTreasury,
    WithdrawVested
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

export class VestedSchema {
    instructionId: number;
    amount: bigint;
    period: number;
    percentage: number;

    static schema: borsh.Schema = new Map([
        [
            SimpleSchema,
            {
                kind: 'struct',
                fields: [
                    ['instructionId', 'u8'],
                    ['amount', 'u64'],
                    ['period', 'u64'],
                    ['percentage', 'u16']
                ]
            }
        ]
    ]);

    constructor(
        id: number,
        amount: bigint,
        period: number,
        percentage: number
    ) {
        this.instructionId = id;
        this.amount = amount;
        this.period = period;
        this.percentage = percentage;
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
            am(SYSVAR_RENT_PUBKEY, false, false),
            am(SystemProgram.programId, false, false)
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
        const fund = await Treasury.treasuryAssociatedAccount(treasury, mint);
        const keys: AccountMeta[] = [
            am(funder, true, true),
            am(authority, true, false),
            am(treasury, false, true),
            am(fund, false, true),
            am(mint, false, false),
            am(settingsId, false, true),
            am(SYSVAR_RENT_PUBKEY, false, false),
            am(TOKEN_PROGRAM_ID, false, false),
            am(SystemProgram.programId, false, false)
        ];

        const instruction = new SimpleSchema(
            TreasuryInstructions.CreateSimpleTreasury
        );
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

    public static async CreateSimpleTreasuryAndFundAccount(
        programId: PublicKey,
        funder: PublicKey,
        authority: PublicKey,
        mint: PublicKey
    ): Promise<TransactionInstruction[]> {
        const treasury = await Treasury.simpleTreasuryId(authority, programId);
        const fund = await Treasury.treasuryAssociatedAccount(treasury, mint);

        return [
            Token.createAssociatedTokenAccountInstruction(
                ASSOCIATED_TOKEN_PROGRAM_ID,
                TOKEN_PROGRAM_ID,
                mint,
                fund,
                treasury,
                funder
            ),
            await TreasuryInstruction.CreateSimpleTreasury(
                programId,
                funder,
                authority,
                mint
            )
        ];
    }

    public static async CreateVestedTreasury(
        programId: PublicKey,
        funder: PublicKey,
        authority: PublicKey,
        mint: PublicKey,
        amount: bigint,
        period: number,
        percentage: number
    ): Promise<TransactionInstruction> {
        const settingsId = await Treasury.settingsId(programId);
        const treasury = await Treasury.vestedTreasuryId(authority, programId);
        const fund = await Treasury.treasuryAssociatedAccount(treasury, mint);

        const keys: AccountMeta[] = [
            am(funder, true, true),
            am(authority, true, false),
            am(treasury, false, true),
            am(fund, false, true),
            am(mint, false, false),
            am(settingsId, false, false),
            am(SYSVAR_RENT_PUBKEY, false, false),
            am(SYSVAR_CLOCK_PUBKEY, false, false),
            am(SystemProgram.programId, false, false)
        ];

        const instruction = new VestedSchema(
            TreasuryInstructions.CreatedVestedTreasury,
            amount,
            period,
            percentage
        );
        const instructionData = borsh.serialize(
            VestedSchema.schema,
            instruction
        );

        return new TransactionInstruction({
            keys: keys,
            programId,
            data: Buffer.from(instructionData)
        });
    }

    public static async WithdrawVested(
        programId: PublicKey,
        funder: PublicKey,
        authority: PublicKey,
        mint: PublicKey
    ): Promise<TransactionInstruction> {
        const settingsId = await Treasury.settingsId(programId);
        const treasury = await Treasury.vestedTreasuryId(authority, programId);
        const fund = await Treasury.treasuryAssociatedAccount(treasury, mint);
        const recipient = await Token.getAssociatedTokenAddress(
            ASSOCIATED_TOKEN_PROGRAM_ID,
            TOKEN_PROGRAM_ID,
            mint,
            authority
        );

        const keys: AccountMeta[] = [
            am(funder, true, true),
            am(authority, true, false),
            am(recipient, false, true),
            am(treasury, false, true),
            am(fund, false, true),
            am(mint, false, false),
            am(settingsId, false, false),
            am(SYSVAR_CLOCK_PUBKEY, false, false),
            am(TOKEN_PROGRAM_ID, false, false),
            am(SystemProgram.programId, false, false)
        ];

        const instruction = new SimpleSchema(
            TreasuryInstructions.WithdrawVested
        );
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
