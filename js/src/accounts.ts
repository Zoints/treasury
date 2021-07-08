import { PublicKey } from '@solana/web3.js';
import borsh from 'borsh';

export class Settings {
    public token: PublicKey;

    static schema: borsh.Schema = new Map([
        [
            Settings,
            {
                kind: 'struct',
                fields: [['token', [32]]]
            }
        ]
    ]);

    constructor(params: { token: Uint8Array }) {
        this.token = new PublicKey(params.token);
    }
}
