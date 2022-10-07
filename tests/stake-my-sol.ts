import * as anchor from "@project-serum/anchor";
import { Program, BN } from "@project-serum/anchor";
import { web3 } from "@project-serum/anchor";
import { StakeMySol } from "../target/types/stake_my_sol";
import {LAMPORTS_PER_SOL} from "@solana/web3.js"
import { assert } from "chai"



const stakeProgram = web3.StakeProgram

describe("stake-my-sol", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as anchor.Wallet;
  const program = anchor.workspace.StakeMySol as Program<StakeMySol>;

  it("create a Stake Account And delegate", async () => {
    const seedPrefix = "slgd36t66gh";
    const initialIndex = 0;
    const totalStakeAmount = 2 * LAMPORTS_PER_SOL;
    // testnet
    // const voteAccount = new web3.PublicKey("9CdZxSmB6RH1Rcd4q2Wb56eFWDCu25TVs3484Y45W6rL");

    // devnet
    const voteAccount = new web3.PublicKey("4QUZQ4c7bZuJ4o4L8tYAEGnePFV27SUFEVmC7BYfsXRp");

    const stakePubkey = await web3.PublicKey.createWithSeed(payer.publicKey, `${seedPrefix}-${initialIndex}`, stakeProgram.programId) 
    console.log("stake pubkey: ", stakePubkey.toBase58())

    const tx = await program.methods.createStakeAccountsAndDelegate(
      new BN(totalStakeAmount), 
      0, 
      seedPrefix,
      )
      .accounts({
        staker: payer.publicKey,
        stakeProgram: stakeProgram.programId,
        systemProgram: web3.SystemProgram.programId,
        rentSysvar: web3.SYSVAR_RENT_PUBKEY,
        clockSysvar: web3.SYSVAR_CLOCK_PUBKEY,
        stakeHistorySysvar: web3.SYSVAR_STAKE_HISTORY_PUBKEY,
        stakeConfigSysvar: web3.STAKE_CONFIG_ID
    }).remainingAccounts([{
      pubkey: voteAccount, isSigner: false, isWritable: false
    },{
      pubkey: stakePubkey, isSigner: false, isWritable: true
    }])
    .signers([payer.payer])
    .rpc();

    const currentStakeAccounts = await provider
      .connection
      .getParsedProgramAccounts(stakeProgram.programId, {
      filters: [
        {
          memcmp: {
            offset: 12,
            bytes: payer.publicKey.toBase58(),
          },
        },
      ],
    });

    assert.equal(currentStakeAccounts.length, 1)

    console.log("Your transaction signature", tx);
  })
});
