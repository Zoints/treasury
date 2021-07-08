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
const fee_authority = new Keypair();

const mint_authority = new Keypair();
const deploy_key = new Keypair();
const programId = deploy_key.publicKey;

const randomSurplus = 1234;

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

    const fee_recipient = await token.createAssociatedTokenAccount(
        fee_authority.publicKey
    );
    const settings_id = (
        await PublicKey.findProgramAddress([Buffer.from('settings')], programId)
    )[0];

    await (async () => {
        const keys: AccountMeta[] = [
            { pubkey: funder.publicKey, isSigner: true, isWritable: false },
            { pubkey: token_id.publicKey, isSigner: false, isWritable: false },
            {
                pubkey: fee_authority.publicKey,
                isSigner: true,
                isWritable: false
            },
            { pubkey: fee_recipient, isSigner: false, isWritable: false },
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
        const data = Buffer.alloc(1 + 8 + 8, 0);
        data.writeBigUInt64LE(1_000n, 1); // user fee
        data.writeBigUInt64LE(5_000n, 9); // zoints fee

        const t = new Transaction().add(
            new TransactionInstruction({
                keys,
                programId,
                data
            })
        );

        sig = await sendAndConfirmTransaction(connection, t, [
            funder,
            fee_authority
        ]);
        console.log(`Initialized: ${sig}`);
    })();

    await update_fee(20_000n, 50_000n, settings_id, fee_recipient);

    const user_1 = new Keypair();
    await launch_user_treasury(user_1, settings_id, fee_recipient);

    console.log(`verify account data`);

    const settings = await connection.getAccountInfo(settings_id);
    if (settings === null) {
        console.log(`!!! settings account is missing !!!`);
    } else {
        const s_token = new PublicKey(settings.data.slice(0, 32));
        const s_fee_recipient = new PublicKey(settings.data.slice(32, 64));
        const s_price_authority = new PublicKey(settings.data.slice(64, 96));
        const s_user_fee = settings.data.slice(96, 104).readBigUInt64LE();
        const s_zoints_fee = settings.data.slice(104, 112).readBigUInt64LE();

        if (!s_token.equals(token_id.publicKey)) {
            console.log(
                `settings.token mismatch ${s_token.toBase58()} ${token_id.publicKey.toBase58()}`
            );
        }

        if (!s_fee_recipient.equals(fee_recipient)) {
            console.log(
                `settings.fee_recipient mismatch ${s_fee_recipient.toBase58()} ${fee_recipient.toBase58()}`
            );
        }

        if (!s_price_authority.equals(fee_authority.publicKey)) {
            console.log(
                `settings.price_authority mismatch ${s_price_authority.toBase58()} ${fee_authority.publicKey.toBase58()}`
            );
        }

        if (s_user_fee != 20_000n) {
            console.log(`settings.user_free mismatch ${s_user_fee} 20_000n`);
        }
        if (s_zoints_fee != 50_000n) {
            console.log(`settings.user_free mismatch ${s_zoints_fee} 50_000n`);
        }
    }

    const user_treasury = await PublicKey.findProgramAddress(
        [Buffer.from('user'), user_1.publicKey.toBuffer()],
        programId
    );

    const user = await connection.getAccountInfo(user_treasury[0]);
    if (user === null) {
        console.log(`!!! user treasury account missing !!!`);
    } else {
        const u_authority = new PublicKey(user.data.slice(0, 32));
        if (!u_authority.equals(user_1.publicKey)) {
            console.log(
                `user.authority mismatch ${u_authority.toBase58()} ${user_1.publicKey.toBase58()}`
            );
        }

        const u_assoc_address = await Token.getAssociatedTokenAddress(
            ASSOCIATED_TOKEN_PROGRAM_ID,
            TOKEN_PROGRAM_ID,
            token_id.publicKey,
            user_1.publicKey
        );
        const u_assoc = await token.getAccountInfo(u_assoc_address);

        if (u_assoc.amount.toNumber() != randomSurplus) {
            console.log(
                `user assoc balance wrong ${u_assoc.amount.toNumber()}`
            );
        }
    }
})();

async function update_fee(
    fee_user: bigint,
    fee_zoints: bigint,
    settings_id: PublicKey,
    fee_recipient: PublicKey
) {
    const keys: AccountMeta[] = [
        { pubkey: funder.publicKey, isSigner: true, isWritable: false },
        {
            pubkey: fee_authority.publicKey,
            isSigner: true,
            isWritable: false
        },
        { pubkey: fee_recipient, isSigner: false, isWritable: false },
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
    const data = Buffer.alloc(1 + 8 + 8, 3);
    data.writeBigUInt64LE(fee_user, 1); // user fee
    data.writeBigUInt64LE(fee_zoints, 9); // zoints fee

    const t = new Transaction().add(
        new TransactionInstruction({
            keys,
            programId,
            data
        })
    );

    const sig = await sendAndConfirmTransaction(connection, t, [
        funder,
        fee_authority
    ]);
    console.log(`Updated fees: ${sig}`);
}
async function launch_user_treasury(
    user: Keypair,
    settings_id: PublicKey,
    fee_recipient: PublicKey
) {
    const user_associated = await token.createAssociatedTokenAccount(
        user.publicKey
    );
    await token.mintTo(
        user_associated,
        mint_authority,
        [],
        20_000 + randomSurplus
    );

    const user_treasury = await PublicKey.findProgramAddress(
        [Buffer.from('user'), user.publicKey.toBuffer()],
        programId
    );
    const user_treasury_associated = await PublicKey.findProgramAddress(
        [
            user_treasury[0].toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            token_id.publicKey.toBuffer()
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const keys: AccountMeta[] = [
        {
            pubkey: funder.publicKey,
            isSigner: true,
            isWritable: false
        },
        {
            pubkey: user.publicKey,
            isSigner: true,
            isWritable: false
        },
        {
            pubkey: user_associated,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: user_treasury[0],
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
            pubkey: fee_recipient,
            isSigner: false,
            isWritable: true
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
            pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,
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

    const t = new Transaction()
        .add(
            new TransactionInstruction({
                keys,
                programId,
                data
            })
        )
        .add(
            Token.createAssociatedTokenAccountInstruction(
                ASSOCIATED_TOKEN_PROGRAM_ID,
                TOKEN_PROGRAM_ID,
                token_id.publicKey,
                user_treasury_associated[0],
                user_treasury[0],
                funder.publicKey
            )
        );

    const sig = await sendAndConfirmTransaction(connection, t, [funder, user]);
    console.log(`User Treasury launched: ${sig}`);
}
