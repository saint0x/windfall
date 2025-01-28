use crate::db::schema::*;
use sqlx::{Pool, Postgres};
use anyhow::{Result, anyhow, Context};
use chrono::{DateTime, Utc};
use aptos_sdk::types::account_address::AccountAddress;

// Fund operations
pub async fn create_fund(
    pool: &SqlitePool,
    name: &str,
    description: &str,
    executor: &AccountAddress,
    metadata: &[(String, String)],
) -> Result<Fund> {
    let mut tx = pool.begin().await?;

    // Create fund
    let fund = sqlx::query_as!(
        Fund,
        r#"
        INSERT INTO funds (name, description, executor)
        VALUES (?, ?, ?)
        RETURNING *
        "#,
        name,
        description,
        executor.to_string(),
    )
    .fetch_one(&mut *tx)
    .await?;

    // Add metadata
    for (key, value) in metadata {
        sqlx::query!(
            r#"
            INSERT INTO fund_metadata (fund_id, key, value)
            VALUES (?, ?, ?)
            "#,
            fund.id,
            key,
            value,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(fund)
}

pub async fn get_fund_with_metadata(pool: &SqlitePool, fund_id: i64) -> Result<(Fund, Vec<FundMetadata>)> {
    let fund = sqlx::query_as!(
        Fund,
        r#"
        SELECT * FROM funds WHERE id = ?
        "#,
        fund_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow!("Fund not found"))?;

    let metadata = sqlx::query_as!(
        FundMetadata,
        r#"
        SELECT * FROM fund_metadata 
        WHERE fund_id = ?
        ORDER BY key
        "#,
        fund_id
    )
    .fetch_all(pool)
    .await?;

    Ok((fund, metadata))
}

pub async fn update_fund_metadata(
    pool: &SqlitePool,
    fund_id: i64,
    key: &str,
    value: &str,
) -> Result<FundMetadata> {
    let metadata = sqlx::query_as!(
        FundMetadata,
        r#"
        INSERT INTO fund_metadata (fund_id, key, value)
        VALUES (?, ?, ?)
        ON CONFLICT(fund_id, key) DO UPDATE SET
            value = excluded.value,
            updated_at = CURRENT_TIMESTAMP
        RETURNING *
        "#,
        fund_id,
        key,
        value,
    )
    .fetch_one(pool)
    .await?;

    Ok(metadata)
}

// Fund member operations
pub async fn add_fund_member(
    pool: &SqlitePool,
    fund_id: i64,
    member_address: &AccountAddress,
) -> Result<FundMember> {
    let member = sqlx::query_as!(
        FundMember,
        r#"
        INSERT INTO fund_members (fund_id, member_address)
        VALUES (?, ?)
        RETURNING *
        "#,
        fund_id,
        member_address.to_string(),
    )
    .fetch_one(pool)
    .await?;

    Ok(member)
}

pub async fn get_fund_members(pool: &SqlitePool, fund_id: i64) -> Result<Vec<FundMember>> {
    let members = sqlx::query_as!(
        FundMember,
        r#"
        SELECT * FROM fund_members WHERE fund_id = ?
        "#,
        fund_id
    )
    .fetch_all(pool)
    .await?;

    Ok(members)
}

// Message operations
pub async fn add_message(
    pool: &SqlitePool,
    fund_id: i64,
    sender: &AccountAddress,
    content: &str,
) -> Result<Message> {
    let message = sqlx::query_as!(
        Message,
        r#"
        INSERT INTO messages (fund_id, sender, content)
        VALUES (?, ?, ?)
        RETURNING *
        "#,
        fund_id,
        sender.to_string(),
        content,
    )
    .fetch_one(pool)
    .await?;

    Ok(message)
}

pub async fn get_fund_messages(
    pool: &SqlitePool,
    fund_id: i64,
    limit: i64,
    before_id: Option<i64>,
) -> Result<Vec<Message>> {
    let messages = if let Some(before) = before_id {
        sqlx::query_as!(
            Message,
            r#"
            SELECT * FROM messages 
            WHERE fund_id = ? AND id < ?
            ORDER BY id DESC
            LIMIT ?
            "#,
            fund_id,
            before,
            limit
        )
    } else {
        sqlx::query_as!(
            Message,
            r#"
            SELECT * FROM messages 
            WHERE fund_id = ?
            ORDER BY id DESC
            LIMIT ?
            "#,
            fund_id,
            limit
        )
    }
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

// Position operations
pub async fn create_position(
    pool: &SqlitePool,
    fund_id: i64,
    size: i64,
    price: i64,
    is_long: bool,
) -> Result<Position> {
    let position = sqlx::query_as!(
        Position,
        r#"
        INSERT INTO positions (fund_id, size, price, is_long)
        VALUES (?, ?, ?, ?)
        RETURNING *
        "#,
        fund_id,
        size,
        price,
        is_long,
    )
    .fetch_one(pool)
    .await?;

    Ok(position)
}

// Proposal operations
pub async fn create_proposal(
    pool: &SqlitePool,
    fund_id: i64,
    proposal_type: i64,
    end_time: DateTime<Utc>,
) -> Result<Proposal> {
    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        INSERT INTO proposals (fund_id, proposal_type, end_time)
        VALUES (?, ?, ?)
        RETURNING *
        "#,
        fund_id,
        proposal_type,
        end_time,
    )
    .fetch_one(pool)
    .await?;

    Ok(proposal)
}

pub async fn add_vote(
    pool: &SqlitePool,
    proposal_id: i64,
    voter: &AccountAddress,
    vote_yes: bool,
) -> Result<Vote> {
    let vote = sqlx::query_as!(
        Vote,
        r#"
        INSERT INTO votes (proposal_id, voter, vote_yes)
        VALUES (?, ?, ?)
        RETURNING *
        "#,
        proposal_id,
        voter.to_string(),
        vote_yes,
    )
    .fetch_one(pool)
    .await?;

    // Update proposal vote counts
    if vote_yes {
        sqlx::query!(
            r#"
            UPDATE proposals 
            SET votes_yes = votes_yes + 1
            WHERE id = ?
            "#,
            proposal_id
        )
        .execute(pool)
        .await?;
    } else {
        sqlx::query!(
            r#"
            UPDATE proposals 
            SET votes_no = votes_no + 1
            WHERE id = ?
            "#,
            proposal_id
        )
        .execute(pool)
        .await?;
    }

    Ok(vote)
}

// Asset operations
pub async fn create_asset(
    pool: &SqlitePool,
    symbol: &str,
    name: &str,
    decimals: i64,
    total_supply: i64,
) -> Result<Asset> {
    let asset = sqlx::query_as!(
        Asset,
        r#"
        INSERT INTO assets (symbol, name, decimals, total_supply)
        VALUES (?, ?, ?, ?)
        RETURNING *
        "#,
        symbol,
        name,
        decimals,
        total_supply,
    )
    .fetch_one(pool)
    .await?;

    Ok(asset)
}

// Balance operations
pub async fn update_balance(
    pool: &SqlitePool,
    asset_id: i64,
    holder: &AccountAddress,
    amount: i64,
) -> Result<Balance> {
    let balance = sqlx::query_as!(
        Balance,
        r#"
        INSERT INTO balances (asset_id, holder, amount)
        VALUES (?, ?, ?)
        ON CONFLICT(asset_id, holder) DO UPDATE SET
            amount = excluded.amount,
            last_updated = CURRENT_TIMESTAMP
        RETURNING *
        "#,
        asset_id,
        holder.to_string(),
        amount,
    )
    .fetch_one(pool)
    .await?;

    Ok(balance)
}

pub async fn get_balance(
    pool: &SqlitePool,
    asset_id: i64,
    holder: &AccountAddress,
) -> Result<Option<Balance>> {
    let balance = sqlx::query_as!(
        Balance,
        r#"
        SELECT * FROM balances 
        WHERE asset_id = ? AND holder = ?
        "#,
        asset_id,
        holder.to_string(),
    )
    .fetch_optional(pool)
    .await?;

    Ok(balance)
} 
