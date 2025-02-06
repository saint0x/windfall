-- Enable foreign key support
PRAGMA foreign_keys = ON;

-- Add missing columns to proposals table
ALTER TABLE proposals ADD COLUMN vetoed BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE proposals ADD COLUMN chain_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE proposals ADD COLUMN synced BOOLEAN NOT NULL DEFAULT FALSE;

-- Add share column to fund_members table
ALTER TABLE fund_members ADD COLUMN share INTEGER NOT NULL DEFAULT 0;

-- Create fund_wallets table if it doesn't exist
CREATE TABLE IF NOT EXISTS fund_wallets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    wallet_address TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id),
    UNIQUE(fund_id, wallet_address)
);

-- Create investments table if it doesn't exist
CREATE TABLE IF NOT EXISTS investments (
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

-- Create indices for new tables
CREATE INDEX IF NOT EXISTS idx_fund_wallets_fund_id ON fund_wallets(fund_id);
CREATE INDEX IF NOT EXISTS idx_investments_fund_id ON investments(fund_id);
CREATE INDEX IF NOT EXISTS idx_investments_asset_id ON investments(asset_id); 