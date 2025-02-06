use sqlx::{Pool, Sqlite};
use sqlx::FromRow;
use anyhow::Context;
use aptos_sdk::types::account_address::AccountAddress;
use crate::db::types::DbDateTime;
use crate::error::{AppError, Result};
use crate::db::schema::*;
use crate::sync::HolderInfo;

async fn get_by_id<T>(pool: &Pool<Sqlite>, table: &str, id: i64) -> Result<T>
where
    T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
{
    sqlx::query_as::<_, T>(&format!(
        r#"
        SELECT * FROM {} WHERE id = ? LIMIT 1
        "#,
        table
    ))
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound(format!("{} with id {} not found", table, id)),
        e => AppError::Database(e)
    })
}

// Fund operations
pub async fn create_fund(
    pool: &Pool<Sqlite>,
    name: String,
    executor_address: String,
) -> Result<Fund> {
    let now = DbDateTime::now();
    
    sqlx::query_as!(
        Fund,
        r#"
        INSERT INTO funds (name, executor_address, version, status, created_at, updated_at)
        VALUES (?, ?, 1, 'active', ?, ?)
        RETURNING 
            id as "id!", 
            name as "name!", 
            executor_address as "executor_address!",
            version as "version!",
            status as "status!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        name,
        executor_address,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(e) if e.message().contains("UNIQUE constraint failed") => {
            AppError::InvalidInput(format!("Fund with name {} already exists", name))
        }
        e => AppError::Database(e)
    })
}

pub async fn get_fund(pool: &Pool<Sqlite>, fund_id: i64) -> Result<Fund> {
    get_by_id::<Fund>(pool, "funds", fund_id).await
}

