'use client'

import { motion } from 'framer-motion'
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
      initial={{ opacity: 0, y: 16 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      whileHover={{ y: -2 }}
      transition={{ duration: 0.25 }}
      className="interactive-card overflow-hidden group cursor-pointer"
    >
      <div className="relative h-60 overflow-hidden border-b border-[rgba(var(--frame-strong),0.7)]">
        <Image
          src={asset.image || '/placeholder-jet.jpg'}
          alt={asset.name}
          fill
          className="object-cover transition-transform duration-500 group-hover:scale-[1.02]"
        />
        <div className="absolute inset-0 bg-[linear-gradient(180deg,transparent,rgba(9,10,13,0.78))]" />
        <div className="absolute left-4 top-4">
          <span className="status-chip">{asset.type}</span>
        </div>
        <div className="absolute bottom-4 left-4 right-4 flex items-end justify-between gap-4">
          <div>
            <div className="text-xs uppercase tracking-[0.18em] text-white/60">Location</div>
            <div className="mt-1 text-sm text-white">{asset.location}</div>
          </div>
        </div>
      </div>

      <div className="p-6">
        <h3 className="text-2xl font-semibold tracking-[-0.03em]">{asset.name}</h3>
        <p className="mt-2 text-sm leading-6 text-muted">
          Secure purchase flow for premium inventory through AeroLedger.
        </p>

        <div className="mt-6 space-y-3 text-sm">
          <div className="flex items-center justify-between border-b border-[rgba(var(--frame-strong),0.5)] pb-3">
            <span className="text-muted">Price</span>
            <span className="font-medium">${asset.totalValue.toLocaleString()}</span>
          </div>
          <div className="flex items-center justify-between border-b border-[rgba(var(--frame-strong),0.5)] pb-3">
            <span className="text-muted">Payment</span>
            <span className="font-medium text-white">BTC, ETH, USDT</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-muted">Status</span>
            <span className="font-medium text-[#7be0c7]">Available</span>
          </div>
        </div>

        <Link href={`/assets/${asset.id}`} className="mt-6 block">
          <Button className="w-full">View Details</Button>
        </Link>
      </div>
    </motion.div>
  )
}
