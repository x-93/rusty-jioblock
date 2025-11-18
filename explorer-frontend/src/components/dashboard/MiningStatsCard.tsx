import { MiningInfo } from '@/types/api';
import { formatNumber } from '@/utils/api';
import { Pickaxe, Zap, AlertTriangle, Activity } from 'lucide-react';

interface MiningStatsCardProps {
  miningInfo: MiningInfo;
}

export default function MiningStatsCard({ miningInfo }: MiningStatsCardProps) {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
      <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
        <Pickaxe className="w-5 h-5 mr-2 text-orange-600 dark:text-orange-400" />
        Mining Statistics
      </h3>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Network Hashrate */}
        <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <div className="flex items-center">
            <Zap className="w-5 h-5 text-yellow-600 dark:text-yellow-400 mr-3" />
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Network Hashrate</p>
              <p className="text-lg font-bold text-gray-900 dark:text-white">
                {formatNumber(miningInfo.network_hash_ps)} H/s
              </p>
            </div>
          </div>
        </div>

        {/* Difficulty */}
        <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <div className="flex items-center">
            <Activity className="w-5 h-5 text-blue-600 dark:text-blue-400 mr-3" />
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Difficulty</p>
              <p className="text-lg font-bold text-gray-900 dark:text-white">
                {formatNumber(miningInfo.difficulty)}
              </p>
            </div>
          </div>
        </div>

        {/* Blocks */}
        <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <div className="flex items-center">
            <Pickaxe className="w-5 h-5 text-green-600 dark:text-green-400 mr-3" />
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Blocks</p>
              <p className="text-lg font-bold text-gray-900 dark:text-white">
                {formatNumber(miningInfo.blocks)}
              </p>
            </div>
          </div>
        </div>

        {/* Pooled Transactions */}
        <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <div className="flex items-center">
            <Activity className="w-5 h-5 text-purple-600 dark:text-purple-400 mr-3" />
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Pooled TX</p>
              <p className="text-lg font-bold text-gray-900 dark:text-white">
                {formatNumber(miningInfo.pooled_tx)}
              </p>
            </div>
          </div>
        </div>

        {/* Network */}
        <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg md:col-span-2">
          <div className="flex items-center">
            <Activity className="w-5 h-5 text-indigo-600 dark:text-indigo-400 mr-3" />
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Network</p>
              <p className="text-lg font-bold text-gray-900 dark:text-white">
                {miningInfo.chain}
              </p>
            </div>
          </div>
        </div>

        {/* Warnings */}
        {miningInfo.warnings && miningInfo.warnings !== '' && (
          <div className="flex items-center justify-between p-3 bg-yellow-50 dark:bg-yellow-900 rounded-lg md:col-span-2">
            <div className="flex items-center">
              <AlertTriangle className="w-5 h-5 text-yellow-600 dark:text-yellow-400 mr-3" />
              <div>
                <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Warnings</p>
                <p className="text-sm text-gray-900 dark:text-white">
                  {miningInfo.warnings}
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Errors */}
        {miningInfo.errors && miningInfo.errors !== '' && (
          <div className="flex items-center justify-between p-3 bg-red-50 dark:bg-red-900 rounded-lg md:col-span-2">
            <div className="flex items-center">
              <AlertTriangle className="w-5 h-5 text-red-600 dark:text-red-400 mr-3" />
              <div>
                <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Errors</p>
                <p className="text-sm text-gray-900 dark:text-white">
                  {miningInfo.errors}
                </p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
