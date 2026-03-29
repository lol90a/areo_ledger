'use client'

import { useEffect, useState } from 'react'
import { motion } from 'framer-motion'
import { formatCurrency } from '@/lib/utils'
import { ArrowUpRight, ArrowDownLeft, ArrowRightLeft, Clock, CheckCircle, Loader2 } from 'lucide-react'
import { api, session } from '@/lib/api'

export default function TransactionsPage() {
  const [transactions, setTransactions] = useState<any[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const fetchTransactions = async () => {
      try {
        const userId = session.getUserId()
        if (!userId) {
          setTransactions([])
          return
        }

        const response = await api.transactions.getUserTransactions(userId)
        setTransactions(response.data)
      } catch (error) {
        console.error('Failed to fetch transactions:', error)
      } finally {
        setLoading(false)
      }
    }

    fetchTransactions()
  }, [])

  if (loading) {
    return (
      <div className="p-8 flex items-center justify-center min-h-screen">
        <Loader2 className="h-12 w-12 animate-spin text-white" />
      </div>
    )
  }

  return (
    <div className="p-8">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="mb-8"
      >
        <h1 className="font-display text-4xl font-bold mb-2">Transaction History</h1>
        <p className="text-muted">View all your investment transactions</p>
      </motion.div>

      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="flex gap-4 mb-8"
      >
        <select className="glass-card px-4 py-2 rounded-lg focus:outline-none focus:ring-2 focus:ring-[rgb(var(--accent))]">
          <option>All Types</option>
          <option>Buy</option>
          <option>Sell</option>
          <option>Transfer</option>
        </select>
        <select className="glass-card px-4 py-2 rounded-lg focus:outline-none focus:ring-2 focus:ring-[rgb(var(--accent))]">
          <option>All Status</option>
          <option>Confirmed</option>
          <option>Pending</option>
        </select>
      </motion.div>

      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="glass-card overflow-hidden"
      >
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-[rgba(var(--accent),0.30)]">
                <th className="text-left p-4 text-sm font-semibold text-muted">Transaction ID</th>
                <th className="text-left p-4 text-sm font-semibold text-muted">Type</th>
                <th className="text-left p-4 text-sm font-semibold text-muted">Asset</th>
                <th className="text-right p-4 text-sm font-semibold text-muted">Amount</th>
                <th className="text-center p-4 text-sm font-semibold text-muted">Status</th>
                <th className="text-left p-4 text-sm font-semibold text-muted">Date</th>
              </tr>
            </thead>
            <tbody>
              {transactions.length === 0 ? (
                <tr>
                  <td colSpan={6} className="p-8 text-center text-muted">
                    No transactions yet. Complete a booking payment to see your history.
                  </td>
                </tr>
              ) : (
                transactions.map((tx, index) => (
                  <motion.tr
                    key={tx.id}
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: 0.3 + index * 0.1 }}
                    className="border-b border-[rgba(var(--accent),0.30)] hover:bg-white/5 transition-colors"
                  >
                    <td className="p-4">
                      <div className="font-mono text-sm">{tx.id}</div>
                      <div className="text-xs text-muted">{tx.tx_hash || 'No hash available'}</div>
                    </td>
                    <td className="p-4">
                      <div className="flex items-center">
                        {tx.type === 'buy' && (
                          <div className="flex items-center text-green-500">
                            <ArrowDownLeft className="h-4 w-4 mr-2" />
                            <span className="capitalize">{tx.type}</span>
                          </div>
                        )}
                        {tx.type === 'sell' && (
                          <div className="flex items-center text-red-500">
                            <ArrowUpRight className="h-4 w-4 mr-2" />
                            <span className="capitalize">{tx.type}</span>
                          </div>
                        )}
                        {tx.type === 'transfer' && (
                          <div className="flex items-center text-white">
                            <ArrowRightLeft className="h-4 w-4 mr-2" />
                            <span className="capitalize">{tx.type}</span>
                          </div>
                        )}
                      </div>
                    </td>
                    <td className="p-4 font-semibold">{tx.asset}</td>
                    <td className="p-4 text-right font-semibold text-white">
                      {formatCurrency(tx.amount)}
                    </td>
                    <td className="p-4 text-center">
                      {tx.status === 'confirmed' ? (
                        <span className="px-3 py-1 bg-green-500/20 text-green-500 rounded-full text-xs font-semibold inline-flex items-center gap-1">
                          <CheckCircle className="h-3 w-3" />
                          Confirmed
                        </span>
                      ) : (
                        <span className="px-3 py-1 bg-blue-500/20 text-blue-400 rounded-full text-xs font-semibold inline-flex items-center gap-1">
                          <Clock className="h-3 w-3" />
                          Pending
                        </span>
                      )}
                    </td>
                    <td className="p-4 text-sm text-muted">{new Date(tx.date).toLocaleString()}</td>
                  </motion.tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </motion.div>
    </div>
  )
}



