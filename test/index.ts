import { Token, TOKEN_PROGRAM_ID, MintLayout } from '@solana/spl-token';
import {
    BPF_LOADER_PROGRAM_ID,
    BpfLoader,
    Account,
    Keypair,
    LAMPORTS_PER_SOL,
    Signer,
    Transaction,
    TransactionInstruction,
    AccountMeta,
    PublicKey,
    SYSVAR_RENT_PUBKEY,
    sendAndConfirmTransaction,
    SystemProgram
} from '@solana/web3.js';
import { Connection } from '@solana/web3.js';
import * as fs from 'fs';

const connection = new Connection('http://localhost:8899');
const funder = new Keypair();
const token_id = new Keypair();
const mint_authority = new Keypair();
const deploy_key = new Keypair();
const programId = deploy_key.publicKey;

(async () => {
    console.log(`Funding ${funder.publicKey.toBase58()} with 20 SOL`);
    let sig = await connection.requestAirdrop(
        funder.publicKey,
        20 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(sig);

    console.log(`Deploy BPF to ${deploy_key.publicKey.toBase58()}`);
    const programdata = fs.readFileSync('../program/target/deploy/treasury.so');
    if (
        !(await BpfLoader.load(
            connection,
            funder,
            deploy_key,
            programdata,
            BPF_LOADER_PROGRAM_ID
        ))
    ) {
        console.log('Loading bpf failed');
        process.exit(1);
    }

    console.log(`Creating new SPL Token ${token_id.publicKey.toBase58()}`);

    const token = new Token(
        connection,
        token_id.publicKey,
        programId,
        new Account(funder.secretKey)
    );

    // Allocate memory for the account
    const balanceNeeded = await Token.getMinBalanceRentForExemptMint(
        connection
    );

    const transaction = new Transaction();
    transaction.add(
        SystemProgram.createAccount({
            fromPubkey: funder.publicKey,
            newAccountPubkey: token_id.publicKey,
            lamports: balanceNeeded,
            space: MintLayout.span,
            programId: TOKEN_PROGRAM_ID
        })
    );

    transaction.add(
        Token.createInitMintInstruction(
            TOKEN_PROGRAM_ID,
            token_id.publicKey,
            0,
            mint_authority.publicKey,
            null
        )
    );

    await sendAndConfirmTransaction(connection, transaction, [
        funder,
        token_id
    ]);

    //    while (await connection.getAccountInfo()) {}

    console.log(`Attempting to initialize`);
    const settings_id = await PublicKey.findProgramAddress(
        [Buffer.from('settings')],
        programId
    );

    const keys: AccountMeta[] = [
        { pubkey: funder.publicKey, isSigner: true, isWritable: false },
        { pubkey: token_id.publicKey, isSigner: false, isWritable: true },
        { pubkey: settings_id[0], isSigner: false, isWritable: true },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
    ];
    const data = Buffer.alloc(1, 0);

    const t = new Transaction().add(
        new TransactionInstruction({
            keys,
            programId,
            data
        })
    );

    sig = await sendAndConfirmTransaction(connection, t, [funder]);
    console.log(`Initialized: ${sig}`);
})();

export function sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}
