import * as web3 from "@solana/web3.js";
import * as token2022WrapperSdk from "../src";
import { createKeypair } from "./utils/helpers";
import { initializeMints } from "./utils/tokenUtils";
import { initializeMints2022 } from "./utils/token2022utils";

/**
 * 1. create and mint test SPL token -- done
 * 2. create associated token account -- done
 * 3. create T22 token account -- done
 * 4. create and mint test T22 token -- done
 * 7. basic test initialize
 * 8. basic test depositMint
 * 9. basic test withdrawBurn
 */
const main = async() => {
    // Create a token and mint it to users

    const connection = new web3.Connection(`https://api.devnet.solana.com`, "confirmed");
    
    let userA = await createKeypair(connection);

    let splTokenInfo = await initializeMints(
        connection,
        1,
        [9],
        [
            userA.publicKey,
        ]
    );

    let token2022TokenInfo = await initializeMints2022(
        connection,
        1,
        [9],
        [0],
        [
            userA.publicKey,
        ]
    );
}

main();