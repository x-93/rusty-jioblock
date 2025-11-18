import axios, { AxiosResponse } from 'axios';
import {
  PaginatedResponse,
  BlockSummary,
  TransactionSummary,
  AddressSummary,
  NetworkStats,
  MiningInfo,
  BlockDagInfo,
  SearchResults,
  TransactionDetails,
  BlockDetails,
  AddressTransaction,
  AddressUTXO,
  APIError,
} from '@/types/api';

const API_BASE_URL = 'http://localhost:3000';

const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor for logging
api.interceptors.request.use(
  (config) => {
    console.log(`API Request: ${config.method?.toUpperCase()} ${config.url}`);
    return config;
  },
  (error) => {
    console.error('API Request Error:', error);
    return Promise.reject(error);
  }
);

// Response interceptor for error handling
api.interceptors.response.use(
  (response) => response,
  (error) => {
    console.error('API Response Error:', error);
    if (error.response?.data) {
      const apiError: APIError = error.response.data;
      throw new Error(apiError.message || 'API request failed');
    }
    throw new Error(error.message || 'Network error');
  }
);

// Blocks API
export const blocksApi = {
  getList: async (page = 1, pageSize = 20): Promise<PaginatedResponse<BlockSummary>> => {
    const response: AxiosResponse<PaginatedResponse<BlockSummary>> = await api.get(
      `/api/v1/blocks?page=${page}&page_size=${pageSize}`
    );
    return response.data;
  },

  getByHash: async (hash: string): Promise<BlockDetails | null> => {
    try {
      const response: AxiosResponse<BlockDetails> = await api.get(`/api/v1/blocks/${hash}`);
      return response.data;
    } catch (error) {
      if (axios.isAxiosError(error) && error.response?.status === 404) {
        return null;
      }
      throw error;
    }
  },

  getByHeight: async (height: number): Promise<BlockDetails | null> => {
    try {
      const response: AxiosResponse<BlockDetails> = await api.get(`/api/v1/blocks/height/${height}`);
      return response.data;
    } catch (error) {
      if (axios.isAxiosError(error) && error.response?.status === 404) {
        return null;
      }
      throw error;
    }
  },

  getRecent: async (limit = 10): Promise<PaginatedResponse<BlockSummary>> => {
    const response: AxiosResponse<PaginatedResponse<BlockSummary>> = await api.get(
      `/api/v1/blocks/recent?page_size=${limit}`
    );
    return response.data;
  },
};

// Transactions API
export const transactionsApi = {
  getList: async (page = 1, pageSize = 20): Promise<PaginatedResponse<TransactionSummary>> => {
    const response: AxiosResponse<PaginatedResponse<TransactionSummary>> = await api.get(
      `/api/v1/transactions?page=${page}&page_size=${pageSize}`
    );
    return response.data;
  },

  getByHash: async (hash: string): Promise<TransactionDetails | null> => {
    try {
      const response: AxiosResponse<TransactionDetails> = await api.get(`/api/v1/transactions/${hash}`);
      return response.data;
    } catch (error) {
      if (axios.isAxiosError(error) && error.response?.status === 404) {
        return null;
      }
      throw error;
    }
  },

  getPending: async (): Promise<TransactionSummary[]> => {
    const response: AxiosResponse<TransactionSummary[]> = await api.get('/api/v1/transactions/pending');
    return response.data;
  },
};

// Addresses API
export const addressesApi = {
  getSummary: async (address: string): Promise<AddressSummary | null> => {
    try {
      const response: AxiosResponse<AddressSummary> = await api.get(`/api/v1/addresses/${address}`);
      return response.data;
    } catch (error) {
      if (axios.isAxiosError(error) && error.response?.status === 404) {
        return null;
      }
      throw error;
    }
  },

  getTransactions: async (
    address: string,
    page = 1,
    pageSize = 20
  ): Promise<PaginatedResponse<AddressTransaction>> => {
    const response: AxiosResponse<PaginatedResponse<AddressTransaction>> = await api.get(
      `/api/v1/addresses/${address}/transactions?page=${page}&page_size=${pageSize}`
    );
    return response.data;
  },

  getUTXOs: async (address: string): Promise<AddressUTXO[]> => {
    const response: AxiosResponse<AddressUTXO[]> = await api.get(`/api/v1/addresses/${address}/utxos`);
    return response.data;
  },
};

// Stats API
export const statsApi = {
  getNetworkStats: async (): Promise<NetworkStats> => {
    const response: AxiosResponse<NetworkStats> = await api.get('/api/v1/stats/network');
    return response.data;
  },

  getMiningInfo: async (): Promise<MiningInfo> => {
    const response: AxiosResponse<MiningInfo> = await api.get('/api/v1/stats/mining');
    return response.data;
  },

  getBlockDagInfo: async (): Promise<BlockDagInfo> => {
    const response: AxiosResponse<BlockDagInfo> = await api.get('/api/v1/stats/blockdag');
    return response.data;
  },
};

// Search API
export const searchApi = {
  search: async (query: string): Promise<SearchResults> => {
    const response: AxiosResponse<SearchResults> = await api.get(`/api/v1/search?q=${encodeURIComponent(query)}`);
    return response.data;
  },
};

// Utility functions
export const formatHash = (hash: string, length = 8): string => {
  if (hash.length <= length * 2) return hash;
  return `${hash.slice(0, length)}...${hash.slice(-length)}`;
};

export const formatAddress = (address: string): string => {
  return formatHash(address, 12);
};

export const formatTimestamp = (timestamp: number): string => {
  return new Date(timestamp * 1000).toLocaleString();
};

export const formatNumber = (num: number): string => {
  return new Intl.NumberFormat().format(num);
};

export const formatBytes = (bytes: number): string => {
  const units = ['B', 'KB', 'MB', 'GB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
};

export const formatDuration = (seconds: number): string => {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  if (hours > 0) {
    return `${hours}h ${minutes}m ${secs}s`;
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`;
  } else {
    return `${secs}s`;
  }
};
