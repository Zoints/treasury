import {
    Token,
    TOKEN_PROGRAM_ID,
    MintLayout,
    ASSOCIATED_TOKEN_PROGRAM_ID
} from '@solana/spl-token';
import {
    BPF_LOADER_PROGRAM_ID,
    BpfLoader,
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

const token = new Token(
    connection,
    token_id.publicKey,
    TOKEN_PROGRAM_ID,
    funder
);

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

    // doing it manually since the library doesn't accept pre-defined pubkeys
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

    console.log(`Attempting to initialize`);

    const settings_id = (
        await PublicKey.findProgramAddress([Buffer.from('settings')], programId)
    )[0];

    await (async () => {
        const keys: AccountMeta[] = [
            { pubkey: funder.publicKey, isSigner: true, isWritable: false },
            { pubkey: token_id.publicKey, isSigner: false, isWritable: false },
            {
                pubkey: settings_id,
                isSigner: false,
                isWritable: true
            },
            { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
            {
                pubkey: SystemProgram.programId,
                isSigner: false,
                isWritable: false
            }
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

    const user_1 = new Keypair();
    await launch_treasury(user_1, settings_id);

    console.log(`verify account data`);

    const settings = await connection.getAccountInfo(settings_id);
    if (settings === null) {
        console.log(`!!! settings account is missing !!!`);
    } else {
        const s_token = new PublicKey(settings.data.slice(0, 32));

        if (!s_token.equals(token_id.publicKey)) {
            console.log(
                `settings.token mismatch ${s_token.toBase58()} ${token_id.publicKey.toBase58()}`
            );
        }
    }

    const treasury = await PublicKey.findProgramAddress(
        [Buffer.from('simple'), user_1.publicKey.toBuffer()],
        programId
    );

    const user = await connection.getAccountInfo(treasury[0]);
    if (user === null) {
        console.log(`!!! user treasury account missing !!!`);
    } else {
        const u_authority = new PublicKey(user.data.slice(2, 32));
        if (!u_authority.equals(user_1.publicKey)) {
            console.log(
                `user.authority mismatch ${u_authority.toBase58()} ${user_1.publicKey.toBase58()}`
            );
        }
    }
})();

async function launch_treasury(authority: Keypair, settings_id: PublicKey) {
    const treasury = (
        await PublicKey.findProgramAddress(
            [Buffer.from('simple'), authority.publicKey.toBuffer()],
            programId
        )
    )[0];
    const treasury_fund = (
        await PublicKey.findProgramAddress(
            [Buffer.from('simple fund'), treasury.toBuffer()],
            programId
        )
    )[0];
    ///   0. `[signer]` The account funding the instruction
    ///   1. `[signer]` The authority that controls the treasury
    ///   2. `[writable]` The treasury account for the authority
    ///   3. `[writable]` The treasury account's fund address
    ///   3. `[]` The ZEE token mint
    ///   4. `[]` The global settings program account
    ///   6. `[]` Rent sysvar
    ///   7. `[]` The SPL Token program
    const keys: AccountMeta[] = [
        {
            pubkey: funder.publicKey,
            isSigner: true,
            isWritable: false
        },
        {
            pubkey: authority.publicKey,
            isSigner: true,
            isWritable: false
        },
        {
            pubkey: treasury,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: treasury_fund,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: token_id.publicKey,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: settings_id,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: SYSVAR_RENT_PUBKEY,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false
        },
        {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false
        }
    ];
    const data = Buffer.alloc(1, 1);

    const tx = new Transaction().add(
        new TransactionInstruction({
            keys,
            programId,
            data
        })
    );

    const sig = await sendAndConfirmTransaction(connection, tx, [
        funder,
        authority
    ]);
    console.log(`Treasury launched: ${sig}`);
}
