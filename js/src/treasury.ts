import { Connection, PublicKey } from '@solana/web3.js';
import { ACCOUNT_SCHEMA, Settings } from './';
import * as borsh from 'borsh';
import { SimpleTreasury, VestedTreasury } from './accounts';
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID
} from '@solana/spl-token';

export class Treasury {
    connection: Connection;
    programId: PublicKey;

    constructor(connection: Connection, programId: PublicKey) {
        this.connection = connection;
        this.programId = programId;
    }

    public async getSettings(): Promise<Settings> {
        const settingsId = await Treasury.settingsId(this.programId);
        const account = await this.connection.getAccountInfo(settingsId);
        if (account === null)
            throw new Error('Unable to find settings account');

        return borsh.deserialize(ACCOUNT_SCHEMA, Settings, account.data);
    }

    public async getSimpleTreasury(
        treasuryId: PublicKey
    ): Promise<SimpleTreasury> {
        const account = await this.connection.getAccountInfo(treasuryId);
        if (account === null)
            throw new Error('Unable to find simple treasury account');

        return borsh.deserialize(ACCOUNT_SCHEMA, SimpleTreasury, account.data);
    }

    public async getVestedTreasury(
        treasuryId: PublicKey
    ): Promise<VestedTreasury> {
        const account = await this.connection.getAccountInfo(treasuryId);
        if (account === null)
            throw new Error('Unable to find vested treasury account');

        return borsh.deserialize(ACCOUNT_SCHEMA, VestedTreasury, account.data);
    }

    static async settingsId(programId: PublicKey): Promise<PublicKey> {
        return (
            await PublicKey.findProgramAddress(
                [Buffer.from('settings')],
                programId
            )
        )[0];
    }

    private static async treasuryAssociatedAccount(
        phrase: string,
        treasury: PublicKey,
        mint: PublicKey,
        programId: PublicKey
    ): Promise<{ authority: PublicKey; fund: PublicKey }> {
        const authority = (
            await PublicKey.findProgramAddress(
                [Buffer.from(phrase), treasury.toBuffer()],
                programId
            )
        )[0];

        return {
            authority,
            fund: (
                await PublicKey.findProgramAddress(
                    [
                        authority.toBuffer(),
                        TOKEN_PROGRAM_ID.toBuffer(),
                        mint.toBuffer()
                    ],
                    ASSOCIATED_TOKEN_PROGRAM_ID
                )
            )[0]
        };
    }

    static async simpleTreasuryAssociatedAccount(
        treasury: PublicKey,
        mint: PublicKey,
        programId: PublicKey
    ): Promise<{ authority: PublicKey; fund: PublicKey }> {
        return Treasury.treasuryAssociatedAccount(
            'simple authority',
            treasury,
            mint,
            programId
        );
    }

    static async vestedTreasuryAssociatedAccount(
        treasury: PublicKey,
        mint: PublicKey,
        programId: PublicKey
    ): Promise<{ authority: PublicKey; fund: PublicKey }> {
        return Treasury.treasuryAssociatedAccount(
            'vested authority',
            treasury,
            mint,
            programId
        );
    }
}
