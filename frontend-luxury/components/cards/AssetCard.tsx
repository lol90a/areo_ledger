'use client'

import { motion } from 'framer-motion'
import { formatCurrency, formatNumber } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import Link from 'next/link'
import Image from 'next/image'

interface AssetCardProps {
  asset: {
    id: string
    name: string
    type: string
    image: string
    totalValue: number
    tokenPrice: number
    availableTokens: number
    roi: number
    location: string
  }
}

export function AssetCard({ asset }: AssetCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      whileHover={{ y: -10 }}
      transition={{ duration: 0.3 }}
      className="glass-card overflow-hidden group cursor-pointer"
    >
      {/* Image */}
      <div className="relative h-64 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-t from-[#0A0A0A] to-transparent z-10" />
        <Image
          src={asset.image || '/placeholder-jet.jpg'}
          alt={asset.name}
          fill
          className="object-cover group-hover:scale-110 transition-transform duration-500"
        />
        <div className="absolute top-4 right-4 z-20">
          <span className="glass-card px-3 py-1 text-xs font-semibold text-gold">
            {asset.type}
          </span>
        </div>
      </div>

      {/* Content */}
      <div className="p-6">
        <div className="flex items-start justify-between mb-4">
          <div>
            <h3 className="font-display text-2xl font-bold mb-1 group-hover:text-gold transition-colors">
              {asset.name}
            </h3>
            <p className="text-sm text-muted">{asset.location}</p>
          </div>
        </div>

        <div className="space-y-3 mb-6">
          <div className="flex justify-between items-center">
            <span className="text-sm text-muted">Price</span>
            <span className="font-semibold text-gold">{formatCurrency(asset.totalValue)}</span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-sm text-muted">Payment</span>
            <span className="font-semibold">BTC, ETH, USDT</span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-sm text-muted">Status</span>
            <span className="font-semibold text-green-500">Available</span>
          </div>
        </div>

        <Link href={`/assets/${asset.id}`}>
          <Button className="w-full">View Details</Button>
        </Link>
      </div>
    </motion.div>
  )
}
