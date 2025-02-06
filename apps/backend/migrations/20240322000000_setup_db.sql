-- Enable foreign key support
PRAGMA foreign_keys = ON;

-- Drop existing tables if they exist
DROP TABLE IF EXISTS balances;
DROP TABLE IF EXISTS votes;
DROP TABLE IF EXISTS positions;
DROP TABLE IF EXISTS messages;
DROP TABLE IF EXISTS investments;
DROP TABLE IF EXISTS fund_wallets;
DROP TABLE IF EXISTS fund_members;
DROP TABLE IF EXISTS proposals;
DROP TABLE IF EXISTS assets;
DROP TABLE IF EXISTS funds;

-- Create funds table
CREATE TABLE funds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    executor_address TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create assets table
CREATE TABLE assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    decimals INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    address TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create proposals table
CREATE TABLE proposals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    end_time DATETIME NOT NULL,
    executed BOOLEAN NOT NULL DEFAULT FALSE,
    vetoed BOOLEAN NOT NULL DEFAULT FALSE,
    chain_id INTEGER NOT NULL DEFAULT 0,
    synced BOOLEAN NOT NULL DEFAULT FALSE,
    proposer_address TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create fund_members table
CREATE TABLE fund_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    member_address TEXT NOT NULL,
    share INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id),
    UNIQUE(fund_id, member_address)
);

-- Create fund_wallets table
CREATE TABLE fund_wallets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    wallet_address TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id),
    UNIQUE(fund_id)
);

-- Create investments table
CREATE TABLE investments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    asset_id INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    withdrawn_amount INTEGER NOT NULL DEFAULT 0,
    investor_address TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id),
    FOREIGN KEY (asset_id) REFERENCES assets(id)
);

-- Create messages table
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    sender_address TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create positions table
CREATE TABLE positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    asset_id INTEGER NOT NULL,
    size INTEGER NOT NULL,
    entry_price INTEGER NOT NULL,
    is_long BOOLEAN NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id),
    FOREIGN KEY (asset_id) REFERENCES assets(id)
);

-- Create votes table
CREATE TABLE votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proposal_id INTEGER NOT NULL,
    voter_address TEXT NOT NULL,
    vote_type BOOLEAN NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (proposal_id) REFERENCES proposals(id),
    UNIQUE(proposal_id, voter_address)
);

-- Create balances table
CREATE TABLE balances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    holder_address TEXT NOT NULL,
    amount INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (asset_id) REFERENCES assets(id),
    UNIQUE(asset_id, holder_address)
);

-- Create indices for better query performance
CREATE INDEX idx_funds_version ON funds(version);
CREATE INDEX idx_assets_version ON assets(version);
CREATE INDEX idx_fund_members_status ON fund_members(status);
CREATE INDEX idx_assets_address ON assets(address);
CREATE INDEX idx_fund_members_fund_id ON fund_members(fund_id);
CREATE INDEX idx_fund_wallets_fund_id ON fund_wallets(fund_id);
CREATE INDEX idx_investments_fund_id ON investments(fund_id);
CREATE INDEX idx_investments_asset_id ON investments(asset_id);
CREATE INDEX idx_messages_fund_id ON messages(fund_id, id DESC);
CREATE INDEX idx_positions_fund_id ON positions(fund_id);
CREATE INDEX idx_votes_proposal_id ON votes(proposal_id);
CREATE INDEX idx_balances_asset_id ON balances(asset_id); 