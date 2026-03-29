'use client'

import { useEffect, useState } from 'react'
import { Navbar } from '@/components/navbar/Navbar'
import { Button } from '@/components/ui/button'
import { motion } from 'framer-motion'
import { useCart } from '@/context/CartContext'
import { Trash2, ShoppingBag, Loader2 } from 'lucide-react'
import Image from 'next/image'
import Link from 'next/link'
import { api } from '@/lib/api'

export default function CartPage() {
  const { items, removeFromCart, total } = useCart()
  const [cryptoPrices, setCryptoPrices] = useState<any>({})
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const fetchPrices = async () => {
      try {
        const response = await api.calculator.getPrices()
        setCryptoPrices(response.data.crypto || {})
      } catch (error) {
        console.error('Failed to fetch prices:', error)
      } finally {
        setLoading(false)
      }
    }
    fetchPrices()
    const interval = setInterval(fetchPrices, 30000)
    return () => clearInterval(interval)
  }, [])

  return (
    <div className="min-h-screen">
      <Navbar />
      
      <main className="pt-32 pb-20 px-4">
        <div className="container mx-auto max-w-6xl">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="mb-12"
          >
            <h1 className="font-display text-5xl font-bold mb-4">
              Shopping <span className="text-white">Cart</span>
            </h1>
            <p className="text-xl text-muted">
              {items.length} {items.length === 1 ? 'item' : 'items'} in your cart
            </p>
          </motion.div>

          {items.length === 0 ? (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="text-center py-20"
            >
              <ShoppingBag className="h-24 w-24 mx-auto mb-6 text-muted" />
              <h2 className="font-display text-3xl font-bold mb-4">Your cart is empty</h2>
              <p className="text-muted mb-8">Start shopping to add items to your cart</p>
              <Link href="/marketplace">
                <Button size="lg">Browse Marketplace</Button>
              </Link>
            </motion.div>
          ) : (
            <div className="grid lg:grid-cols-3 gap-8">
              <div className="lg:col-span-2 space-y-4">
                {items.map((item, index) => (
                  <motion.div
                    key={item.id}
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: index * 0.1 }}
                    className="glass-card p-6"
                  >
                    <div className="flex gap-6">
                      <div className="relative w-32 h-32 rounded-lg overflow-hidden flex-shrink-0">
                        <Image
                          src={item.image}
                          alt={item.name}
                          fill
                          className="object-cover"
                        />
                      </div>
                      
                      <div className="flex-1">
                        <div className="flex justify-between mb-2">
                          <div>
                            <h3 className="font-display text-2xl font-bold">{item.name}</h3>
                            <p className="text-sm text-muted">{item.category}</p>
                          </div>
                          <button
                            onClick={() => removeFromCart(item.id)}
                            className="text-red-500 hover:text-red-400 transition-colors"
                          >
                            <Trash2 className="h-5 w-5" />
                          </button>
                        </div>
                        
                        <div className="text-2xl font-bold text-white">
                          ${item.price.toLocaleString()}
                        </div>
                      </div>
                    </div>
                  </motion.div>
                ))}
              </div>

              <div className="lg:col-span-1">
                <motion.div
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  className="glass-card p-8 sticky top-32"
                >
                  <h3 className="font-display text-2xl font-bold mb-6">Order Summary</h3>
                  
                  <div className="space-y-4 mb-6">
                    <div className="flex justify-between text-lg">
                      <span className="text-muted">Subtotal</span>
                      <span className="font-semibold">${total.toLocaleString()}</span>
                    </div>
                    <div className="flex justify-between text-lg">
                      <span className="text-muted">Service Fee (5%)</span>
                      <span className="font-semibold">${(total * 0.05).toLocaleString()}</span>
                    </div>
                    <div className="border-t border-[rgba(var(--accent),0.30)] pt-4">
                      <div className="flex justify-between text-2xl font-bold">
                        <span>Total</span>
                        <span className="text-white">${(total * 1.05).toLocaleString()}</span>
                      </div>
                    </div>
                  </div>

                  <div className="mb-6">
                    <div className="text-sm text-muted mb-3">Crypto Equivalent:</div>
                    {loading ? (
                      <div className="flex justify-center py-4">
                        <Loader2 className="h-6 w-6 animate-spin text-white" />
                      </div>
                    ) : (
                      <div className="space-y-2 text-sm">
                        {Object.entries(cryptoPrices).slice(0, 6).map(([crypto, data]: [string, any]) => (
                          <div key={crypto} className="flex justify-between">
                            <span className="text-muted">{crypto}</span>
                            <span className="font-semibold">
                              {((total * 1.05) / (data.price || 1)).toFixed(crypto === 'BTC' ? 4 : crypto === 'ETH' ? 3 : 2)} {crypto}
                            </span>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>

                  <Link href="/checkout">
                    <Button className="w-full py-6 text-lg">
                      Proceed to Checkout
                    </Button>
                  </Link>
                </motion.div>
              </div>
            </div>
          )}
        </div>
      </main>
    </div>
  )
}



