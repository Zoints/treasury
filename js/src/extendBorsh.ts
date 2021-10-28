import { PublicKey } from '@solana/web3.js';
import { BinaryReader, BinaryWriter } from 'borsh';
import { SimpleTreasuryMode } from '.';

declare module 'borsh' {
    interface BinaryWriter {
        writePublicKey(value: PublicKey): void;
        writeSimpleTreasuryMode(value: number): void;
        writeBigInt(value: bigint): void;
    }
    interface BinaryReader {
        readPublicKey(): PublicKey;
        readSimpleTreasuryMode(): SimpleTreasuryMode;
        readBigInt(): bigint;
    }
}

BinaryWriter.prototype.writePublicKey = function (value: PublicKey) {
    this.writeFixedArray(value.toBuffer());
};

BinaryReader.prototype.readPublicKey = function () {
    return new PublicKey(this.readFixedArray(32));
};

BinaryWriter.prototype.writeSimpleTreasuryMode = function (value: number) {
    this.writeU8(value);
};

BinaryReader.prototype.readSimpleTreasuryMode = function () {
    const mode = this.readU8();
    switch (mode) {
        case SimpleTreasuryMode.Locked:
            return SimpleTreasuryMode.Locked;
        case SimpleTreasuryMode.Unlocked:
            return SimpleTreasuryMode.Unlocked;
        default:
            throw new Error('invalid simple treasury mode');
    }
};

BinaryWriter.prototype.writeBigInt = function (value: bigint) {
    const buf = Buffer.alloc(8);
    buf.writeBigInt64LE(value);
    this.writeFixedArray(buf);
};

BinaryReader.prototype.readBigInt = function () {
    const buf = Buffer.from(this.readFixedArray(8));
    return buf.readBigInt64LE();
};
