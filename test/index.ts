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
    Transaction,
    PublicKey,
    sendAndConfirmTransaction,
    SystemProgram
} from '@solana/web3.js';
import { Connection } from '@solana/web3.js';
import * as fs from 'fs';
import {
    SimpleTreasuryMode,
    Treasury,
    TreasuryInstruction
} from '@zoints/treasury';

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

const treasury = new Treasury(connection, programId);

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

    const funderAssoc = await token.getOrCreateAssociatedAccountInfo(
        funder.publicKey
    );
    await token.mintTo(funderAssoc.address, mint_authority, [], 100_000);

    console.log(`Attempting to initialize`);

    const settings_id = (
        await PublicKey.findProgramAddress([Buffer.from('settings')], programId)
    )[0];

    const initTx = new Transaction().add(
        await TreasuryInstruction.Initialize(
            programId,
            funder.publicKey,
            token_id.publicKey
        )
    );

    sig = await sendAndConfirmTransaction(connection, initTx, [funder]);
    console.log(`Initialized: ${sig}`);

    const treasury1 = new Keypair();
    await launch_treasury(treasury1);

    console.log(`verify account data`);

    try {
        const settings = await treasury.getSettings();
        if (!settings.token.equals(token_id.publicKey)) {
            console.log(
                `settings.token mismatch ${settings.token.toBase58()} ${token_id.publicKey.toBase58()}`
            );
        }
    } catch (e) {
        console.log(e);
    }

    try {
        const simple = await treasury.getSimpleTreasuryByAuthority(
            treasury1.publicKey
        );
        if (simple.mode !== SimpleTreasuryMode.Locked) {
            console.log(`simple.mode mismatch`);
        }
        if (!simple.authority.equals(treasury1.publicKey)) {
            console.log(
                `simple.authority mismatch ${simple.authority.toBase58()} ${treasury1.publicKey.toBase58()}`
            );
        }

        const treasuryId = await Treasury.simpleTreasuryId(
            treasury1.publicKey,
            programId
        );
        const fund = await Treasury.simpleTreasuryFundId(
            treasuryId,
            token_id.publicKey
        );

        const transfer = await token.transfer(
            funderAssoc.address,
            fund,
            funder,
            [],
            100_000
        );
        console.log(`Transferred ${transfer}`);

        const treasuryFund = await token.getAccountInfo(fund);
        console.log(`Fund funds: ${treasuryFund.amount}`);
    } catch (e) {
        console.log(e);
    }
})();

async function launch_treasury(authority: Keypair) {
    const tx = new Transaction().add(
        ...(await TreasuryInstruction.CreateSimpleTreasuryAndFundAccount(
            programId,
            funder.publicKey,
            authority.publicKey,
            token_id.publicKey
        ))
    );

    const sig = await sendAndConfirmTransaction(connection, tx, [
        funder,
        authority
    ]);
    console.log(`Treasury launched: ${sig}`);
}
