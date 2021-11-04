import {
    Cluster,
    clusterApiUrl,
    Connection,
    Keypair,
    PublicKey,
    sendAndConfirmTransaction,
    Transaction
} from '@solana/web3.js';
import { TreasuryInstruction } from '@zoints/treasury';

(async () => {
    let url = process.env.URL || 'http://localhost:8899';
    if (process.env.URL !== undefined)
        url = clusterApiUrl(process.env.URL as Cluster);

    const programId = new PublicKey(process.env.PROGRAM_ID || 'error');
    const mint = new PublicKey(process.env.MINT || 'error');
    const funder = Keypair.fromSecretKey(
        Buffer.from(process.env.FUNDER || 'error', 'hex')
    );
    const connection = new Connection(url, 'confirmed');

    const tx = new Transaction().add(
        await TreasuryInstruction.Initialize(programId, funder.publicKey, mint)
    );
    console.log(`Sending transaction to cluster ${url} ...`);
    console.log(await sendAndConfirmTransaction(connection, tx, [funder]));
})().then(() => process.exit(0));
