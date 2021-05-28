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

const user_1 = new Keypair();
const zoints_1 = new Keypair();
const zoints_1_keyword = Buffer.from('test-community');

(async () => {
    console.log(`Funding ${funder.publicKey.toBase58()} with 20 SOL`);
    let sig = await connection.requestAirdrop(
        funder.publicKey,
        20 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(sig);

    // it's not necessary for this account to exist to make an associated address for it
    /*await connection.requestAirdrop(
        fee_authority.publicKey,
        1 * LAMPORTS_PER_SOL
    );*/

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

    const token = new Token(
        connection,
        token_id.publicKey,
        TOKEN_PROGRAM_ID,
        new Account(funder.secretKey)
    );

    const fee_recipient = await token.createAssociatedTokenAccount(
        fee_authority.publicKey
    );

    console.log(`Attempting to initialize`);
    const settings_id = await PublicKey.findProgramAddress(
        [Buffer.from('settings')],
        programId
    );

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
            { pubkey: token_id.publicKey, isSigner: false, isWritable: true },
            {
                pubkey: fee_authority.publicKey,
                isSigner: true,
                isWritable: false
            },
            { pubkey: fee_recipient, isSigner: false, isWritable: false },
            { pubkey: settings_id[0], isSigner: false, isWritable: true },
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

    const user_1_associated = await token.createAssociatedTokenAccount(
        user_1.publicKey
    );
    await token.mintTo(
        user_1_associated,
        new Account(mint_authority.secretKey),
        [],
        1_000
    );

    const user_1_treasury = await PublicKey.findProgramAddress(
        [Buffer.from('user'), user_1.publicKey.toBuffer()],
        programId
    );
    const user_1_treasury_associated = await PublicKey.findProgramAddress(
        [
            user_1_treasury[0].toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            token_id.publicKey.toBuffer()
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // launch user treasury
    await (async () => {
        /*
            let iter = &mut accounts.iter();
        let funder_info = next_account_info(iter)?;
        let creator_info = next_account_info(iter)?;
        let creator_associated_info = next_account_info(iter)?;
        let treasury_info = next_account_info(iter)?;
        let mint_info = next_account_info(iter)?;
        let settings_info = next_account_info(iter)?;
        let fee_recipient_info = next_account_info(iter)?;
        let rent_info = next_account_info(iter)?;
        let rent = Rent::from_account_info(rent_info)?;
        */

        const keys: AccountMeta[] = [
            {
                pubkey: funder.publicKey,
                isSigner: true,
                isWritable: true
            },
            {
                pubkey: user_1.publicKey,
                isSigner: true,
                isWritable: true
            },
            {
                pubkey: user_1_associated,
                isSigner: false,
                isWritable: true
            },
            {
                pubkey: user_1_treasury[0],
                isSigner: false,
                isWritable: true
            },
            {
                pubkey: token_id.publicKey,
                isSigner: false,
                isWritable: false
            },
            {
                pubkey: settings_id[0],
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
                    user_1_treasury_associated[0],
                    user_1_treasury[0],
                    funder.publicKey
                )
            );

        sig = await sendAndConfirmTransaction(connection, t, [funder, user_1]);
        console.log(`User Treasury launched: ${sig}`);
    })();

    const zoints_1_associated = await token.createAssociatedTokenAccount(
        zoints_1.publicKey
    );
    await token.mintTo(
        zoints_1_associated,
        new Account(mint_authority.secretKey),
        [],
        5_000
    );

    const zoints_1_treasury = await PublicKey.findProgramAddress(
        [Buffer.from('zoints'), zoints_1_keyword],
        programId
    );

    const zoints_1_treasury_associated = await PublicKey.findProgramAddress(
        [
            zoints_1_treasury[0].toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            token_id.publicKey.toBuffer()
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
    );

    await (async () => {
        const keys: AccountMeta[] = [
            {
                pubkey: funder.publicKey,
                isSigner: true,
                isWritable: true
            },
            {
                pubkey: zoints_1.publicKey,
                isSigner: true,
                isWritable: true
            },
            {
                pubkey: zoints_1_associated,
                isSigner: false,
                isWritable: true
            },
            {
                pubkey: zoints_1_treasury[0],
                isSigner: false,
                isWritable: true
            },
            {
                pubkey: token_id.publicKey,
                isSigner: false,
                isWritable: false
            },
            {
                pubkey: settings_id[0],
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
        prefix.writeUInt16LE(zoints_1_keyword.length, 1);
        const data = Buffer.concat([prefix, zoints_1_keyword]);

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
                    zoints_1_treasury_associated[0],
                    zoints_1_treasury[0],
                    funder.publicKey
                )
            );

        sig = await sendAndConfirmTransaction(connection, t, [
            funder,
            zoints_1
        ]);
        console.log(`Zoints Treasury launched: ${sig}`);
    })();
})();

export function sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}
