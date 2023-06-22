use concordium_std::*;

#[concordium_contract(storage)]
pub fn mint_nft(
    ctx: &impl HasActions,
    owner: AccountAddress,
    identity: String,
    conditions: String,
) -> (Storage, Event) {
    let caller = ctx.get_caller();

    // Verify the ownership and identity
    if caller != owner {
        return (
            Storage::new(),
            Event::new().error("Caller is not the owner of the NFT"),
        );
    }

    // Store the NFT details in the contract storage
    let nft = NFT::new(owner, identity, conditions);
    let storage = Storage::new().set(&caller, &nft);

    (
        storage,
        Event::new().info("NFT minted successfully"),
    )
}

#[concordium_contract(storage)]
pub fn buy_nft(
    ctx: &impl HasActions,
    owner: AccountAddress,
) -> (Storage, Event) {
    let caller = ctx.get_caller();

    // Retrieve the NFT details from the contract storage
    let storage = Storage::new();
    let nft: Option<NFT> = storage.get(&owner);

    match nft {
        Some(nft) => {
            // Perform the conditions check
            let conditions = nft.get_conditions();
            let can_buy = check_conditions(&caller, &conditions);

            if can_buy {
                // Transfer ownership of the NFT to the caller
                let new_nft = NFT::new(caller.clone(), nft.get_identity(), conditions);
                let new_storage = storage.set(&caller, &new_nft).remove(&owner);

                (
                    new_storage,
                    Event::new().info("NFT purchased successfully"),
                )
            } else {
                (
                    storage,
                    Event::new().error("Conditions not met to purchase the NFT"),
                )
            }
        }
        None => (
            storage,
            Event::new().error("NFT with the specified owner not found"),
        ),
    }
}

fn check_conditions(caller: &AccountAddress, conditions: &str) -> bool {
    // Perform your custom conditions check here
    // Example: Allow the caller to buy if they are whitelisted
    let whitelist = ["0x12345678", "0xabcdef01"];
    whitelist.contains(&caller.to_hex())
}

struct NFT {
    owner: AccountAddress,
    identity: String,
    conditions: String,
}

impl NFT {
    fn new(owner: AccountAddress, identity: String, conditions: String) -> Self {
        Self {
            owner,
            identity,
            conditions,
        }
    }

    fn get_identity(&self) -> &str {
        &self.identity
    }

    fn get_conditions(&self) -> &str {
        &self.conditions
    }
}
