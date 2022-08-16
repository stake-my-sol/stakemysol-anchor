import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { StakeMySol } from "../target/types/stake_my_sol";

describe("stake-my-sol", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.StakeMySol as Program<StakeMySol>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