// Fund member operations
pub async fn add_fund_member(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    member_address: AccountAddress,
) -> Result<FundMember> {
    let now = DbDateTime::now();
    let member_str = member_address.to_string();
    
    let member = sqlx::query_as!(
        FundMember,
        r#"
        INSERT INTO fund_members (fund_id, member_address, share, status, created_at, updated_at)
        VALUES (?, ?, 0, 'active', ?, ?)
        RETURNING 
            id as "id!", 
            fund_id as "fund_id!", 
            member_address as "member_address!", 
            share as "share!",
            status as "status!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        fund_id,
        member_str,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to add fund member")?;

    Ok(member)
}

pub async fn get_fund_members(
    pool: &Pool<Sqlite>,
    fund_id: i64,
) -> Result<Vec<FundMember>> {
    let members = sqlx::query_as!(
        FundMember,
        r#"
        SELECT 
            id as "id!", 
            fund_id as "fund_id!", 
            member_address as "member_address!", 
            share as "share!",
            status as "status!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM fund_members WHERE fund_id = ?
        "#,
        fund_id
    )
    .fetch_all(pool)
    .await
    .context("Failed to get fund members")?;

    Ok(members)
}

// Message operations
pub async fn create_message(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    sender_address: String,
    content: String,
) -> Result<Message> {
    let now = DbDateTime::now();
    
    let message = sqlx::query_as!(
        Message,
        r#"
        INSERT INTO messages (fund_id, sender_address, content, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING 
            id as "id!", 
            fund_id as "fund_id!", 
            sender_address as "sender_address!", 
            content as "content!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        fund_id,
        sender_address,
        content,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create message")?;
    
    Ok(message)
}

pub async fn get_messages(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    limit: i64,
    before_id: Option<i64>,
) -> Result<Vec<Message>> {
    let query = format!(
        r#"
        SELECT 
            id as "id!", 
            fund_id as "fund_id!", 
            sender_address as "sender_address!", 
            content as "content!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM messages 
        WHERE fund_id = ?1 {}
        ORDER BY id DESC
        LIMIT ?2
        "#,
        before_id.map_or(String::new(), |_| String::from("AND id < ?3"))
    );

    let mut q = sqlx::query_as::<_, Message>(&query)
        .bind(fund_id)
        .bind(limit);

    if let Some(before) = before_id {
        q = q.bind(before);
    }

    q.fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

// Position operations
pub async fn create_position(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    asset_id: i64,
    size: i64,
    entry_price: i64,
    is_long: bool,
) -> Result<Position> {
    let now = DbDateTime::now();
    let position = sqlx::query_as!(
        Position,
        r#"
        INSERT INTO positions (fund_id, asset_id, size, entry_price, is_long, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        RETURNING 
            id as "id!", 
            fund_id as "fund_id!", 
            asset_id as "asset_id!", 
            size as "size!", 
            entry_price as "entry_price!", 
            is_long as "is_long!", 
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        fund_id,
        asset_id,
        size,
        entry_price,
        is_long,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create position")?;

    Ok(position)
}

pub async fn get_position_by_id(pool: &Pool<Sqlite>, position_id: i64) -> Result<Position> {
    get_by_id::<Position>(pool, "positions", position_id).await
}

// Proposal operations
pub async fn create_proposal(
    pool: &Pool<Sqlite>,
    title: &str,
    description: &str,
    end_time: DbDateTime,
) -> Result<Proposal> {
    let now = DbDateTime::now();
    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        INSERT INTO proposals (
            title, description, end_time, 
            executed, vetoed, chain_id, synced,
            proposer_address, created_at, updated_at
        )
        VALUES (?, ?, ?, false, false, 0, false, NULL, ?, ?)
        RETURNING 
            id as "id!", 
            title as "title!", 
            description as "description!", 
            end_time as "end_time!", 
            executed as "executed!", 
            vetoed as "vetoed!", 
            chain_id as "chain_id!", 
            synced as "synced!",
            proposer_address,
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        title,
        description,
        end_time,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create proposal")?;
    Ok(proposal)
}

pub async fn get_proposal_by_id(
    pool: &Pool<Sqlite>,
    proposal_id: i64,
) -> Result<Proposal> {
    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        SELECT 
            id as "id!", 
            title as "title!", 
            description as "description!", 
            end_time as "end_time!", 
            executed as "executed!", 
            vetoed as "vetoed!", 
            chain_id as "chain_id!", 
            synced as "synced!",
            proposer_address,
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM proposals WHERE id = ?
        "#,
        proposal_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to get proposal")?;

    Ok(proposal)
}

pub async fn get_proposal(pool: &Pool<Sqlite>, proposal_id: i64) -> Result<Proposal> {
    get_by_id::<Proposal>(pool, "proposals", proposal_id).await
}

pub async fn vote_on_proposal(
    pool: &Pool<Sqlite>,
    proposal_id: i64,
    voter_address: AccountAddress,
    vote_type: bool,
) -> Result<Vote> {
    let now = DbDateTime::now();
    let voter_str = voter_address.to_string();
    
    let vote = sqlx::query_as!(
        Vote,
        r#"
        INSERT INTO votes (proposal_id, voter_address, vote_type, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING 
            id as "id!", 
            proposal_id as "proposal_id!", 
            voter_address as "voter_address!", 
            vote_type as "vote_type!", 
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        proposal_id,
        voter_str,
        vote_type,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create vote")?;

    Ok(vote)
}

pub async fn emergency_veto_proposal(
    pool: &Pool<Sqlite>,
    proposal_id: i64,
) -> Result<Proposal> {
    let now = DbDateTime::now();
    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        UPDATE proposals 
        SET vetoed = true, updated_at = ?
        WHERE id = ?
        RETURNING 
            id as "id!", 
            title as "title!", 
            description as "description!", 
            end_time as "end_time!", 
            executed as "executed!", 
            vetoed as "vetoed!", 
            chain_id as "chain_id!", 
            synced as "synced!",
            proposer_address,
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        now,
        proposal_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to veto proposal")?;
    Ok(proposal)
}

pub async fn sync_proposal_creation(
    pool: &Pool<Sqlite>,
    proposal_id: u64,
    proposer: &str,
) -> Result<Proposal> {
    let now = DbDateTime::now();
    let chain_id = proposal_id as i64;
    
    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        UPDATE proposals 
        SET chain_id = ?, proposer_address = ?, synced = true, updated_at = ?
        WHERE id = ?
        RETURNING 
            id as "id!", 
            title as "title!", 
            description as "description!", 
            end_time as "end_time!", 
            executed as "executed!", 
            vetoed as "vetoed!", 
            chain_id as "chain_id!", 
            synced as "synced!",
            proposer_address,
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        chain_id,
        proposer,
        now,
        chain_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to sync proposal creation")?;

    Ok(proposal)
}

pub async fn sync_proposal_execution(
    pool: &Pool<Sqlite>,
    proposal_id: u64,
) -> Result<Proposal> {
    let now = DbDateTime::now();
    let id = proposal_id as i64;
    
    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        UPDATE proposals 
        SET executed = true, updated_at = ?
        WHERE id = ?
        RETURNING 
            id as "id!", 
            title as "title!", 
            description as "description!", 
            end_time as "end_time!", 
            executed as "executed!", 
            vetoed as "vetoed!", 
            chain_id as "chain_id!", 
            synced as "synced!",
            proposer_address,
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        now,
        id
    )
    .fetch_one(pool)
    .await
    .context("Failed to sync proposal execution")?;

    Ok(proposal)
}

pub async fn sync_proposal_veto(
    pool: &Pool<Sqlite>,
    proposal_id: u64,
) -> Result<Proposal> {
    let now = DbDateTime::now();
    let id = proposal_id as i64;
    
    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        UPDATE proposals 
        SET vetoed = true, updated_at = ?
        WHERE id = ?
        RETURNING 
            id as "id!", 
            title as "title!", 
            description as "description!", 
            end_time as "end_time!", 
            executed as "executed!", 
            vetoed as "vetoed!", 
            chain_id as "chain_id!", 
            synced as "synced!",
            proposer_address,
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        now,
        id
    )
    .fetch_one(pool)
    .await
    .context("Failed to sync proposal veto")?;

    Ok(proposal)
}

// Asset operations
pub async fn create_asset(
    pool: &Pool<Sqlite>,
    symbol: String,
    name: String,
    decimals: i32,
) -> Result<Asset> {
    let now = DbDateTime::now();
    
    let asset = sqlx::query_as!(
        Asset,
        r#"
        INSERT INTO assets (symbol, name, decimals, version, address, total_supply, created_at, updated_at)
        VALUES (?, ?, ?, 1, NULL, 0, ?, ?)
        RETURNING 
            id as "id!", 
            symbol as "symbol!", 
            name as "name!", 
            decimals as "decimals!: i32",
            version as "version!",
            address as "address",
            total_supply as "total_supply!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        symbol,
        name,
        decimals,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create asset")?;

    Ok(asset)
}

// Balance operations
pub async fn get_asset_balances(
    pool: &Pool<Sqlite>,
    asset_id: i64,
) -> Result<Vec<Balance>> {
    let balances = sqlx::query_as!(
        Balance,
        r#"
        SELECT 
            id as "id!", 
            asset_id as "asset_id!", 
            holder_address as "holder_address!", 
            amount as "amount!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM balances WHERE asset_id = ?
        "#,
        asset_id
    )
    .fetch_all(pool)
    .await
    .context("Failed to get asset balances")?;

    Ok(balances)
}

pub async fn create_balance(
    pool: &Pool<Sqlite>,
    asset_id: i64,
    holder_address: AccountAddress,
    amount: i64,
) -> Result<Balance> {
    let now = DbDateTime::now();
    let holder_str = holder_address.to_string();
    
    let balance = sqlx::query_as!(
        Balance,
        r#"
        INSERT INTO balances (asset_id, holder_address, amount, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING 
            id as "id!", 
            asset_id as "asset_id!", 
            holder_address as "holder_address!", 
            amount as "amount!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        asset_id,
        holder_str,
        amount,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create balance")?;

    Ok(balance)
}

pub async fn get_fund_by_id(pool: &Pool<Sqlite>, fund_id: i64) -> Result<Fund> {
    get_fund(pool, fund_id).await
}

pub async fn get_fund_messages(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    limit: i64,
    before_id: Option<i64>,
) -> Result<Vec<Message>> {
    let query = format!(
        r#"
        SELECT 
            id as "id!", 
            fund_id as "fund_id!", 
            sender_address as "sender_address!", 
            content as "content!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM messages 
        WHERE fund_id = ?1 {}
        ORDER BY id DESC
        LIMIT ?2
        "#,
        before_id.map_or(String::new(), |_| String::from("AND id < ?3"))
    );

    let mut q = sqlx::query_as::<_, Message>(&query)
        .bind(fund_id)
        .bind(limit);

    if let Some(before) = before_id {
        q = q.bind(before);
    }

    q.fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn create_fund_wallet(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    wallet_address: &str,
) -> Result<FundWallet> {
    let now = DbDateTime::now();
    let wallet = sqlx::query_as!(
        FundWallet,
        r#"
        INSERT INTO fund_wallets (fund_id, wallet_address, created_at, updated_at)
        VALUES (?, ?, ?, ?)
        RETURNING 
            id as "id!", 
            fund_id as "fund_id!", 
            wallet_address as "wallet_address!", 
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        fund_id,
        wallet_address,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create fund wallet")?;
    Ok(wallet)
}

pub async fn get_fund_wallet(
    pool: &Pool<Sqlite>,
    fund_id: i64,
) -> Result<FundWallet> {
    let wallet = sqlx::query_as!(
        FundWallet,
        r#"
        SELECT 
            id as "id!", 
            fund_id as "fund_id!", 
            wallet_address as "wallet_address!", 
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM fund_wallets 
        WHERE fund_id = ?
        "#,
        fund_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to get fund wallet")?;
    Ok(wallet)
}

pub async fn create_investment(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    asset_id: i64,
    amount: i64,
    investor_address: &str,
) -> Result<Investment> {
    let now = DbDateTime::now();
    let investment = sqlx::query_as!(
        Investment,
        r#"
        INSERT INTO investments (
            fund_id, asset_id, amount, withdrawn_amount,
            investor_address, created_at, updated_at
        )
        VALUES (?, ?, ?, 0, ?, ?, ?)
        RETURNING 
            id as "id!", 
            fund_id as "fund_id!", 
            asset_id as "asset_id!", 
            amount as "amount!", 
            withdrawn_amount as "withdrawn_amount!", 
            investor_address as "investor_address!", 
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        fund_id,
        asset_id,
        amount,
        investor_address,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create investment")?;
    Ok(investment)
}

pub async fn withdraw_investment(
    pool: &Pool<Sqlite>,
    investment_id: i64,
    amount: i64,
) -> Result<Investment> {
    let now = DbDateTime::now();
    let investment = sqlx::query_as!(
        Investment,
        r#"
        UPDATE investments 
        SET withdrawn_amount = withdrawn_amount + ?, updated_at = ?
        WHERE id = ?
        RETURNING 
            id as "id!", 
            fund_id as "fund_id!", 
            asset_id as "asset_id!", 
            amount as "amount!", 
            withdrawn_amount as "withdrawn_amount!", 
            investor_address as "investor_address!", 
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        amount,
        now,
        investment_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to withdraw investment")?;
    Ok(investment)
}

pub async fn update_member_share(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    member_address: &str,
    new_share: i64,
) -> Result<FundMember> {
    let now = DbDateTime::now();
    Ok(sqlx::query_as!(
        FundMember,
        r#"
        UPDATE fund_members 
        SET share = ?, updated_at = ?
        WHERE fund_id = ? AND member_address = ?
        RETURNING 
            id as "id!", 
            fund_id as "fund_id!", 
            member_address as "member_address!", 
            share as "share!",
            status as "status!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        new_share,
        now,
        fund_id,
        member_address
    )
    .fetch_one(pool)
    .await
    .context("Failed to update member share")?)
}

pub async fn update_balances(
    pool: &Pool<Sqlite>,
    symbol: &str,
    from_address: &str,
    to_address: &str,
    amount: u64,
) -> Result<()> {
    let now = DbDateTime::now();
    let amount_i64 = amount as i64;
    
    // Get asset ID from symbol
    let asset = sqlx::query_as!(
        Asset,
        r#"
        SELECT 
            id as "id!", 
            symbol as "symbol!", 
            name as "name!", 
            decimals as "decimals!: i32",
            version as "version!",
            address,
            total_supply as "total_supply!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM assets WHERE symbol = ?
        "#,
        symbol
    )
    .fetch_one(pool)
    .await
    .context("Failed to find asset")?;

    // Update sender's balance
    sqlx::query!(
        r#"
        UPDATE balances 
        SET amount = amount - ?, updated_at = ?
        WHERE asset_id = ? AND holder_address = ?
        "#,
        amount_i64,
        now,
        asset.id,
        from_address
    )
    .execute(pool)
    .await
    .context("Failed to update sender balance")?;

    // Update receiver's balance
    sqlx::query!(
        r#"
        UPDATE balances 
        SET amount = amount + ?, updated_at = ?
        WHERE asset_id = ? AND holder_address = ?
        "#,
        amount_i64,
        now,
        asset.id,
        to_address
    )
    .execute(pool)
    .await
    .context("Failed to update receiver balance")?;

    Ok(())
}

pub async fn sync_member_addition(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    member_address: &str,
) -> Result<FundMember> {
    let now = DbDateTime::now();
    
    let member = sqlx::query_as!(
        FundMember,
        r#"
        INSERT INTO fund_members (fund_id, member_address, share, status, created_at, updated_at)
        VALUES (?, ?, 0, 'active', ?, ?)
        RETURNING 
            id as "id!", 
            fund_id as "fund_id!", 
            member_address as "member_address!", 
            share as "share!",
            status as "status!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        fund_id,
        member_address,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to sync member addition")?;

    Ok(member)
}

pub async fn sync_member_removal(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    member_address: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM fund_members
        WHERE fund_id = ? AND member_address = ?
        "#,
        fund_id,
        member_address
    )
    .execute(pool)
    .await
    .context("Failed to sync member removal")?;

    Ok(())
}

pub async fn get_asset_by_symbol(
    pool: &Pool<Sqlite>,
    symbol: &str,
) -> Result<Asset> {
    Ok(sqlx::query_as!(
        Asset,
        r#"
        SELECT 
            id as "id!", 
            symbol as "symbol!", 
            name as "name!", 
            decimals as "decimals!: i32",
            version as "version!",
            address,
            total_supply as "total_supply!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM assets 
        WHERE symbol = ?
        "#,
        symbol
    )
    .fetch_one(pool)
    .await
    .context("Failed to get asset")?)
}

pub async fn create_fund_member(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    member_address: &str,
    share: i64,
) -> Result<FundMember> {
    let now = DbDateTime::now();
    Ok(sqlx::query_as!(
        FundMember,
        r#"
        INSERT INTO fund_members (fund_id, member_address, share, status, created_at, updated_at)
        VALUES (?, ?, ?, 'active', ?, ?)
        RETURNING 
            id as "id!", 
            fund_id as "fund_id!", 
            member_address as "member_address!", 
            share as "share!",
            status as "status!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        fund_id,
        member_address,
        share,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create fund member")?)
}

pub async fn get_fund_member(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    member_address: &str,
) -> Result<FundMember> {
    Ok(sqlx::query_as!(
        FundMember,
        r#"
        SELECT 
            id as "id!", 
            fund_id as "fund_id!", 
            member_address as "member_address!", 
            share as "share!",
            status as "status!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM fund_members 
        WHERE fund_id = ? AND member_address = ?
        "#,
        fund_id,
        member_address
    )
    .fetch_one(pool)
    .await
    .context("Failed to get fund member")?)
}

pub async fn get_all_funds(pool: &Pool<Sqlite>) -> Result<Vec<Fund>> {
    let funds = sqlx::query_as!(
        Fund,
        r#"
        SELECT 
            id as "id!", 
            name as "name!", 
            executor_address as "executor_address!",
            version as "version!",
            status as "status!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM funds
        "#
    )
    .fetch_all(pool)
    .await
    .context("Failed to get all funds")?;

    Ok(funds)
}

pub async fn update_fund_state(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    version: u64,
    status: String,
) -> Result<Fund> {
    let now = DbDateTime::now();
    let version_i64 = version as i64;
    
    let fund = sqlx::query_as!(
        Fund,
        r#"
        UPDATE funds 
        SET version = ?, status = ?, updated_at = ?
        WHERE id = ?
        RETURNING 
            id as "id!", 
            name as "name!", 
            executor_address as "executor_address!",
            version as "version!",
            status as "status!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        version_i64,
        status,
        now,
        fund_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to update fund state")?;

    Ok(fund)
}

pub async fn sync_member_state(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    member_address: &str,
    share: u64,
    status: String,
) -> Result<FundMember> {
    let now = DbDateTime::now();
    let share_i64 = share as i64;
    
    // Try to update existing member first
    let result = sqlx::query_as!(
        FundMember,
        r#"
        UPDATE fund_members 
        SET share = ?, status = ?, updated_at = ?
        WHERE fund_id = ? AND member_address = ?
        RETURNING 
            id as "id!", 
            fund_id as "fund_id!", 
            member_address as "member_address!", 
            share as "share!",
            status as "status!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        share_i64,
        status,
        now,
        fund_id,
        member_address
    )
    .fetch_optional(pool)
    .await
    .context("Failed to update member state")?;

    match result {
        Some(member) => Ok(member),
        None => {
            // Member doesn't exist, create new one
            let member = sqlx::query_as!(
                FundMember,
                r#"
                INSERT INTO fund_members (fund_id, member_address, share, status, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?)
                RETURNING 
                    id as "id!", 
                    fund_id as "fund_id!", 
                    member_address as "member_address!", 
                    share as "share!",
                    status as "status!",
                    created_at as "created_at!", 
                    updated_at as "updated_at!"
                "#,
                fund_id,
                member_address,
                share_i64,
                status,
                now,
                now
            )
            .fetch_one(pool)
            .await
            .context("Failed to create member")?;

            Ok(member)
        }
    }
}

pub async fn get_all_assets(pool: &Pool<Sqlite>) -> Result<Vec<Asset>> {
    let assets = sqlx::query_as!(
        Asset,
        r#"
        SELECT 
            id as "id!", 
            symbol as "symbol!", 
            name as "name!", 
            decimals as "decimals!: i32",
            version as "version!",
            address,
            total_supply as "total_supply!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM assets
        "#
    )
    .fetch_all(pool)
    .await
    .context("Failed to get all assets")?;

    Ok(assets)
}

pub async fn update_asset_state(
    pool: &Pool<Sqlite>,
    asset_id: i64,
    version: u64,
    total_supply: u64,
    holders: Vec<HolderInfo>,
) -> Result<Asset> {
    let now = DbDateTime::now();
    let version_i64 = version as i64;
    let total_supply_i64 = total_supply as i64;
    
    // Start a transaction
    let mut tx = pool.begin().await?;

    // Update asset
    let asset = sqlx::query_as!(
        Asset,
        r#"
        UPDATE assets 
        SET version = ?, total_supply = ?, updated_at = ?
        WHERE id = ?
        RETURNING 
            id as "id!", 
            symbol as "symbol!", 
            name as "name!", 
            decimals as "decimals!: i32",
            version as "version!",
            address as "address",
            total_supply as "total_supply!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        "#,
        version_i64,
        total_supply_i64,
        now,
        asset_id
    )
    .fetch_one(&mut *tx)
    .await
    .context("Failed to update asset state")?;

    // Update balances for all holders
    for holder in holders {
        let balance_i64 = holder.balance as i64;
        sqlx::query!(
            r#"
            INSERT INTO balances (asset_id, holder_address, amount, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT (asset_id, holder_address) DO UPDATE
            SET amount = excluded.amount, updated_at = excluded.updated_at
            "#,
            asset_id,
            holder.address,
            balance_i64,
            now,
            now
        )
        .execute(&mut *tx)
        .await
        .context("Failed to update holder balance")?;
    }

    // Commit transaction
    tx.commit().await?;

    Ok(asset)
}

pub async fn get_asset_by_id(
    pool: &Pool<Sqlite>,
    id: i64,
) -> Result<Asset> {
    Ok(sqlx::query_as!(
        Asset,
        r#"
        SELECT 
            id as "id!", 
            symbol as "symbol!", 
            name as "name!", 
            decimals as "decimals!: i32",
            version as "version!",
            address,
            total_supply as "total_supply!",
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM assets 
        WHERE id = ?
        "#,
        id
    )
    .fetch_one(pool)
    .await
    .context("Failed to get asset")?)
}

