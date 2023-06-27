use concordium_std::*;
use std::collections::HashSet;

// Verifier function for the mint_nft contract
#[concordium_test]
fn verify_mint_nft() {
    let owner = AccountAddress::from_hex("0x01234567").unwrap();
    let caller = AccountAddress::from_hex("0x01234567").unwrap();
    let identity = "NFT Identity".to_string();
    let conditions = "NFT Conditions".to_string();

    let (storage, event) = mint_nft(&TestContext::new(caller), owner.clone(), identity.clone(), conditions.clone());

    assert_eq!(event.data(), "NFT minted successfully");

    let stored_nft: Option<NFT> = storage.get(&owner);
    assert!(stored_nft.is_some());

    let nft = stored_nft.unwrap();
    assert_eq!(nft.owner, owner);
    assert_eq!(nft.identity, identity);
    assert_eq!(nft.conditions, conditions);
}

// Verifier function for the buy_nft contract
#[concordium_test]
fn verify_buy_nft() {
    let owner = AccountAddress::from_hex("0x01234567").unwrap();
    let caller = AccountAddress::from_hex("0x01234567").unwrap();

    let storage = Storage::new().set(&owner, &NFT::new(owner.clone(), "NFT Identity".to_string(), "NFT Conditions".to_string()));

    let (new_storage, event) = buy_nft(&TestContext::new(caller), owner.clone());

    assert_eq!(event.data(), "NFT purchased successfully");

    let stored_nft: Option<NFT> = new_storage.get(&caller);
    assert!(stored_nft.is_some());

    let nft = stored_nft.unwrap();
    assert_eq!(nft.owner, caller);
    assert_eq!(nft.identity, "NFT Identity");
    assert_eq!(nft.conditions, "NFT Conditions");
}

// Verifier function for the generate_reveal_statement contract
#[concordium_test]
fn verify_generate_reveal_statement() {
    let attributes = Attributes {
        first_name: Some("Raj".to_string()),
        last_name: Some("oberoi".to_string()),
        sex: Some("Male".to_string()),
        date_of_birth: Some("1990-01-01".to_string()),
        country_of_residence: Some("india".to_string()),
        country_of_nationality: Some("india".to_string()),
        id_document_type: Some("Passport".to_string()),
        id_document_number: Some("123456789".to_string()),
        id_document_issuer: Some("INDIAN Government".to_string()),
        id_valid_from: Some("2020-01-01".to_string()),
        id_valid_to: Some("2030-01-01".to_string()),
        national_id_number: Some("987654321".to_string()),
        tax_id_number: Some("555-123-4567".to_string()),
    };

    let (storage, event) = generate_reveal_statement(&TestContext::new(AccountAddress::from_hex("0x01234567").unwrap()), attributes.clone());

    let expected_statement = "\
        - First Name: Raj\n\
        - Last Name: Oberoi\n\
        - Sex: Male\n\
        - Date of Birth: 1990-01-01\n\
        - Country of Residence: india\n\
        - Country of Nationality: indian\n\
        - ID Document Type: Passport\n\
        - ID Document Number: 123456789\n\
        - ID Document Issuer: INDIAN Government\n\
        - ID Valid From: 2020-01-01\n\
        - ID Valid To: 2030-01-01\n\
        - National ID number: 987654321\n\
        - Tax ID number: 555-123-4567\n\
    ";

    assert_eq!(event.data("reveal_statement").unwrap(), expected_statement);
    assert_eq!(storage.into_mutation().len(), 0);
}

// Verifier function for the generate_range_proof contract
#[concordium_test]
fn verify_generate_range_proof() {
    let range_start = 10;
    let range_end = 20;

    let (storage, event) = generate_range_proof(&TestContext::new(AccountAddress::from_hex("0x01234567").unwrap()), range_start, range_end);

    let expected_range_proof = "Range Proof: 10 - 20";
    assert_eq!(event.data("range_proof").unwrap(), expected_range_proof);
    assert_eq!(storage.into_mutation().len(), 0);
}

// Verifier function for the generate_membership_proof contract
#[concordium_test]
fn verify_generate_membership_proof() {
    let attributes = Attributes {
        first_name: None,
        last_name: None,
        sex: None,
        date_of_birth: None,
        country_of_residence: Some("india".to_string()),
        country_of_nationality: Some("india".to_string()),
        id_document_type: Some("Passport".to_string()),
        id_document_number: None,
        id_document_issuer: Some("INDIAN Government".to_string()),
        id_valid_from: None,
        id_valid_to: None,
        national_id_number: None,
        tax_id_number: None,
    };

    let (storage, event) = generate_membership_proof(&TestContext::new(AccountAddress::from_hex("0x01234567").unwrap()), attributes.clone());

    assert_eq!(event.data(), "Caller has the required membership attribute");
    assert_eq!(storage.into_mutation().len(), 0);
}


