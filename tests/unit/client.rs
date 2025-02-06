use super::*;
use mockall::predicate;
use aptos_sdk::{
    types::{
        account_address::AccountAddress,
        transaction::{SignedTransaction, TransactionPayload},
    },
};

#[tokio::test]
async fn test_get_account_balance() {
    let mut mock = Mock::new();
    let expected_balance = 1000u64;
    let address = AccountAddress::from_hex_literal("0x1").unwrap();

    mock.expect_get_account_balance()
        .with(predicate::eq(address))
        .times(1)
        .returning(move |_| Ok(expected_balance));

    let client = Client::new(mock);
    let balance = client.get_account_balance(address).await.unwrap();
    assert_eq!(balance, expected_balance);
}

#[tokio::test]
async fn test_get_sequence_number() {
    let mut mock = Mock::new();
    let expected_sequence = 5u64;
    let address = AccountAddress::from_hex_literal("0x1").unwrap();

    mock.expect_get_sequence_number()
        .with(predicate::eq(address))
        .times(1)
        .returning(move |_| Ok(expected_sequence));

    let client = Client::new(mock);
    let sequence = client.get_sequence_number(address).await.unwrap();
    assert_eq!(sequence, expected_sequence);
}

#[tokio::test]
async fn test_submit_transaction() {
    let mut mock = Mock::new();
    let address = AccountAddress::from_hex_literal("0x1").unwrap();
    let txn = SignedTransaction::default(); // You'll need to create a proper transaction

    mock.expect_submit_transaction()
        .with(predicate::always())
        .times(1)
        .returning(move |_| Ok(Response::new(PendingTransaction::default())));

    let client = Client::new(mock);
    let result = client.submit_transaction(txn).await;
    assert!(result.is_ok());
} 