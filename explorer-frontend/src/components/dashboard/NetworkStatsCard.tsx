import { NetworkStats } from '@/types/api';
import { formatNumber, formatBytes } from '@/utils/api';
import { Activity, Blocks, Users, Zap, TrendingUp, Clock } from 'lucide-react';

interface NetworkStatsCardProps {
  stats: NetworkStats;
}

export default function NetworkStatsCard({ stats }: NetworkStatsCardProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
      {/* Block Count */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex items-center">
          <div className="p-2 bg-blue-100 dark:bg-blue-900 rounded-lg">
            <Blocks className="w-6 h-6 text-blue-600 dark:text-blue-400" />
          </div>
          <div className="ml-4">
            <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Blocks</p>
            <p className="text-2xl font-bold text-gray-900 dark:text-white">
              {formatNumber(stats.block_count)}
            </p>
          </div>
        </div>
      </div>

      {/* Transaction Count */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex items-center">
          <div className="p-2 bg-green-100 dark:bg-green-900 rounded-lg">
            <Activity className="w-6 h-6 text-green-600 dark:text-green-400" />
          </div>
          <div className="ml-4">
            <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Transactions</p>
            <p className="text-2xl font-bold text-gray-900 dark:text-white">
              {formatNumber(stats.tx_count)}
            </p>
          </div>
        </div>
      </div>

      {/* Address Count */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex items-center">
          <div className="p-2 bg-purple-100 dark:bg-purple-900 rounded-lg">
            <Users className="w-6 h-6 text-purple-600 dark:text-purple-400" />
          </div>
          <div className="ml-4">
            <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Addresses</p>
            <p className="text-2xl font-bold text-gray-900 dark:text-white">
              {formatNumber(stats.address_count)}
            </p>
          </div>
        </div>
      </div>

      {/* Total Supply */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex items-center">
          <div className="p-2 bg-yellow-100 dark:bg-yellow-900 rounded-lg">
            <Zap className="w-6 h-6 text-yellow-600 dark:text-yellow-400" />
          </div>
          <div className="ml-4">
            <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Total Supply</p>
            <p className="text-2xl font-bold text-gray-900 dark:text-white">
              {formatNumber(stats.total_supply)}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
