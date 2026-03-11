'use client'

import { Navbar } from '@/components/navbar/Navbar'
import { Button } from '@/components/ui/button'
import { motion } from 'framer-motion'
import { ArrowLeft, Check, Loader2 } from 'lucide-react'
import Link from 'next/link'
import Image from 'next/image'
import { useCart } from '@/context/CartContext'
import { useState, useEffect } from 'react'
import { api } from '@/lib/api'

const cryptoPrices = {
  BTC: { price: 45000, symbol: '₿', name: 'Bitcoin' },
  ETH: { price: 2500, symbol: 'Ξ', name: 'Ethereum' },
  USDT: { price: 1, symbol: '₮', name: 'Tether' },
  USDC: { price: 1, symbol: '$', name: 'USD Coin' },
  SOL: { price: 100, symbol: '◎', name: 'Solana' },
  BNB: { price: 320, symbol: 'BNB', name: 'Binance Coin' },
}

export default function ProductDetailPage({ params }: { params: { id: string } }) {
  const { addToCart } = useCart()
  const [added, setAdded] = useState(false)
  const [item, setItem] = useState<any>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const fetchProduct = async () => {
      try {
        const response = await api.products.getById(parseInt(params.id))
        setItem(response.data)
      } catch (error) {
        console.error('Failed to fetch product:', error)
      } finally {
        setLoading(false)
      }
    }
    fetchProduct()
  }, [params.id])

  if (loading) {
    return (
      <div className="min-h-screen">
        <Navbar />
        <main className="pt-32 pb-20 px-4">
          <div className="container mx-auto max-w-7xl flex justify-center items-center h-96">
            <Loader2 className="h-12 w-12 animate-spin text-gold" />
          </div>
        </main>
      </div>
    )
  }

  if (!item) {
    return (
      <div className="min-h-screen">
        <Navbar />
        <main className="pt-32 pb-20 px-4">
          <div className="container mx-auto max-w-7xl">
            <Link href="/marketplace">
              <Button variant="ghost" className="mb-8">
                <ArrowLeft className="mr-2 h-4 w-4" />
                Back to Marketplace
              </Button>
            </Link>
            <div className="text-center py-20">
              <h1 className="text-3xl font-bold mb-4">Product not found</h1>
              <p className="text-muted">The product you're looking for doesn't exist.</p>
            </div>
          </div>
        </main>
      </div>
    )
  }

  const handleAddToCart = () => {
    addToCart({
      id: item.id,
      name: item.name,
      category: item.category,
      price: item.price,
      image: item.image
    })
    setAdded(true)
    setTimeout(() => setAdded(false), 2000)
  }

  return (
    <div className="min-h-screen">
      <Navbar />
      
      <main className="pt-32 pb-20 px-4">
        <div className="container mx-auto max-w-7xl">
          <Link href="/marketplace">
            <Button variant="ghost" className="mb-8">
              <ArrowLeft className="mr-2 h-4 w-4" />
              Back to Marketplace
            </Button>
          </Link>

          <div className="grid lg:grid-cols-2 gap-12">
            {/* Image Gallery */}
            <motion.div
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              className="space-y-4"
            >
              <div className="relative h-[500px] rounded-2xl overflow-hidden luxury-shadow">
                <Image
                  src={item.image}
                  alt={item.name}
                  fill
                  className="object-cover"
                />
              </div>
            </motion.div>

            {/* Product Details */}
            <motion.div
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              className="space-y-8"
            >
              <div>
                <div className="inline-block px-4 py-2 bg-gold/20 text-gold rounded-full text-sm font-semibold mb-4">
                  {item.category}
                </div>
                <h1 className="font-display text-5xl font-bold mb-4">{item.name}</h1>
                <p className="text-xl text-muted mb-6">{item.specs}</p>
                <p className="text-lg leading-relaxed">{item.description}</p>
              </div>

              {/* Luxury Pricing Display */}
              <div className="glass-card p-8 luxury-shadow">
                <div className="mb-6">
                  <div className="text-sm text-muted mb-2 uppercase tracking-wider">Price</div>
                  <div className="text-5xl font-display font-bold text-gold mb-6">
                    ${item.price.toLocaleString('en-US')}
                  </div>
                  <div className="text-sm text-muted mb-3 uppercase tracking-wider">Cryptocurrency Equivalent</div>
                  <div className="grid grid-cols-2 gap-4">
                    {Object.entries(cryptoPrices).map(([key, crypto]) => (
                      <div key={key} className="glass-card p-4">
                        <div className="flex items-center gap-2 mb-2">
                          <span className="text-2xl">{crypto.symbol}</span>
                          <span className="font-semibold">{key}</span>
                        </div>
                        <div className="text-gold font-bold text-lg">
                          {(item.price / crypto.price).toFixed(key === 'BTC' ? 4 : key === 'ETH' ? 3 : 2)}
                        </div>
                        <div className="text-xs text-muted">{crypto.name}</div>
                      </div>
                    ))}
                  </div>
                </div>

                <Button 
                  onClick={handleAddToCart}
                  className="w-full py-6 text-lg"
                  disabled={added}
                >
                  {added ? (
                    <>
                      <Check className="mr-2 h-5 w-5" />
                      Added to Cart
                    </>
                  ) : (
                    'Add to Cart'
                  )}
                </Button>
              </div>

              {/* Features */}
              <div className="glass-card p-8">
                <h3 className="font-display text-2xl font-bold mb-6">Key Features</h3>
                <ul className="space-y-3">
                  {item.features.map((feature, index) => (
                    <li key={index} className="flex items-start gap-3">
                      <div className="bg-gold/20 rounded-full p-1 mt-1">
                        <Check className="h-4 w-4 text-gold" />
                      </div>
                      <span className="text-lg">{feature}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </motion.div>
          </div>
        </div>
      </main>
    </div>
  )
}
