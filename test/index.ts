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

const connection = new Connection('http://localhost:8899', 'confirmed');
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

    const simple_treasury = new Keypair();
    const simple_authority = new Keypair();
    await launch_treasury(simple_treasury, simple_authority);

    const vested_authority = new Keypair();
    const vested_treasury = new Keypair();
    try {
        const vested_assoc = await Treasury.vestedTreasuryAssociatedAccount(
            vested_treasury.publicKey,
            token_id.publicKey,
            programId
        );
        const ins =
            await TreasuryInstruction.CreateVestedTreasuryAndFundAccount(
                programId,
                funder.publicKey,
                vested_treasury.publicKey,
                vested_authority.publicKey,
                token_id.publicKey,
                100_000n,
                10,
                1000
            );

        ins.push(
            Token.createMintToInstruction(
                TOKEN_PROGRAM_ID,
                token_id.publicKey,
                vested_assoc.fund,
                mint_authority.publicKey,
                [],
                100_000
            )
        );

        const authority_associated = await Token.getAssociatedTokenAddress(
            ASSOCIATED_TOKEN_PROGRAM_ID,
            TOKEN_PROGRAM_ID,
            token_id.publicKey,
            vested_authority.publicKey
        );

        ins.push(
            Token.createAssociatedTokenAccountInstruction(
                ASSOCIATED_TOKEN_PROGRAM_ID,
                TOKEN_PROGRAM_ID,
                token_id.publicKey,
                authority_associated,
                vested_authority.publicKey,
                funder.publicKey
            )
        );

        const tx = new Transaction().add(...ins);
        const sig = await sendAndConfirmTransaction(connection, tx, [
            funder,
            vested_treasury,
            //vested_authority,
            mint_authority
        ]);

        console.log(`vested treasury created: ${sig}`);
    } catch (e) {
        console.log(e);
    }

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
        const simple = await treasury.getSimpleTreasury(
            simple_treasury.publicKey
        );
        if (simple.mode !== SimpleTreasuryMode.Locked) {
            console.log(`simple.mode mismatch`);
        }
        if (!simple.authority.equals(simple_authority.publicKey)) {
            console.log(
                `simple.authority mismatch ${simple.authority.toBase58()} ${simple_authority.publicKey.toBase58()}`
            );
        }

        const fundAssoc = await Treasury.simpleTreasuryAssociatedAccount(
            simple_treasury.publicKey,
            token_id.publicKey,
            programId
        );

        const transfer = await token.transfer(
            funderAssoc.address,
            fundAssoc.fund,
            funder,
            [],
            100_000
        );
        console.log(`Transferred ${transfer}`);

        const treasuryFund = await token.getAccountInfo(fundAssoc.fund);
        console.log(`Fund funds: ${treasuryFund.amount}`);
    } catch (e) {
        console.log(e);
    }

    try {
        console.log(`Sleeping 20 seconds for vested...`);
        await new Promise((resolve) => setTimeout(resolve, 20000));
        const tx = new Transaction().add(
            await TreasuryInstruction.WithdrawVested(
                programId,
                funder.publicKey,
                vested_treasury.publicKey,
                vested_authority.publicKey,
                token_id.publicKey
            )
        );
        let vt = await treasury.getVestedTreasury(vested_treasury.publicKey);
        console.log(
            `VT: Available ${vt
                .available(new Date())
                .toString()}, Max Available ${vt
                .maximum_available(new Date())
                .toString()}, Withdrawn ${vt.withdrawn.toString()}`
        );
        const sig = await sendAndConfirmTransaction(connection, tx, [
            funder,
            vested_authority
        ]);
        console.log(`Withdraw Vested: ${sig}`);

        console.log(`Sleeping 20 seconds for vested...`);
        await new Promise((resolve) => setTimeout(resolve, 20000));
        const tx2 = new Transaction().add(
            await TreasuryInstruction.WithdrawVested(
                programId,
                funder.publicKey,
                vested_treasury.publicKey,
                vested_authority.publicKey,
                token_id.publicKey
            )
        );

        vt = await treasury.getVestedTreasury(vested_treasury.publicKey);
        console.log(
            `VT: Available ${vt
                .available(new Date())
                .toString()}, Max Available ${vt
                .maximum_available(new Date())
                .toString()}, Withdrawn ${vt.withdrawn.toString()}`
        );
        const sig2 = await sendAndConfirmTransaction(connection, tx2, [
            funder,
            vested_authority
        ]);
        console.log(`Withdraw Vested (again): ${sig2}`);

        const vested_assoc = await Token.getAssociatedTokenAddress(
            ASSOCIATED_TOKEN_PROGRAM_ID,
            TOKEN_PROGRAM_ID,
            token_id.publicKey,
            vested_authority.publicKey
        );

        const acc = await token.getAccountInfo(vested_assoc);
        console.log(`Account Money: ${acc.amount}`);

        vt = await treasury.getVestedTreasury(vested_treasury.publicKey);
        console.log(
            `VT: Available ${vt
                .available(new Date())
                .toString()}, Max Available ${vt
                .maximum_available(new Date())
                .toString()}, Withdrawn ${vt.withdrawn.toString()}`
        );
    } catch (e) {
        console.log(e);
    }
})();

async function launch_treasury(treasury: Keypair, authority: Keypair) {
    const tx = new Transaction().add(
        ...(await TreasuryInstruction.CreateSimpleTreasuryAndFundAccount(
            programId,
            funder.publicKey,
            treasury.publicKey,
            authority.publicKey,
            token_id.publicKey
        ))
    );

    const sig = await sendAndConfirmTransaction(connection, tx, [
        funder,
        treasury
    ]);
    console.log(`Treasury launched: ${sig}`);
}
