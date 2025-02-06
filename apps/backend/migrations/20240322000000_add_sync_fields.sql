-- Add version and status fields to funds table
ALTER TABLE funds ADD COLUMN version INTEGER NOT NULL DEFAULT 0;
ALTER TABLE funds ADD COLUMN status TEXT NOT NULL DEFAULT 'active';

-- Add version field to assets table
ALTER TABLE assets ADD COLUMN version INTEGER NOT NULL DEFAULT 0;

-- Add status field to fund_members table
ALTER TABLE fund_members ADD COLUMN status TEXT NOT NULL DEFAULT 'active';

-- Add address field to assets table for blockchain reference
ALTER TABLE assets ADD COLUMN address TEXT;

-- Create indices for better query performance
CREATE INDEX IF NOT EXISTS idx_funds_version ON funds(version);
CREATE INDEX IF NOT EXISTS idx_assets_version ON assets(version);
CREATE INDEX IF NOT EXISTS idx_fund_members_status ON fund_members(status);
CREATE INDEX IF NOT EXISTS idx_assets_address ON assets(address); 