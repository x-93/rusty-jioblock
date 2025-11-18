'use client';

import { useEffect, useState } from 'react';
import Layout from '@/components/layout/Layout';
import NetworkStatsCard from '@/components/dashboard/NetworkStatsCard';
import MiningStatsCard from '@/components/dashboard/MiningStatsCard';
import BlockDagInfoCard from '@/components/dashboard/BlockDagInfoCard';
import RecentBlocksCard from '@/components/dashboard/RecentBlocksCard';
import RecentTransactionsCard from '@/components/dashboard/RecentTransactionsCard';
import { NetworkStats, MiningInfo, BlockDagInfo, BlockSummary, TransactionSummary } from '@/types/api';
import { statsApi, blocksApi, transactionsApi } from '@/utils/api';
import { formatBytes, formatNumber } from '@/utils/api';
import { TrendingUp, Clock, Coins } from 'lucide-react';

export default function Dashboard() {
  const [stats, setStats] = useState<NetworkStats | null>(null);
  const [miningInfo, setMiningInfo] = useState<MiningInfo | null>(null);
  const [blockDagInfo, setBlockDagInfo] = useState<BlockDagInfo | null>(null);
  const [recentBlocks, setRecentBlocks] = useState<BlockSummary[]>([]);
  const [recentTransactions, setRecentTransactions] = useState<TransactionSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [
          networkStats,
          miningStats,
          blockDagStats,
          blocksResponse,
          transactionsResponse
        ] = await Promise.all([
          statsApi.getNetworkStats(),
          statsApi.getMiningInfo().catch(() => null), // Optional, don't fail if not available
          statsApi.getBlockDagInfo().catch(() => null), // Optional, don't fail if not available
          blocksApi.getRecent(5),
          transactionsApi.getPending(),
        ]);
        setStats(networkStats);
        setMiningInfo(miningStats);
        setBlockDagInfo(blockDagStats);
        setRecentBlocks(blocksResponse.data.slice(0, 5));
        setRecentTransactions(transactionsResponse.slice(0, 5));
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch dashboard data');
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);

  if (loading) {
    return (
      <Layout>
        <div className="flex items-center justify-center min-h-screen">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-500"></div>
        </div>
      </Layout>
    );
  }

  if (error) {
    return (
      <Layout>
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-center">
            <div className="text-red-500 text-lg font-semibold mb-2">Error</div>
            <div className="text-gray-600 dark:text-gray-400">{error}</div>
          </div>
        </div>
      </Layout>
    );
  }

  return (
    <Layout>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            JIO Blockchain Dashboard
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Real-time overview of the JIO blockchain network
          </p>
        </div>

        {/* Network Stats */}
        {stats && <NetworkStatsCard stats={stats} />}

        {/* Mining Stats */}
        {miningInfo && <MiningStatsCard miningInfo={miningInfo} />}

        {/* Block DAG Info */}
        {blockDagInfo && <BlockDagInfoCard blockDagInfo={blockDagInfo} />}

        {/* Recent Activity */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
          {recentBlocks.length > 0 && <RecentBlocksCard blocks={recentBlocks} />}
          {recentTransactions.length > 0 && <RecentTransactionsCard transactions={recentTransactions} />}
        </div>

        {/* Additional Stats */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Total Coins Mined */}
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
              <Coins className="w-5 h-5 mr-2 text-yellow-600 dark:text-yellow-400" />
              Total Coins Mined
            </h3>
            <div className="text-3xl font-bold text-gray-900 dark:text-white">
              {stats ? formatNumber(stats.total_supply) : '0'} JIO
            </div>
            <p className="text-sm text-gray-600 dark:text-gray-400 mt-2">
              Circulating supply
            </p>
          </div>

          {/* Network Info */}
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Network Information
            </h3>
            <div className="space-y-3">
              {stats && (
                <>
                  <div className="flex justify-between">
                    <span className="text-gray-600 dark:text-gray-400">Mempool Size</span>
                    <span className="font-medium text-gray-900 dark:text-white">
                      {formatBytes(stats.mempool_bytes)}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600 dark:text-gray-400">Mempool Transactions</span>
                    <span className="font-medium text-gray-900 dark:text-white">
                      {stats.mempool_size}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600 dark:text-gray-400">Peer Count</span>
                    <span className="font-medium text-gray-900 dark:text-white">
                      {stats.peer_count}
                    </span>
                  </div>
                  {stats.hashrate && (
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-400">Hashrate</span>
                      <span className="font-medium text-gray-900 dark:text-white">
                        {stats.hashrate} H/s
                      </span>
                    </div>
                  )}
                  {stats.difficulty && (
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-400">Difficulty</span>
                      <span className="font-medium text-gray-900 dark:text-white">
                        {stats.difficulty}
                      </span>
                    </div>
                  )}
                </>
              )}
            </div>
          </div>

          {/* Performance Metrics */}
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Performance Metrics
            </h3>
            <div className="space-y-3">
              {stats && (
                <>
                  {stats.avg_block_time && (
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-400">Avg Block Time</span>
                      <span className="font-medium text-gray-900 dark:text-white">
                        {Math.round(stats.avg_block_time)}s
                      </span>
                    </div>
                  )}
                  <div className="flex justify-between">
                    <span className="text-gray-600 dark:text-gray-400">Last Updated</span>
                    <span className="font-medium text-gray-900 dark:text-white">
                      <Clock className="w-4 h-4 inline mr-1" />
                      {new Date(stats.timestamp * 1000).toLocaleTimeString()}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600 dark:text-gray-400">Status</span>
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
                      <TrendingUp className="w-3 h-3 mr-1" />
                      Active
                    </span>
                  </div>
                </>
              )}
            </div>
          </div>
        </div>

        {/* Quick Actions */}
        <div className="mt-8">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Quick Actions
          </h3>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <a
              href="/blocks"
              className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4 hover:shadow-md transition-shadow"
            >
              <div className="flex items-center">
                <div className="w-5 h-5 bg-blue-100 dark:bg-blue-900 rounded mr-3 flex items-center justify-center">
                  <span className="text-blue-600 dark:text-blue-400 text-xs font-bold">B</span>
                </div>
                <span className="font-medium text-gray-900 dark:text-white">Browse Blocks</span>
              </div>
            </a>
            <a
              href="/transactions"
              className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4 hover:shadow-md transition-shadow"
            >
              <div className="flex items-center">
                <div className="w-5 h-5 bg-green-100 dark:bg-green-900 rounded mr-3 flex items-center justify-center">
                  <span className="text-green-600 dark:text-green-400 text-xs font-bold">T</span>
                </div>
                <span className="font-medium text-gray-900 dark:text-white">View Transactions</span>
              </div>
            </a>
            <a
              href="/stats"
              className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4 hover:shadow-md transition-shadow"
            >
              <div className="flex items-center">
                <TrendingUp className="w-5 h-5 text-purple-600 dark:text-purple-400 mr-3" />
                <span className="font-medium text-gray-900 dark:text-white">Network Stats</span>
              </div>
            </a>
            <a
              href="/search"
              className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4 hover:shadow-md transition-shadow"
            >
              <div className="flex items-center">
                <div className="w-5 h-5 bg-gray-300 dark:bg-gray-600 rounded mr-3 flex items-center justify-center">
                  <span className="text-gray-700 dark:text-gray-300 text-xs font-bold">S</span>
                </div>
                <span className="font-medium text-gray-900 dark:text-white">Search</span>
              </div>
            </a>
          </div>
        </div>
      </div>
    </Layout>
  );
}
