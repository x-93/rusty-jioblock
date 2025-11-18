import { BlockDagInfo } from '@/types/api';
import { formatHash, formatNumber } from '@/utils/api';
import { Network, GitBranch, Hash, MapPin } from 'lucide-react';

interface BlockDagInfoCardProps {
  blockDagInfo: BlockDagInfo;
}

export default function BlockDagInfoCard({ blockDagInfo }: BlockDagInfoCardProps) {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
      <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
        <Network className="w-5 h-5 mr-2 text-teal-600 dark:text-teal-400" />
        Block DAG Network
      </h3>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Block Count */}
        <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <div className="flex items-center">
            <GitBranch className="w-5 h-5 text-blue-600 dark:text-blue-400 mr-3" />
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Block Count</p>
              <p className="text-lg font-bold text-gray-900 dark:text-white">
                {formatNumber(blockDagInfo.block_count)}
              </p>
            </div>
          </div>
        </div>

        {/* Difficulty */}
        <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <div className="flex items-center">
            <Network className="w-5 h-5 text-orange-600 dark:text-orange-400 mr-3" />
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Difficulty</p>
              <p className="text-lg font-bold text-gray-900 dark:text-white">
                {formatNumber(blockDagInfo.difficulty)}
              </p>
            </div>
          </div>
        </div>

        {/* Network */}
        <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <div className="flex items-center">
            <MapPin className="w-5 h-5 text-green-600 dark:text-green-400 mr-3" />
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Network</p>
              <p className="text-lg font-bold text-gray-900 dark:text-white">
                {blockDagInfo.network}
              </p>
            </div>
          </div>
        </div>

        {/* Tip Hashes Count */}
        <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <div className="flex items-center">
            <Hash className="w-5 h-5 text-purple-600 dark:text-purple-400 mr-3" />
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Tip Hashes</p>
              <p className="text-lg font-bold text-gray-900 dark:text-white">
                {blockDagInfo?.tip_hashes?.length || 0}
              </p>
            </div>
          </div>
        </div>

        {/* Virtual Parent Hashes Count */}
        <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
          <div className="flex items-center">
            <GitBranch className="w-5 h-5 text-indigo-600 dark:text-indigo-400 mr-3" />
            <div>
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Virtual Parents</p>
              <p className="text-lg font-bold text-gray-900 dark:text-white">
                {blockDagInfo?.virtual_parent_hashes?.length || 0}
              </p>
            </div>
          </div>
        </div>

        {/* Pruning Point Hash */}
        <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg md:col-span-2">
          <div className="flex items-center">
            <Hash className="w-5 h-5 text-red-600 dark:text-red-400 mr-3" />
            <div className="flex-1">
              <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Pruning Point</p>
              <p className="text-sm font-mono text-gray-900 dark:text-white break-all">
                {formatHash(blockDagInfo.pruning_point_hash, 16)}
              </p>
            </div>
          </div>
        </div>

        {/* Tip Hashes List */}
        {blockDagInfo.tip_hashes.length > 0 && (
          <div className="md:col-span-2">
            <h4 className="text-sm font-medium text-gray-600 dark:text-gray-400 mb-2">Tip Hashes</h4>
            <div className="space-y-1 max-h-32 overflow-y-auto">
              {blockDagInfo.tip_hashes.slice(0, 5).map((hash, index) => (
                <div key={index} className="text-xs font-mono text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-600 px-2 py-1 rounded">
                  {formatHash(hash, 12)}
                </div>
              ))}
              {blockDagInfo.tip_hashes.length > 5 && (
                <div className="text-xs text-gray-500 dark:text-gray-400">
                  ... and {blockDagInfo.tip_hashes.length - 5} more
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
