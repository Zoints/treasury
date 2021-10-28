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

    constructor(params: { instructionId: number }) {
        this.instructionId = params.instructionId;
    }
}

export class SimpleSchema {
    instructionId: number;
    mode: SimpleTreasuryMode;

    constructor(params: { instructionId: number; mode: SimpleTreasuryMode }) {
        this.instructionId = params.instructionId;
        this.mode = params.mode;
    }
}

export class SimpleWithdrawSchema {
    instructionId: number;
    amount: bigint;

    constructor(params: { instructionId: number; amount: bigint }) {
        this.instructionId = params.instructionId;
        this.amount = params.amount;
    }
}

export class VestedSchema {
    instructionId: number;
    amount: bigint;
    period: BigInt;
    percentage: number;

    constructor(params: {
        instructionId: number;
        amount: bigint;
        period: BigInt;
        percentage: number;
    }) {
        this.instructionId = params.instructionId;
        this.amount = params.amount;
        this.period = params.period;
        this.percentage = params.percentage;
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

        const instruction = new BasicSchema({
            instructionId: TreasuryInstructions.Initialize
        });
        const instructionData = borsh.serialize(
            INSTRUCTION_SCHEMA,
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

        const instruction = new SimpleSchema({
            instructionId: TreasuryInstructions.CreateSimpleTreasury,
            mode
        });
        const instructionData = borsh.serialize(
            INSTRUCTION_SCHEMA,
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

        const instruction = new SimpleWithdrawSchema({
            instructionId: TreasuryInstructions.CreateSimpleTreasury,
            amount
        });
        const instructionData = borsh.serialize(
            INSTRUCTION_SCHEMA,
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
        period: bigint,
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

        const instruction = new VestedSchema({
            instructionId: TreasuryInstructions.CreatedVestedTreasury,
            amount,
            period,
            percentage
        });
        const instructionData = borsh.serialize(
            INSTRUCTION_SCHEMA,
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
        period: bigint,
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
            TreasuryInstruction.CreateVestedTreasury(
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

        const instruction = new BasicSchema({
            instructionId: TreasuryInstructions.WithdrawVested
        });
        const instructionData = borsh.serialize(
            INSTRUCTION_SCHEMA,
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

export const INSTRUCTION_SCHEMA: borsh.Schema = new Map<any, any>([
    [
        BasicSchema,
        {
            kind: 'struct',
            fields: [['instructionId', 'u8']]
        }
    ],
    [
        SimpleSchema,
        {
            kind: 'struct',
            fields: [
                ['instructionId', 'u8'],
                ['mode', 'SimpleTreasuryMode']
            ]
        }
    ],
    [
        SimpleWithdrawSchema,
        {
            kind: 'struct',
            fields: [
                ['instructionId', 'u8'],
                ['amount', 'BigInt']
            ]
        }
    ],
    [
        VestedSchema,
        {
            kind: 'struct',
            fields: [
                ['instructionId', 'u8'],
                ['amount', 'BigInt'],
                ['period', 'BigInt'],
                ['percentage', 'u16']
            ]
        }
    ]
]);
