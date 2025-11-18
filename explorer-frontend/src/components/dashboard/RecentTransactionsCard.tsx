import { TransactionSummary } from '@/types/api';
import { formatHash, formatTimestamp, formatNumber } from '@/utils/api';
import { Activity, Clock, Hash, ArrowUpRight, ArrowDownLeft } from 'lucide-react';

interface RecentTransactionsCardProps {
  transactions: TransactionSummary[];
}

export default function RecentTransactionsCard({ transactions }: RecentTransactionsCardProps) {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
      <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
        <Activity className="w-5 h-5 mr-2 text-green-600 dark:text-green-400" />
        Recent Transactions
      </h3>
      <div className="space-y-4">
        {transactions.map((tx) => (
          <div key={tx.hash} className="border-b border-gray-200 dark:border-gray-700 pb-4 last:border-b-0 last:pb-0">
            <div className="flex justify-between items-start mb-2">
              <div className="flex items-center">
                <Hash className="w-4 h-4 text-gray-500 mr-2" />
                <a
                  href={`/transactions/${tx.hash}`}
                  className="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 font-mono text-sm"
                >
                  {formatHash(tx.hash)}
                </a>
              </div>
              <div className="flex items-center text-sm">
                {tx.is_confirmed ? (
                  <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
                    Confirmed
                  </span>
                ) : (
                  <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
                    Pending
                  </span>
                )}
              </div>
            </div>
            <div className="flex justify-between items-center text-sm text-gray-600 dark:text-gray-400">
              <div className="flex items-center space-x-4">
                <span className="flex items-center">
                  <ArrowUpRight className="w-3 h-3 mr-1 text-red-500" />
                  {tx.input_count} inputs
                </span>
                <span className="flex items-center">
                  <ArrowDownLeft className="w-3 h-3 mr-1 text-green-500" />
                  {tx.output_count} outputs
                </span>
              </div>
              <span className="flex items-center">
                <Clock className="w-3 h-3 mr-1" />
                {formatTimestamp(tx.timestamp)}
              </span>
            </div>
            <div className="mt-2 text-sm">
              <span className="font-medium text-gray-900 dark:text-white">
                {formatNumber(tx.value)} JIO
              </span>
            </div>
          </div>
        ))}
      </div>
      <div className="mt-4">
        <a
          href="/transactions"
          className="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 text-sm font-medium"
        >
          View all transactions →
        </a>
      </div>
    </div>
  );
}
