import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import * as borsh from 'borsh';

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
                    ['mode', 'u8'],
                    ['authority', [32]]
                ]
            }
        ]
    ]);

    constructor(params: { mode: number; authority: Uint8Array }) {
        switch (params.mode) {
            case 0:
                this.mode = SimpleTreasuryMode.Locked;
                break;
            default:
                throw new Error('invalid mode');
        }
        this.authority = new PublicKey(params.authority);
    }
}

export class VestedTreasury {
    public authority: PublicKey;
    public initialAmount: BN;
    public start: Date;
    public vestmentPeriod: BN;
    public vestmentPercentage: number;
    public withdrawn: BN;

    static schema: borsh.Schema = new Map([
        [
            VestedTreasury,
            {
                kind: 'struct',
                fields: [
                    ['authority', [32]],
                    ['initialAmount', 'u64'],
                    ['start', 'u64'],
                    ['vestmentPeriod', 'u64'],
                    ['vestmentPercentage', 'u16'],
                    ['withdrawn', 'u64']
                ]
            }
        ]
    ]);

    constructor(params: {
        authority: Uint8Array;
        initialAmount: BN;
        start: BN;
        vestmentPeriod: BN;
        vestmentPercentage: number;
        withdrawn: BN;
    }) {
        this.authority = new PublicKey(params.authority);
        this.initialAmount = params.initialAmount;
        this.start = new Date(params.start.toNumber() * 1000);
        this.vestmentPeriod = params.vestmentPeriod;
        this.vestmentPercentage = params.vestmentPercentage;
        this.withdrawn = params.withdrawn;
    }

    public maximum_available(now: Date): BN {
        const period =
            Math.floor(now.getTime() / 1000) -
            Math.floor(this.start.getTime() / 1000);
        if (period <= 0) {
            return new BN(0);
        }

        const ticks = new BN(period).div(this.vestmentPeriod);
        const percentage = this.vestmentPercentage / 10000;
        const amount = this.initialAmount.muln(percentage).mul(ticks);
        return amount.gt(this.initialAmount) ? this.initialAmount : amount;
    }

    public available(now: Date): BN {
        const available = this.maximum_available(now).sub(this.withdrawn);
        return available.ltn(0) ? new BN(0) : available;
    }
}
