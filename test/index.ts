import {
    Token,
    TOKEN_PROGRAM_ID,
    MintLayout,
    ASSOCIATED_TOKEN_PROGRAM_ID
} from '@solana/spl-token';
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
const fee_authority = new Keypair();

const mint_authority = new Keypair();
const deploy_key = new Keypair();
const programId = deploy_key.publicKey;

const token = new Token(
    connection,
    token_id.publicKey,
    TOKEN_PROGRAM_ID,
    new Account(funder.secretKey)
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
        /*
        let funder_info = next_account_info(iter)?;
        let token_info = next_account_info(iter)?;
        let authority_info = next_account_info(iter)?;
        let fee_recipient_info = next_account_info(iter)?;
        let settings_info = next_account_info(iter)?;
        let rent_info = next_account_info(iter)?;
        */
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

    const zoints_1 = new Keypair();
    const zoints_1_keyword = Buffer.from('test-community');
    await launch_zoints_treasury(
        zoints_1,
        zoints_1_keyword,
        settings_id,
        fee_recipient
    );
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

async function launch_zoints_treasury(
    owner: Keypair,
    name: Buffer,
    settings_id: PublicKey,
    fee_recipient: PublicKey
) {
    const zoints_associated = await token.createAssociatedTokenAccount(
        owner.publicKey
    );
    await token.mintTo(
        zoints_associated,
        new Account(mint_authority.secretKey),
        [],
        50_000
    );

    const zoints_treasury = await PublicKey.findProgramAddress(
        [Buffer.from('zoints'), name],
        programId
    );

    const zoints_treasury_associated = await PublicKey.findProgramAddress(
        [
            zoints_treasury[0].toBuffer(),
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
            pubkey: owner.publicKey,
            isSigner: true,
            isWritable: false
        },
        {
            pubkey: zoints_associated,
            isSigner: false,
            isWritable: true
        },
        {
            pubkey: zoints_treasury[0],
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
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false
        }
    ];
    const prefix = Buffer.alloc(1 + 2, 0);
    prefix[0] = 2;
    prefix.writeUInt16LE(name.length, 1);
    const data = Buffer.concat([prefix, name]);

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
                zoints_treasury_associated[0],
                zoints_treasury[0],
                funder.publicKey
            )
        );

    const sig = await sendAndConfirmTransaction(connection, t, [funder, owner]);
    console.log(`Zoints Treasury launched: ${sig}`);
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
        new Account(mint_authority.secretKey),
        [],
        20_000
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
