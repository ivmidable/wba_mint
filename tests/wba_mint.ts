import * as anchor from "@coral-xyz/anchor";
import { Program, } from "@coral-xyz/anchor";
import { WbaMint } from "../target/types/wba_mint";
import { ASSOCIATED_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("wba_mint", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  let provider = anchor.AnchorProvider.env();

  const program = anchor.workspace.WbaMint as Program<WbaMint>;
  const authority = anchor.web3.Keypair.generate();

  console.log("Authority pubkey: ", authority.publicKey.toBase58());
  console.log("Authority secret: ", authority.secretKey);

  let [wba_mint, wba_miny_bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("wba_mint")],
    program.programId);

  let [wba_auth, wba_auth_bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("wba_auth")],
    program.programId);

  let [wba_vault, wba_vault_bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("wba_vault"),
    wba_mint.toBuffer(),
    ],
    program.programId);

  let [state, state_bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("state")],
    program.programId);


  let auth_ata = anchor.utils.token.associatedAddress({ mint: wba_mint, owner: authority.publicKey });

  before(async () => {
    try {
      let tx_sig = await provider.connection.requestAirdrop(authority.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL);
      let latestBlockHash = await provider.connection.getLatestBlockhash();
      await provider.connection.confirmTransaction({
        blockhash: latestBlockHash.blockhash,
        lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
        signature: tx_sig,
      });
    }
    catch (e) {
      console.log(e);
    }
  });

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize(
      new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL),
    ).accounts({
      authority: authority.publicKey,
      wbaMint: wba_mint,
      wbaAuth: wba_auth,
      state: state,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([authority]).rpc();


    console.log("Your transaction signature", tx);
  });

  it("Mints a WBA", async () => {
      const tx = await program.methods.mint(1).accounts({
        cadet: authority.publicKey,
        cadetAta: auth_ata,
        state: state,
        wbaMint: wba_mint,
        wbaAuth: wba_auth,
        wbaVault: wba_vault,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).signers([authority]).rpc();
      console.log("Your transaction signature", tx);
  });

  it("Claims SOL", async () => { 
    const tx = await program.methods.claim().accounts({
      authority: authority.publicKey,
      payTo: authority.publicKey,
      wbaMint: wba_mint,
      wbaVault: wba_vault,
      systemProgram: anchor.web3.SystemProgram.programId,

    }).signers([authority]).rpc();
  
    console.log(tx);
  });
});
