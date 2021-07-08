import { Connection, PublicKey } from '@solana/web3.js';
import { Settings } from './';
import * as borsh from 'borsh';

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

        return borsh.deserialize(Settings.schema, Settings, account.data);
    }

    static async settingsId(programId: PublicKey): Promise<PublicKey> {
        return (
            await PublicKey.findProgramAddress(
                [Buffer.from('settings')],
                programId
            )
        )[0];
    }

    static async simpleTreasuryId(
        authority: PublicKey,
        programId: PublicKey
    ): Promise<PublicKey> {
        return (
            await PublicKey.findProgramAddress(
                [Buffer.from('simple'), authority.toBuffer()],
                programId
            )
        )[0];
    }

    static async simpleTreasuryFundId(
        authority: PublicKey,
        programId: PublicKey
    ): Promise<PublicKey> {
        return (
            await PublicKey.findProgramAddress(
                [Buffer.from('simple fund'), authority.toBuffer()],
                programId
            )
        )[0];
    }
}
