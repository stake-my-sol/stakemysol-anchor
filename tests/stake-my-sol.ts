import * as anchor from "@project-serum/anchor";
import { Program, BN } from "@project-serum/anchor";
import { web3 } from "@project-serum/anchor";
import { StakeMySol } from "../target/types/stake_my_sol";
import { LAMPORTS_PER_SOL } from "@solana/web3.js"
import { assert } from "chai"
import crypto from "crypto"
import sleep from "../utils/sleep";
import {sendSignedTransaction} from "../utils/sendSignedTransaction";

const stakeProgram = web3.StakeProgram

describe("stake-my-sol", () => {
  // Configure the client to use the specified cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  // top up the payer wallet to cover the transaction fees
  // and stake amounts being delegated
  // at least 10 SOLs are required
  const payer = provider.wallet as anchor.Wallet;
  const program = anchor.workspace.StakeMySol as Program<StakeMySol>;

  // fetching 10 vote accounts from network
  console.log("fetching some vote accounts from network...")
  let voteAccounts: web3.PublicKey[] = [];
  let tempKeypair = anchor.web3.Keypair.generate();


  it("fetch some vote accounts and airdrop sol to tempKeypair", async () => {
    const res = await provider.connection.getVoteAccounts(provider.connection.commitment);
    voteAccounts = res.current
                      .slice(0,11)
                      .map(validator => new web3.PublicKey(validator.votePubkey))

    const airdropTx = await provider.connection.requestAirdrop(tempKeypair.publicKey, 1 * LAMPORTS_PER_SOL)
    await provider.connection.confirmTransaction(airdropTx)
    
    assert.equal(10, voteAccounts.length)
  })

  it("create multiple stake accounts and delegate", async () => {
    // maximum number program is capable of is 3
    // Todo: fix the problem in smart contract to allow more accounts to be created
    const numberOfStakeAccounts = 3;
    
    // create random string for seedPrefix
    const seedPrefix = crypto.randomBytes(4).toString('hex') 
    console.log("🚀 ~ file: stake-my-sol.ts ~ line 45 ~ it ~ seedPrefix", seedPrefix)
    // initial index for stake pubkey seeds
    let initialIndex = -1;
    // total amount of lamports to be staked
    // deducting a little bit to cover TX fees
    const totalStakeAmount = 0.8 * LAMPORTS_PER_SOL;
    let remainingAccounts: web3.AccountMeta[] = []
    let bumpsArr: number[] = []

    
    // creating #numberOfStakeAccounts stake accounts and delegating
    console.log(`creating ${numberOfStakeAccounts} stake accounts and delegating...`)

    let calculatedStakePubkeyNum = 0;
    while(calculatedStakePubkeyNum < numberOfStakeAccounts){

      initialIndex += 1;
      
      // creating #numberOfStakeAccounts stake accounts and delegating
      for (let i=0; i <  numberOfStakeAccounts; i++){
        let seedPostFix = (initialIndex + i).toString();
        let [stakePubkey, bump] = web3.PublicKey.findProgramAddressSync([tempKeypair.publicKey.toBuffer(), Buffer.from(`${seedPrefix}-${seedPostFix}`)], stakeProgram.programId) 
        
        if (web3.PublicKey.isOnCurve(stakePubkey)) {
          remainingAccounts = []
          calculatedStakePubkeyNum = 0
          break
        }

        bumpsArr[i] = bump
        remainingAccounts.splice(i, 0, {pubkey: voteAccounts[i], isSigner: false, isWritable: false})
        remainingAccounts.splice(i + numberOfStakeAccounts, 0, {pubkey: stakePubkey, isSigner: false, isWritable: true})
        calculatedStakePubkeyNum += 1;
      }
    }

    // create a new stake account and delegate it to voteAccounts[0...numberOfStakeAccounts]
    console.log(`creating a new stake account and delegate it to voteAccounts[0...numberOfStakeAccounts]...`)
    const tx = await program.methods.createStakeAccountsAndDelegate(
      new BN(totalStakeAmount), 
      initialIndex,
      seedPrefix,
      bumpsArr
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
    .rpc({skipPreflight: true});

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
})
