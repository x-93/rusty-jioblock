// API Response Types for JIO Blockchain Explorer

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export interface BlockSummary {
  hash: string;
  height: number;
  timestamp: number;
  tx_count: number;
  size: number;
  weight?: number;
  version: number;
  merkle_root: string;
  bits: number;
  nonce: number;
  difficulty?: number;
  blue_score?: number;
  daa_score?: number;
  coinbase_value?: number;
}

export interface TransactionSummary {
  hash: string;
  block_hash?: string;
  block_height?: number;
  timestamp: number;
  size: number;
  fee?: number;
  value: number;
  input_count: number;
  output_count: number;
  is_coinbase: boolean;
  is_confirmed: boolean;
  confirmation_count?: number;
}

export interface AddressSummary {
  address: string;
  balance: number;
  tx_count: number;
  received_count: number;
  sent_count: number;
  total_received: number;
  total_sent: number;
  utxo_count: number;
  first_seen_height?: number;
  first_seen_timestamp?: number;
  last_seen_height?: number;
  last_seen_timestamp?: number;
}

export interface NetworkStats {
  block_count: number;
  tx_count: number;
  address_count: number;
  total_supply: number;
  hashrate?: number;
  difficulty?: number;
  avg_block_time?: number;
  mempool_size: number;
  mempool_bytes: number;
  peer_count: number;
  timestamp: number;
}

export interface MiningInfo {
  network_hash_ps: number;
  pooled_tx: number;
  chain: string;
  warnings: string;
  difficulty: number;
  blocks: number;
  current_block_weight?: number;
  current_block_tx?: number;
  errors?: string;
}

export interface BlockDagInfo {
  block_count: number;
  tip_hashes: string[];
  difficulty: number;
  network: string;
  virtual_parent_hashes: string[];
  pruning_point_hash: string;
}

export interface SearchResults {
  blocks: BlockSummary[];
  transactions: TransactionSummary[];
  addresses: AddressSummary[];
  total: number;
}

export interface TransactionInput {
  previous_outpoint_hash?: string;
  previous_outpoint_index?: number;
  sequence?: number;
  script_sig?: string;
  value?: number;
  address?: string;
}

export interface TransactionOutput {
  value: number;
  script_public_key_script: string;
  script_public_key_version?: number;
  address?: string;
  is_spent: boolean;
  spent_by_tx_hash?: string;
  spent_by_input_index?: number;
}

export interface TransactionDetails extends TransactionSummary {
  inputs: TransactionInput[];
  outputs: TransactionOutput[];
  lock_time?: number;
  version: number;
}

export interface BlockDetails extends BlockSummary {
  transactions: TransactionSummary[];
  parent_hashes: string[];
  accepted_id_merkle_root?: string;
  utxo_commitment?: string;
  blue_work?: string;
  pruning_point?: string;
}

export interface AddressTransaction {
  hash: string;
  timestamp: number;
  value: number;
  is_input: boolean;
  block_height?: number;
  is_confirmed: boolean;
}

export interface AddressUTXO {
  tx_hash: string;
  index: number;
  value: number;
  script_public_key_script: string;
  address: string;
  block_height?: number;
  timestamp: number;
}

// WebSocket Event Types
export interface WSBlockEvent {
  type: 'block:new';
  data: BlockSummary;
}

export interface WSTransactionEvent {
  type: 'transaction:new';
  data: TransactionSummary;
}

export interface WSMempoolEvent {
  type: 'mempool:update';
  data: {
    size: number;
    bytes: number;
    transactions: TransactionSummary[];
  };
}

export interface WSNetworkStatsEvent {
  type: 'network:stats';
  data: NetworkStats;
}

export type WSEvent = WSBlockEvent | WSTransactionEvent | WSMempoolEvent | WSNetworkStatsEvent;

// API Error Response
export interface APIError {
  error: string;
  message: string;
  status_code: number;
}
