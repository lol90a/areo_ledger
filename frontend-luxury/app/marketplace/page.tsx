'use client'

import { useState, useEffect } from 'react'
import { Navbar } from '@/components/navbar/Navbar'
import { Button } from '@/components/ui/button'
import { motion } from 'framer-motion'
import { Search } from 'lucide-react'
import Image from 'next/image'
import Link from 'next/link'
import { useCart } from '@/context/CartContext'
import { api } from '@/lib/api'

export default function MarketplacePage() {
  const { addToCart } = useCart()
  const [items, setItems] = useState<any[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedCategory, setSelectedCategory] = useState('All')
  const [searchQuery, setSearchQuery] = useState('')

  useEffect(() => {
    const fetchProducts = async () => {
      try {
        const response = await api.products.getAll()
        setItems(response.data)
      } catch (error) {
        console.error('Failed to fetch products:', error)
      } finally {
        setLoading(false)
      }
    }
    fetchProducts()
  }, [])

  const categories = ['All', 'Private Jet', 'Yacht', 'Real Estate', 'Car']

  const filteredItems = items.filter(item => {
    const matchesCategory = selectedCategory === 'All' || item.category === selectedCategory
    const matchesSearch = item.name.toLowerCase().includes(searchQuery.toLowerCase())
    return matchesCategory && matchesSearch
  })

  return (
    <div className="min-h-screen">
      <Navbar />
      
      <main className="pt-32 pb-20 px-4">
        <div className="container mx-auto">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="mb-12"
          >
            <h1 className="font-display text-5xl md:text-6xl font-bold mb-4">
              Luxury <span className="text-white">Marketplace</span>
            </h1>
            <p className="text-xl text-muted max-w-2xl">
              Buy private jets, yachts, real estate, and luxury cars with cryptocurrency
            </p>
          </motion.div>

          {/* Filters */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
            className="mb-8 space-y-4"
          >
            <div className="relative max-w-md">
              <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-muted" />
              <input
                type="text"
                placeholder="Search items..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full glass-card pl-12 pr-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-[rgb(var(--accent))]"
              />
            </div>

            <div className="flex flex-wrap gap-3">
              {categories.map((category) => (
                <button
                  key={category}
                  onClick={() => setSelectedCategory(category)}
                  className={`px-6 py-2 rounded-lg transition-all ${
                    selectedCategory === category
                      ? 'bg-[rgb(var(--accent))] text-white font-semibold'
                      : 'glass-card hover:bg-white/10'
                  }`}
                >
                  {category}
                </button>
              ))}
            </div>
          </motion.div>

          <div className="mb-6 text-muted">
            {loading ? 'Loading...' : `Showing ${filteredItems.length} items`}
          </div>

          {/* Items Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            {filteredItems.map((item, index) => (
              <motion.div
                key={item.id}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: index * 0.1 }}
                className="glass-card overflow-hidden group cursor-pointer hover:bg-white/10 transition-all"
              >
                <div className="relative h-64 overflow-hidden">
                  <Image
                    src={item.image}
                    alt={item.name}
                    fill
                    className="object-cover group-hover:scale-110 transition-transform duration-500"
                  />
                  <div className="absolute top-4 right-4">
                    <span className="glass-card px-3 py-1 text-xs font-semibold text-white">
                      {item.category}
                    </span>
                  </div>
                </div>

                <div className="p-6">
                  <h3 className="font-display text-2xl font-bold mb-2 group-hover:text-white transition-colors">
                    {item.name}
                  </h3>
                  <p className="text-sm text-muted mb-4">{item.specs}</p>
                  
                  <div className="mb-4">
                    <div className="text-sm text-muted mb-1">Price in USD</div>
                    <div className="text-3xl font-bold text-white mb-3">
                      ${item.price.toLocaleString()}
                    </div>
                    <div className="text-xs text-muted space-y-1">
                      <div>≈ {(item.price / 45000).toFixed(4)} BTC</div>
                      <div>≈ {(item.price / 2500).toFixed(2)} ETH</div>
                      <div>≈ {(item.price / 1).toFixed(0)} USDT</div>
                    </div>
                  </div>

                  <div className="flex gap-2">
                    <Link href={`/marketplace/${item.id}`} className="flex-1">
                      <Button variant="outline" className="w-full">
                        View Details
                      </Button>
                    </Link>
                    <Button 
                      onClick={() => addToCart({
                        id: item.id,
                        name: item.name,
                        category: item.category,
                        price: item.price,
                        image: item.image
                      })}
                      className="flex-1"
                    >
                      Add to Cart
                    </Button>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>

          {filteredItems.length === 0 && (
            <div className="text-center py-20">
              <p className="text-xl text-muted">No items found</p>
            </div>
          )}
        </div>
      </main>
    </div>
  )
}


