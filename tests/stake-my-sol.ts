import * as anchor from "@project-serum/anchor";
import { Program, BN } from "@project-serum/anchor";
import { web3 } from "@project-serum/anchor";
import { StakeMySol } from "../target/types/stake_my_sol";
import {LAMPORTS_PER_SOL} from "@solana/web3.js"
import { assert } from "chai"

const stake_program = web3.StakeProgram

describe("stake-my-sol", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as anchor.Wallet;
  const program = anchor.workspace.StakeMySol as Program<StakeMySol>;

  it("create Stake Account And Split", async () => {
    const seedPrefix = "slgjogjos";
    const initialIndex = 0;
    const totalStakeAmount = 2 * LAMPORTS_PER_SOL;
    const voteAccount = new web3.PublicKey("9CdZxSmB6RH1Rcd4q2Wb56eFWDCu25TVs3484Y45W6rL");

    const stakePubkey = await web3.PublicKey.createWithSeed(payer.publicKey, `${seedPrefix}-${initialIndex}`, stake_program.programId) 
    console.log(stakePubkey.toBase58())

    const tx = await program.methods.createStakeAccountsAndDelegate(
      new BN(totalStakeAmount), 
      0, 
      seedPrefix,
      )
      .accounts({
        staker: payer.publicKey,
        stakeProgram: stake_program.programId,
    }).remainingAccounts([{
      pubkey: voteAccount, isSigner: false, isWritable: false
    },{
      pubkey: stakePubkey, isSigner: false, isWritable: false
    }])
    .signers([payer.payer])
    .rpc();

    const currentStakeAccounts = await provider
      .connection
      .getParsedProgramAccounts(stake_program.programId, {
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
