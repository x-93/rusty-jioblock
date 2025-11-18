//! Database schema definitions

// This module will contain the SQL schema
// The actual migrations will be in the migrations/ directory

pub const CREATE_BLOCKS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS blocks (
    hash VARCHAR(64) PRIMARY KEY,
    height BIGINT NOT NULL,
    version INTEGER NOT NULL,
    timestamp BIGINT NOT NULL,
    bits INTEGER NOT NULL,
    nonce BIGINT NOT NULL,
    merkle_root VARCHAR(64) NOT NULL,
    accepted_id_merkle_root VARCHAR(64),
    utxo_commitment VARCHAR(64),
    daa_score BIGINT NOT NULL,
    blue_score BIGINT NOT NULL,
    blue_work VARCHAR(128),
    pruning_point VARCHAR(64),
    size INTEGER NOT NULL,
    tx_count INTEGER NOT NULL,
    coinbase_value BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_blocks_height ON blocks(height);
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks(timestamp);
CREATE INDEX IF NOT EXISTS idx_blocks_blue_score ON blocks(blue_score);
"#;

pub const CREATE_BLOCK_PARENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS block_parents (
    block_hash VARCHAR(64) NOT NULL,
    parent_hash VARCHAR(64) NOT NULL,
    level INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (block_hash, parent_hash, level)
);

CREATE INDEX IF NOT EXISTS idx_block_parents_parent ON block_parents(parent_hash);
"#;

pub const CREATE_TRANSACTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS transactions (
    hash VARCHAR(64) PRIMARY KEY,
    block_hash VARCHAR(64),
    block_height BIGINT,
    index_in_block INTEGER,
    version INTEGER NOT NULL,
    lock_time BIGINT,
    input_count INTEGER NOT NULL,
    output_count INTEGER NOT NULL,
    size INTEGER NOT NULL,
    fee BIGINT,
    value BIGINT NOT NULL,
    timestamp BIGINT NOT NULL,
    is_coinbase BOOLEAN DEFAULT FALSE,
    is_confirmed BOOLEAN DEFAULT FALSE,
    confirmation_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_transactions_block_hash ON transactions(block_hash);
CREATE INDEX IF NOT EXISTS idx_transactions_block_height ON transactions(block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_timestamp ON transactions(timestamp);
CREATE INDEX IF NOT EXISTS idx_transactions_is_confirmed ON transactions(is_confirmed);
"#;

pub const CREATE_TRANSACTION_INPUTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS transaction_inputs (
    id SERIAL PRIMARY KEY,
    tx_hash VARCHAR(64) NOT NULL,
    index INTEGER NOT NULL,
    previous_outpoint_hash VARCHAR(64),
    previous_outpoint_index INTEGER,
    sequence BIGINT,
    script_sig BYTEA,
    UNIQUE(tx_hash, index)
);

CREATE INDEX IF NOT EXISTS idx_tx_inputs_tx_hash ON transaction_inputs(tx_hash);
CREATE INDEX IF NOT EXISTS idx_tx_inputs_previous_outpoint ON transaction_inputs(previous_outpoint_hash, previous_outpoint_index);
"#;

pub const CREATE_TRANSACTION_OUTPUTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS transaction_outputs (
    id SERIAL PRIMARY KEY,
    tx_hash VARCHAR(64) NOT NULL,
    index INTEGER NOT NULL,
    value BIGINT NOT NULL,
    script_public_key_version INTEGER,
    script_public_key_script BYTEA,
    address VARCHAR(255),
    is_spent BOOLEAN DEFAULT FALSE,
    spent_by_tx_hash VARCHAR(64),
    spent_by_input_index INTEGER,
    UNIQUE(tx_hash, index)
);

CREATE INDEX IF NOT EXISTS idx_tx_outputs_tx_hash ON transaction_outputs(tx_hash);
CREATE INDEX IF NOT EXISTS idx_tx_outputs_address ON transaction_outputs(address);
CREATE INDEX IF NOT EXISTS idx_tx_outputs_is_spent ON transaction_outputs(is_spent);
"#;

pub const CREATE_ADDRESSES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS addresses (
    address VARCHAR(255) PRIMARY KEY,
    first_seen_height BIGINT,
    first_seen_timestamp BIGINT,
    last_seen_height BIGINT,
    last_seen_timestamp BIGINT,
    tx_count INTEGER DEFAULT 0,
    received_count INTEGER DEFAULT 0,
    sent_count INTEGER DEFAULT 0,
    total_received BIGINT DEFAULT 0,
    total_sent BIGINT DEFAULT 0,
    balance BIGINT DEFAULT 0,
    utxo_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_addresses_balance ON addresses(balance);
CREATE INDEX IF NOT EXISTS idx_addresses_tx_count ON addresses(tx_count);
"#;

pub const CREATE_ADDRESS_TRANSACTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS address_transactions (
    address VARCHAR(255) NOT NULL,
    tx_hash VARCHAR(64) NOT NULL,
    is_input BOOLEAN NOT NULL,
    value BIGINT NOT NULL,
    PRIMARY KEY (address, tx_hash, is_input)
);

CREATE INDEX IF NOT EXISTS idx_address_txs_address ON address_transactions(address);
CREATE INDEX IF NOT EXISTS idx_address_txs_tx_hash ON address_transactions(tx_hash);
"#;

pub const CREATE_MEMPOOL_TRANSACTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS mempool_transactions (
    hash VARCHAR(64) PRIMARY KEY,
    version INTEGER NOT NULL,
    lock_time BIGINT,
    fee BIGINT,
    size INTEGER NOT NULL,
    first_seen TIMESTAMP DEFAULT NOW(),
    last_seen TIMESTAMP DEFAULT NOW()
);
"#;

pub const CREATE_NETWORK_STATS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS network_stats (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL UNIQUE,
    block_count BIGINT NOT NULL,
    tx_count BIGINT NOT NULL,
    address_count BIGINT NOT NULL,
    total_supply BIGINT NOT NULL,
    hashrate BIGINT,
    difficulty DOUBLE PRECISION,
    avg_block_time DOUBLE PRECISION,
    mempool_size INTEGER,
    mempool_bytes BIGINT,
    peer_count INTEGER
);

CREATE INDEX IF NOT EXISTS idx_network_stats_timestamp ON network_stats(timestamp);
"#;

