import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
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

export enum SimpleTreasuryMode {
    Locked
}

export class SimpleTreasury {
    public mode: SimpleTreasuryMode;
    public authority: PublicKey;

    static schema: borsh.Schema = new Map([
        [
            SimpleTreasury,
            {
                kind: 'struct',
                fields: [
                    ['mode', 'u16'],
                    ['authority', [32]]
                ]
            }
        ]
    ]);

    constructor(params: { mode: BN; authority: Uint8Array }) {
        switch (params.mode.toNumber()) {
            case 0:
                this.mode = SimpleTreasuryMode.Locked;
                break;
            default:
                throw new Error('invalid mode');
        }
        this.authority = new PublicKey(params.authority);
    }
}
