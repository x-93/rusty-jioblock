-- Initial database schema for JIO Blockchain Explorer

-- Blocks table
CREATE TABLE IF NOT EXISTS blocks (
    hash TEXT PRIMARY KEY,
    height INTEGER NOT NULL,
    version INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    bits INTEGER NOT NULL,
    nonce INTEGER NOT NULL,
    merkle_root TEXT NOT NULL,
    accepted_id_merkle_root TEXT,
    utxo_commitment TEXT,
    daa_score INTEGER NOT NULL,
    blue_score INTEGER NOT NULL,
    blue_work TEXT,
    pruning_point TEXT,
    size INTEGER NOT NULL,
    tx_count INTEGER NOT NULL,
    coinbase_value INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_blocks_height ON blocks(height);
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks(timestamp);
CREATE INDEX IF NOT EXISTS idx_blocks_blue_score ON blocks(blue_score);

-- Block parents table (DAG structure)
CREATE TABLE IF NOT EXISTS block_parents (
    block_hash TEXT NOT NULL,
    parent_hash TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (block_hash, parent_hash, level)
);

CREATE INDEX IF NOT EXISTS idx_block_parents_parent ON block_parents(parent_hash);

-- Transactions table
CREATE TABLE IF NOT EXISTS transactions (
    hash TEXT PRIMARY KEY,
    block_hash TEXT,
    block_height INTEGER,
    index_in_block INTEGER,
    version INTEGER NOT NULL,
    lock_time INTEGER,
    input_count INTEGER NOT NULL,
    output_count INTEGER NOT NULL,
    size INTEGER NOT NULL,
    fee INTEGER,
    value INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    is_coinbase BOOLEAN DEFAULT FALSE,
    is_confirmed BOOLEAN DEFAULT FALSE,
    confirmation_count INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_transactions_block_hash ON transactions(block_hash);
CREATE INDEX IF NOT EXISTS idx_transactions_block_height ON transactions(block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_timestamp ON transactions(timestamp);
CREATE INDEX IF NOT EXISTS idx_transactions_is_confirmed ON transactions(is_confirmed);

-- Transaction inputs table
CREATE TABLE IF NOT EXISTS transaction_inputs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tx_hash TEXT NOT NULL,
    "index" INTEGER NOT NULL,
    previous_outpoint_hash TEXT,
    previous_outpoint_index INTEGER,
    sequence INTEGER,
    script_sig BLOB,
    UNIQUE(tx_hash, "index")
);

CREATE INDEX IF NOT EXISTS idx_tx_inputs_tx_hash ON transaction_inputs(tx_hash);
CREATE INDEX IF NOT EXISTS idx_tx_inputs_previous_outpoint ON transaction_inputs(previous_outpoint_hash, previous_outpoint_index);

-- Transaction outputs table
CREATE TABLE IF NOT EXISTS transaction_outputs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tx_hash TEXT NOT NULL,
    "index" INTEGER NOT NULL,
    value INTEGER NOT NULL,
    script_public_key_version INTEGER,
    script_public_key_script BLOB,
    address TEXT,
    is_spent BOOLEAN DEFAULT FALSE,
    spent_by_tx_hash TEXT,
    spent_by_input_index INTEGER,
    UNIQUE(tx_hash, "index")
);

CREATE INDEX IF NOT EXISTS idx_tx_outputs_tx_hash ON transaction_outputs(tx_hash);
CREATE INDEX IF NOT EXISTS idx_tx_outputs_address ON transaction_outputs(address);
CREATE INDEX IF NOT EXISTS idx_tx_outputs_is_spent ON transaction_outputs(is_spent);

-- Addresses table
CREATE TABLE IF NOT EXISTS addresses (
    address TEXT PRIMARY KEY,
    first_seen_height INTEGER,
    first_seen_timestamp INTEGER,
    last_seen_height INTEGER,
    last_seen_timestamp INTEGER,
    tx_count INTEGER DEFAULT 0,
    received_count INTEGER DEFAULT 0,
    sent_count INTEGER DEFAULT 0,
    total_received INTEGER DEFAULT 0,
    total_sent INTEGER DEFAULT 0,
    balance INTEGER DEFAULT 0,
    utxo_count INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_addresses_balance ON addresses(balance);
CREATE INDEX IF NOT EXISTS idx_addresses_tx_count ON addresses(tx_count);

-- Address transactions junction table
CREATE TABLE IF NOT EXISTS address_transactions (
    address TEXT NOT NULL,
    tx_hash TEXT NOT NULL,
    is_input BOOLEAN NOT NULL,
    value INTEGER NOT NULL,
    PRIMARY KEY (address, tx_hash, is_input)
);

CREATE INDEX IF NOT EXISTS idx_address_txs_address ON address_transactions(address);
CREATE INDEX IF NOT EXISTS idx_address_txs_tx_hash ON address_transactions(tx_hash);

-- Mempool transactions table
CREATE TABLE IF NOT EXISTS mempool_transactions (
    hash TEXT PRIMARY KEY,
    version INTEGER NOT NULL,
    lock_time INTEGER,
    fee INTEGER,
    size INTEGER NOT NULL,
    first_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_seen DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Network statistics table
CREATE TABLE IF NOT EXISTS network_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL UNIQUE,
    block_count INTEGER NOT NULL,
    tx_count INTEGER NOT NULL,
    address_count INTEGER NOT NULL,
    total_supply INTEGER NOT NULL,
    hashrate INTEGER,
    difficulty REAL,
    avg_block_time REAL,
    mempool_size INTEGER,
    mempool_bytes INTEGER,
    peer_count INTEGER
);

CREATE INDEX IF NOT EXISTS idx_network_stats_timestamp ON network_stats(timestamp);
