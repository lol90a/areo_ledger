'use client'

import { useState, useEffect } from 'react'
import { Navbar } from '@/components/navbar/Navbar'
import { Button } from '@/components/ui/button'
import { motion } from 'framer-motion'
import { ArrowRightLeft, TrendingUp, Loader2 } from 'lucide-react'
import { api } from '@/lib/api'

export default function CalculatorPage() {
  const [amount, setAmount] = useState('1')
  const [fromType, setFromType] = useState<'fiat' | 'crypto'>('fiat')
  const [toType, setToType] = useState<'fiat' | 'crypto'>('crypto')
  const [fromCurrency, setFromCurrency] = useState('USD')
  const [toCurrency, setToCurrency] = useState('BTC')
  const [cryptoPrices, setCryptoPrices] = useState<any>({})
  const [fiatRates, setFiatRates] = useState<any>({})
  const [loading, setLoading] = useState(true)
  const [result, setResult] = useState(0)

  useEffect(() => {
    const fetchPrices = async () => {
      try {
        const response = await api.calculator.getPrices()
        setCryptoPrices(response.data.crypto || {})
        setFiatRates(response.data.fiat || {})
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

  useEffect(() => {
    const newResult = calculate()
    setResult(newResult)
  }, [amount, fromCurrency, toCurrency, fromType, toType, cryptoPrices, fiatRates])

  const calculate = () => {
    const inputAmount = parseFloat(amount) || 0

    if (fromType === 'fiat' && toType === 'crypto') {
      const fiatRate = fiatRates[fromCurrency]?.price || 1
      const usdAmount = inputAmount / fiatRate
      const cryptoPrice = cryptoPrices[toCurrency]?.price || 1
      return usdAmount / cryptoPrice
    } else if (fromType === 'crypto' && toType === 'fiat') {
      const cryptoPrice = cryptoPrices[fromCurrency]?.price || 1
      const usdAmount = inputAmount * cryptoPrice
      const fiatRate = fiatRates[toCurrency]?.price || 1
      return usdAmount * fiatRate
    } else if (fromType === 'fiat' && toType === 'fiat') {
      const fromRate = fiatRates[fromCurrency]?.price || 1
      const toRate = fiatRates[toCurrency]?.price || 1
      return (inputAmount / fromRate) * toRate
    } else {
      const fromPrice = cryptoPrices[fromCurrency]?.price || 1
      const toPrice = cryptoPrices[toCurrency]?.price || 1
      return (inputAmount * fromPrice) / toPrice
    }
  }

  const swapCurrencies = () => {
    const newFromType = toType
    const newToType = fromType
    const newFromCurrency = toCurrency
    const newToCurrency = fromCurrency
    
    setFromType(newFromType)
    setToType(newToType)
    
    const fromOptions = newFromType === 'fiat' ? fiatRates : cryptoPrices
    const toOptions = newToType === 'fiat' ? fiatRates : cryptoPrices
    
    if (!fromOptions[newFromCurrency]) {
      setFromCurrency(newFromType === 'fiat' ? 'USD' : 'BTC')
    } else {
      setFromCurrency(newFromCurrency)
    }
    
    if (!toOptions[newToCurrency]) {
      setToCurrency(newToType === 'crypto' ? 'BTC' : 'USD')
    } else {
      setToCurrency(newToCurrency)
    }
  }

  const fromOptions = fromType === 'fiat' ? fiatRates : cryptoPrices
  const toOptions = toType === 'fiat' ? fiatRates : cryptoPrices

  if (loading) {
    return (
      <div className="min-h-screen">
        <Navbar />
        <main className="pt-32 pb-20 px-4 flex items-center justify-center">
          <Loader2 className="h-12 w-12 animate-spin text-gold" />
        </main>
      </div>
    )
  }

  return (
    <div className="min-h-screen">
      <Navbar />
      
      <main className="pt-32 pb-20 px-4">
        <div className="container mx-auto max-w-4xl">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="mb-12 text-center"
          >
            <h1 className="font-display text-5xl md:text-6xl font-bold mb-4">
              Crypto <span className="text-gold">Calculator</span>
            </h1>
            <p className="text-xl text-muted">
              Convert between all world currencies and cryptocurrencies
            </p>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
            className="glass-card p-8 luxury-shadow"
          >
            {/* From Section */}
            <div className="mb-6">
              <label className="block text-sm text-muted mb-3 uppercase tracking-wider">From</label>
              
              <div className="flex gap-3 mb-4">
                <button
                  onClick={() => {
                    setFromType('fiat')
                    if (!fiatRates[fromCurrency]) setFromCurrency('USD')
                  }}
                  className={`flex-1 py-2 rounded-lg transition-all ${
                    fromType === 'fiat' ? 'bg-gold text-primary font-semibold' : 'glass-card'
                  }`}
                >
                  Fiat Currency
                </button>
                <button
                  onClick={() => {
                    setFromType('crypto')
                    if (!cryptoPrices[fromCurrency]) setFromCurrency('BTC')
                  }}
                  className={`flex-1 py-2 rounded-lg transition-all ${
                    fromType === 'crypto' ? 'bg-gold text-primary font-semibold' : 'glass-card'
                  }`}
                >
                  Cryptocurrency
                </button>
              </div>

              <div className="grid md:grid-cols-2 gap-4">
                <div>
                  <input
                    type="number"
                    value={amount}
                    onChange={(e) => setAmount(e.target.value)}
                    placeholder="Enter amount"
                    className="w-full glass-card px-4 py-4 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold text-2xl font-bold"
                  />
                </div>
                <div>
                  <select
                    value={fromCurrency}
                    onChange={(e) => setFromCurrency(e.target.value)}
                    className="w-full glass-card px-4 py-4 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold text-lg font-semibold bg-[#0A0A0A] text-white"
                    style={{ color: 'white' }}
                  >
                    {Object.entries(fromOptions).map(([code, data]: [string, any]) => (
                      <option key={code} value={code} style={{ backgroundColor: '#1a1a1a', color: 'white' }}>
                        {code} - {data.name}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
            </div>

            {/* Swap Button */}
            <div className="flex justify-center my-6">
              <button
                onClick={swapCurrencies}
                className="glass-card p-4 rounded-full hover:bg-white/10 transition-all hover:rotate-180 duration-300"
              >
                <ArrowRightLeft className="h-6 w-6 text-gold" />
              </button>
            </div>

            {/* To Section */}
            <div className="mb-6">
              <label className="block text-sm text-muted mb-3 uppercase tracking-wider">To</label>
              
              <div className="flex gap-3 mb-4">
                <button
                  onClick={() => {
                    setToType('fiat')
                    if (!fiatRates[toCurrency]) setToCurrency('USD')
                  }}
                  className={`flex-1 py-2 rounded-lg transition-all ${
                    toType === 'fiat' ? 'bg-gold text-primary font-semibold' : 'glass-card'
                  }`}
                >
                  Fiat Currency
                </button>
                <button
                  onClick={() => {
                    setToType('crypto')
                    if (!cryptoPrices[toCurrency]) setToCurrency('BTC')
                  }}
                  className={`flex-1 py-2 rounded-lg transition-all ${
                    toType === 'crypto' ? 'bg-gold text-primary font-semibold' : 'glass-card'
                  }`}
                >
                  Cryptocurrency
                </button>
              </div>

              <div>
                <select
                  value={toCurrency}
                  onChange={(e) => setToCurrency(e.target.value)}
                  className="w-full glass-card px-4 py-4 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold text-lg font-semibold mb-4 bg-[#0A0A0A] text-white"
                  style={{ color: 'white' }}
                >
                  {Object.entries(toOptions).map(([code, data]: [string, any]) => (
                    <option key={code} value={code} style={{ backgroundColor: '#1a1a1a', color: 'white' }}>
                      {code} - {data.name}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            {/* Result */}
            <div className="glass-card p-8 bg-gold/10 border-gold/30">
              <div className="text-sm text-muted mb-2 uppercase tracking-wider">Result</div>
              <div className="text-5xl font-display font-bold text-gold mb-2">
                {result.toFixed(toType === 'crypto' ? 8 : 2)}
              </div>
              <div className="text-xl text-muted">
                {toCurrency} - {toOptions[toCurrency]?.name || 'Unknown'}
              </div>
            </div>
          </motion.div>

          {/* Quick Conversions */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            className="mt-8 glass-card p-8"
          >
            <h3 className="font-display text-2xl font-bold mb-6 flex items-center gap-2">
              <TrendingUp className="h-6 w-6 text-gold" />
              Live Crypto Prices (USD)
            </h3>
            <div className="grid md:grid-cols-3 gap-4">
              {Object.entries(cryptoPrices).slice(0, 6).map(([code, data]: [string, any]) => (
                <div key={code} className="glass-card p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <span className="text-2xl">{data.symbol || code}</span>
                    <span className="font-bold">{code}</span>
                  </div>
                  <div className="text-gold font-bold text-xl">
                    ${(data.price || 0).toLocaleString()}
                  </div>
                  <div className="text-xs text-muted">{data.name}</div>
                </div>
              ))}
            </div>
          </motion.div>
        </div>
      </main>
    </div>
  )
}
