'use client'

import { useState, useEffect } from 'react'
import { Navbar } from '@/components/navbar/Navbar'
import { AssetCard } from '@/components/cards/AssetCard'
import { motion } from 'framer-motion'
import { Search } from 'lucide-react'
import { api } from '@/lib/api'

export default function AssetsPage() {
  const [assets, setAssets] = useState<any[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedType, setSelectedType] = useState('All')
  const [searchQuery, setSearchQuery] = useState('')

  useEffect(() => {
    const fetchAssets = async () => {
      try {
        const response = await api.assets.getAll()
        setAssets(response.data)
      } catch (error) {
        console.error('Failed to fetch assets:', error)
      } finally {
        setLoading(false)
      }
    }
    fetchAssets()
  }, [])

  const assetTypes = ['All', 'jet', 'yacht', 'real-estate', 'car']
  const typeLabels: Record<string, string> = {
    'All': 'All',
    'jet': 'Private Jet',
    'yacht': 'Super Yacht',
    'real-estate': 'Real Estate',
    'car': 'Luxury Car'
  }

  const filteredAssets = assets.filter(asset => {
    const matchesType = selectedType === 'All' || asset.type === selectedType
    const matchesSearch = asset.name.toLowerCase().includes(searchQuery.toLowerCase())
    return matchesType && matchesSearch
  })

  return (
    <div className="min-h-screen">
      <Navbar />
      
      <main className="pt-32 pb-20 px-4">
        <div className="container mx-auto">
          {/* Header */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="mb-12"
          >
            <h1 className="font-display text-5xl md:text-6xl font-bold mb-4">
              Luxury Asset <span className="text-white">Marketplace</span>
            </h1>
            <p className="text-xl text-muted max-w-2xl">
              Purchase luxury assets with cryptocurrency - BTC, ETH, USDT, USDC, SOL, BNB
            </p>
          </motion.div>

          {/* Filters */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
            className="mb-8 space-y-4"
          >
            {/* Search */}
            <div className="relative max-w-md">
              <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-muted" />
              <input
                type="text"
                placeholder="Search assets..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full glass-card pl-12 pr-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-[rgb(var(--accent))]"
              />
            </div>

            {/* Type Filters */}
            <div className="flex flex-wrap gap-3">
              {assetTypes.map((type) => (
                <button
                  key={type}
                  onClick={() => setSelectedType(type)}
                  className={`px-6 py-2 rounded-lg transition-all ${
                    selectedType === type
                      ? 'bg-[rgb(var(--accent))] text-white font-semibold'
                      : 'glass-card hover:bg-white/10'
                  }`}
                >
                  {typeLabels[type]}
                </button>
              ))}
            </div>
          </motion.div>

          {/* Results Count */}
          <div className="mb-6 text-muted">
            {loading ? 'Loading...' : `Showing ${filteredAssets.length} assets`}
          </div>

          {/* Asset Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            {filteredAssets.map((asset, index) => (
              <motion.div
                key={asset.id}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: index * 0.1 }}
              >
                <AssetCard asset={asset} />
              </motion.div>
            ))}
          </div>

          {/* Empty State */}
          {filteredAssets.length === 0 && (
            <div className="text-center py-20">
              <p className="text-xl text-muted">No assets found matching your criteria</p>
            </div>
          )}
        </div>
      </main>
    </div>
  )
}


