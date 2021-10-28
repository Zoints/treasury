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
import { SimpleTreasuryMode } from './accounts';

export enum TreasuryInstructions {
    Initialize,
    CreateSimpleTreasury,
    WithdrawSimple,
    CreatedVestedTreasury,
    WithdrawVested
}

export class BasicSchema {
    instructionId: number;

    static schema: borsh.Schema = new Map([
        [
            BasicSchema,
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

export class SimpleSchema {
    instructionId: number;
    mode: SimpleTreasuryMode;

    static schema: borsh.Schema = new Map([
        [
            SimpleSchema,
            {
                kind: 'struct',
                fields: [
                    ['instructionId', 'u8'],
                    ['mode', 'u8']
                ]
            }
        ]
    ]);

    constructor(id: number, mode: SimpleTreasuryMode) {
        this.instructionId = id;
        this.mode = mode;
    }
}

export class SimpleWithdrawSchema {
    instructionId: number;
    amount: bigint;

    static schema: borsh.Schema = new Map([
        [
            SimpleWithdrawSchema,
            {
                kind: 'struct',
                fields: [
                    ['instructionId', 'u8'],
                    ['amount', 'u64']
                ]
            }
        ]
    ]);

    constructor(id: number, amount: bigint) {
        this.instructionId = id;
        this.amount = amount;
    }
}

export class VestedSchema {
    instructionId: number;
    amount: bigint;
    period: number;
    percentage: number;

    static schema: borsh.Schema = new Map([
        [
            VestedSchema,
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

        const instruction = new BasicSchema(TreasuryInstructions.Initialize);
        const instructionData = borsh.serialize(
            BasicSchema.schema,
            instruction
        );

        return new TransactionInstruction({
            keys: keys,
            programId,
            data: Buffer.from(instructionData)
        });
    }

    private static async CreateSimpleTreasury(
        programId: PublicKey,
        funder: PublicKey,
        treasury: PublicKey,
        authority: PublicKey,
        mode: SimpleTreasuryMode = SimpleTreasuryMode.Locked
    ): Promise<TransactionInstruction> {
        const keys: AccountMeta[] = [
            am(funder, true, true),
            am(authority, false, false),
            am(treasury, true, true),
            am(SYSVAR_RENT_PUBKEY, false, false),
            am(TOKEN_PROGRAM_ID, false, false),
            am(SystemProgram.programId, false, false)
        ];

        const instruction = new SimpleSchema(
            TreasuryInstructions.CreateSimpleTreasury,
            mode
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
        treasury: PublicKey,
        authority: PublicKey,
        mint: PublicKey,
        mode: SimpleTreasuryMode = SimpleTreasuryMode.Locked
    ): Promise<TransactionInstruction[]> {
        const fund = await Treasury.simpleTreasuryAssociatedAccount(
            treasury,
            mint,
            programId
        );

        return [
            Token.createAssociatedTokenAccountInstruction(
                ASSOCIATED_TOKEN_PROGRAM_ID,
                TOKEN_PROGRAM_ID,
                mint,
                fund.fund,
                fund.authority,
                funder
            ),
            await TreasuryInstruction.CreateSimpleTreasury(
                programId,
                funder,
                treasury,
                authority,
                mode
            )
        ];
    }

    public static async WithdrawSimple(
        programId: PublicKey,
        funder: PublicKey,
        treasury: PublicKey,
        authority: PublicKey,
        associated: PublicKey,
        mint: PublicKey,
        amount: bigint
    ): Promise<TransactionInstruction> {
        const fund = await Treasury.simpleTreasuryAssociatedAccount(
            treasury,
            mint,
            programId
        );
        const settings = await Treasury.settingsId(programId);

        const keys: AccountMeta[] = [
            am(funder, true, true),
            am(authority, false, false),
            am(associated, false, true),
            am(treasury, false, false),
            am(fund.authority, false, false),
            am(fund.fund, false, true),
            am(mint, false, false),
            am(settings, false, false),
            am(TOKEN_PROGRAM_ID, false, false)
        ];

        const instruction = new SimpleWithdrawSchema(
            TreasuryInstructions.CreateSimpleTreasury,
            amount
        );
        const instructionData = borsh.serialize(
            SimpleWithdrawSchema.schema,
            instruction
        );

        return new TransactionInstruction({
            keys: keys,
            programId,
            data: Buffer.from(instructionData)
        });
    }

    private static CreateVestedTreasury(
        programId: PublicKey,
        funder: PublicKey,
        treasury: PublicKey,
        authority: PublicKey,
        amount: bigint,
        period: number,
        percentage: number
    ): TransactionInstruction {
        const keys: AccountMeta[] = [
            am(funder, true, true),
            am(authority, false, false),
            am(treasury, true, true),
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

    public static async CreateVestedTreasuryAndFundAccount(
        programId: PublicKey,
        funder: PublicKey,
        treasury: PublicKey,
        authority: PublicKey,
        mint: PublicKey,
        amount: bigint,
        period: number,
        percentage: number
    ): Promise<TransactionInstruction[]> {
        const fundAssoc = await Treasury.vestedTreasuryAssociatedAccount(
            treasury,
            mint,
            programId
        );

        return [
            Token.createAssociatedTokenAccountInstruction(
                ASSOCIATED_TOKEN_PROGRAM_ID,
                TOKEN_PROGRAM_ID,
                mint,
                fundAssoc.fund,
                fundAssoc.authority,
                funder
            ),
            await TreasuryInstruction.CreateVestedTreasury(
                programId,
                funder,
                treasury,
                authority,
                amount,
                period,
                percentage
            )
        ];
    }

    public static async WithdrawVested(
        programId: PublicKey,
        funder: PublicKey,
        treasury: PublicKey,
        authority: PublicKey,
        mint: PublicKey
    ): Promise<TransactionInstruction> {
        const settingsId = await Treasury.settingsId(programId);
        const fundAssoc = await Treasury.vestedTreasuryAssociatedAccount(
            treasury,
            mint,
            programId
        );
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
            am(fundAssoc.authority, false, false),
            am(fundAssoc.fund, false, true),
            am(mint, false, false),
            am(settingsId, false, false),
            am(SYSVAR_CLOCK_PUBKEY, false, false),
            am(TOKEN_PROGRAM_ID, false, false),
            am(SystemProgram.programId, false, false)
        ];

        const instruction = new BasicSchema(
            TreasuryInstructions.WithdrawVested
        );
        const instructionData = borsh.serialize(
            BasicSchema.schema,
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
