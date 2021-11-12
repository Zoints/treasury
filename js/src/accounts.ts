import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import * as borsh from 'borsh';
import './extendBorsh';

export enum SimpleTreasuryMode {
    Locked,
    Unlocked
}

export class SimpleTreasury {
    public mint: PublicKey;
    public mode: SimpleTreasuryMode;
    public authority: PublicKey;

    constructor(params: {
        mint: PublicKey;
        mode: SimpleTreasuryMode;
        authority: PublicKey;
    }) {
        this.mint = params.mint;
        this.mode = params.mode;
        this.authority = params.authority;
    }
}

export class VestedTreasury {
    public mint: PublicKey;
    public authority: PublicKey;
    public initialAmount: BN;
    public start: Date;
    public vestmentPeriod: BN;
    public vestmentPercentage: number;
    public withdrawn: BN;

    constructor(params: {
        mint: PublicKey;
        authority: PublicKey;
        initialAmount: BN;
        start: BN;
        vestmentPeriod: BN;
        vestmentPercentage: number;
        withdrawn: BN;
    }) {
        this.mint = params.mint;
        this.authority = params.authority;
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

export const ACCOUNT_SCHEMA: borsh.Schema = new Map<any, any>([
    [
        SimpleTreasury,
        {
            kind: 'struct',
            fields: [
                ['mint', 'PublicKey'],
                ['mode', 'SimpleTreasuryMode'],
                ['authority', 'PublicKey']
            ]
        }
    ],
    [
        VestedTreasury,
        {
            kind: 'struct',
            fields: [
                ['mint', 'PublicKey'],
                ['authority', 'PublicKey'],
                ['initialAmount', 'u64'],
                ['start', 'u64'],
                ['vestmentPeriod', 'u64'],
                ['vestmentPercentage', 'u16'],
                ['withdrawn', 'u64']
            ]
        }
    ]
]);
