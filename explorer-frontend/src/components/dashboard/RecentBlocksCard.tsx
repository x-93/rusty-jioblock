import { BlockSummary } from '@/types/api';
import { formatHash, formatTimestamp } from '@/utils/api';
import { Blocks, Clock, Hash } from 'lucide-react';

interface RecentBlocksCardProps {
  blocks: BlockSummary[];
}

export default function RecentBlocksCard({ blocks }: RecentBlocksCardProps) {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
      <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
        <Blocks className="w-5 h-5 mr-2 text-blue-600 dark:text-blue-400" />
        Recent Blocks
      </h3>
      <div className="space-y-4">
        {blocks.map((block) => (
          <div key={block.hash} className="border-b border-gray-200 dark:border-gray-700 pb-4 last:border-b-0 last:pb-0">
            <div className="flex justify-between items-start mb-2">
              <div className="flex items-center">
                <Hash className="w-4 h-4 text-gray-500 mr-2" />
                <a
                  href={`/blocks/${block.hash}`}
                  className="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 font-mono text-sm"
                >
                  {formatHash(block.hash)}
                </a>
              </div>
              <span className="text-sm text-gray-600 dark:text-gray-400">
                #{block.height}
              </span>
            </div>
            <div className="flex justify-between text-sm text-gray-600 dark:text-gray-400">
              <span>{block.tx_count} transactions</span>
              <span className="flex items-center">
                <Clock className="w-3 h-3 mr-1" />
                {formatTimestamp(block.timestamp)}
              </span>
            </div>
          </div>
        ))}
      </div>
      <div className="mt-4">
        <a
          href="/blocks"
          className="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 text-sm font-medium"
        >
          View all blocks →
        </a>
      </div>
    </div>
  );
}
