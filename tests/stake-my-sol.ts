import * as anchor from "@project-serum/anchor";
import { Program, BN } from "@project-serum/anchor";
import { web3 } from "@project-serum/anchor";
import { StakeMySol } from "../target/types/stake_my_sol";
import { LAMPORTS_PER_SOL } from "@solana/web3.js"
import { assert } from "chai"
import crypto from "crypto"
import sleep from "../utils/sleep";

const stakeProgram = web3.StakeProgram

describe("stake-my-sol", () => {
  // Configure the client to use the specified cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  // const payer = provider.wallet as anchor.Wallet;
  const program = anchor.workspace.StakeMySol as Program<StakeMySol>;

  // fetching 10 vote accounts from network
  console.log("fetching some vote accounts from network...")
  let voteAccounts: web3.PublicKey[] = [];

  (async () =>{
    const res = await provider.connection.getVoteAccounts(provider.connection.commitment);
    voteAccounts = res.current
                      .slice(0,11)
                      .map(validator => new web3.PublicKey(validator.votePubkey))
  })()

  const tempKeypair = web3.Keypair.generate();




  it("create multiple stake accounts and delegate", async () => {
    // wait for 5 seconds to be able to airdrop again

    // choose a random number of stake accounts to create
    // between 3 and 10
    // const numberOfStakeAccounts = Math.ceil(Math.random() * 8) + 2;

    // maximum number program is capable of is 3
    const numberOfStakeAccounts = 3;
    
    const airdropAmount = 1 * LAMPORTS_PER_SOL;

    console.log(`air dropping ${airdropAmount / LAMPORTS_PER_SOL} SOL to tempKeypair...`)
    const airdrop_tx = await provider.connection.requestAirdrop(tempKeypair.publicKey, airdropAmount);
    await provider.connection.confirmTransaction(airdrop_tx);

    // create random string for seedPrefix
    const seedPrefix = crypto.randomBytes(4).toString('hex') 
    console.log("🚀 ~ file: stake-my-sol.ts ~ line 45 ~ it ~ seedPrefix", seedPrefix)
    // initial index for 
    const initialIndex = 0;
    // total amount of lamports to be staked
    // deducting a little bit to cover TX fees
    const totalStakeAmount = airdropAmount - 0.1 * LAMPORTS_PER_SOL;
    const remainingAccounts: web3.AccountMeta[] = []

    // creating #numberOfStakeAccounts stake accounts and delegating
    console.log(`creating ${numberOfStakeAccounts} stake accounts and delegating...`)
    for (let i=0; i < numberOfStakeAccounts; i++){
      let stakePubkey = await web3.PublicKey.createWithSeed(tempKeypair.publicKey, `${seedPrefix}-${i}`, stakeProgram.programId) 
      console.log("stake pubkey: ", stakePubkey.toBase58())

      remainingAccounts.splice(i, 0, {pubkey: voteAccounts[i], isSigner: false, isWritable: false})
      remainingAccounts.splice(i + numberOfStakeAccounts, 0, {pubkey: stakePubkey, isSigner: false, isWritable: true})
    }

    // create a new stake account and delegate it to voteAccounts[0...numberOfStakeAccounts]
    console.log(`creating a new stake account and delegate it to voteAccounts[0...numberOfStakeAccounts]...`)
    const tx = await program.methods.createStakeAccountsAndDelegate(
      new BN(totalStakeAmount), 
      0,
      seedPrefix,
      )
      .accounts({
        staker: tempKeypair.publicKey,
        stakeProgram: stakeProgram.programId,
        systemProgram: web3.SystemProgram.programId,
        rentSysvar: web3.SYSVAR_RENT_PUBKEY,
        clockSysvar: web3.SYSVAR_CLOCK_PUBKEY,
        stakeHistorySysvar: web3.SYSVAR_STAKE_HISTORY_PUBKEY,
        stakeConfigSysvar: web3.STAKE_CONFIG_ID
    }).remainingAccounts(remainingAccounts)
    .signers([tempKeypair])
    .rpc();

    console.log("createStakeAccountsAndDelegate tx: ", tx)

    // fetching new stake accounts for tempKeypair
    console.log("fetching new stake accounts for tempKeypair")

    const currentStakeAccounts = await provider
      .connection
      .getParsedProgramAccounts(stakeProgram.programId, {
      filters: [
        {
          memcmp: {
            offset: 12,
            bytes: tempKeypair.publicKey.toBase58(),
          },
        },
      ],
    });

    assert.equal(currentStakeAccounts.length, numberOfStakeAccounts)
  })


  it("create a Stake Account And delegate", async () => {

    await sleep(20000);
    const airdropAmount = 1 * LAMPORTS_PER_SOL;
    console.log("air dropping ${airdropAmount / LAMPORTS_PER_SOL} SOL to tempKeypair...")
    const airdrop_tx = await provider.connection.requestAirdrop(tempKeypair.publicKey, airdropAmount);
    await provider.connection.confirmTransaction(airdrop_tx);

    // create random string for seedPrefix
    const seedPrefix = crypto.randomBytes(4).toString('hex') 
    console.log("🚀 ~ file: stake-my-sol.ts ~ line 45 ~ it ~ seedPrefix", seedPrefix)
    // initial index for 
    const initialIndex = 0;
    // total amount of lamports to be staked
    const totalStakeAmount = 0.8 * LAMPORTS_PER_SOL;

    const stakePubkey = await web3.PublicKey.createWithSeed(tempKeypair.publicKey, `${seedPrefix}-${initialIndex}`, stakeProgram.programId) 
    console.log("stake pubkey: ", stakePubkey.toBase58())

    // create a new stake account and delegate it to voteAccounts[0]
    console.log(`creating a new stake account and delegate it to ${voteAccounts[0]}...`)
    const tx = await program.methods.createStakeAccountsAndDelegate(
      new BN(totalStakeAmount), 
      0,
      seedPrefix,
      )
      .accounts({
        staker: tempKeypair.publicKey,
        stakeProgram: stakeProgram.programId,
        systemProgram: web3.SystemProgram.programId,
        rentSysvar: web3.SYSVAR_RENT_PUBKEY,
        clockSysvar: web3.SYSVAR_CLOCK_PUBKEY,
        stakeHistorySysvar: web3.SYSVAR_STAKE_HISTORY_PUBKEY,
        stakeConfigSysvar: web3.STAKE_CONFIG_ID
    }).remainingAccounts([{
      pubkey: voteAccounts[0], isSigner: false, isWritable: false
    },{
      pubkey: stakePubkey, isSigner: false, isWritable: true
    }])
    .signers([tempKeypair])
    .rpc();

    console.log("createStakeAccountsAndDelegate tx: ", tx)

    // fetching new stake accounts for tempKeypair
    console.log("fetching new stake accounts for tempKeypair")

    const currentStakeAccounts = await provider
      .connection
      .getParsedProgramAccounts(stakeProgram.programId, {
      filters: [
        {
          memcmp: {
            offset: 12,
            bytes: tempKeypair.publicKey.toBase58(),
          },
        },
      ],
    });

    assert.equal(currentStakeAccounts.length, 1)
  })
});
