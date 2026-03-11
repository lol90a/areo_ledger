'use client'

import { useState, useEffect } from 'react'
import { Navbar } from '@/components/navbar/Navbar'
import { Button } from '@/components/ui/button'
import { motion } from 'framer-motion'
import { useCart } from '@/context/CartContext'
import { Check, Copy, Wallet, CreditCard, Hash, Loader2 } from 'lucide-react'
import { useRouter } from 'next/navigation'
import { api } from '@/lib/api'

export default function CheckoutPage() {
  const router = useRouter()
  const { total, items, clearCart } = useCart()
  const [selectedCrypto, setSelectedCrypto] = useState('BTC')
  const [orderData, setOrderData] = useState<any>(null)
  const [txHash, setTxHash] = useState('')
  const [senderWallet, setSenderWallet] = useState('')
  const [publicKey, setPublicKey] = useState('')
  const [copied, setCopied] = useState(false)
  const [loading, setLoading] = useState(false)
  const [cryptoPrices, setCryptoPrices] = useState<any>({})
  const [pricesLoading, setPricesLoading] = useState(true)

  useEffect(() => {
    const fetchPrices = async () => {
      try {
        const response = await api.calculator.getPrices()
        setCryptoPrices(response.data.crypto || {})
      } catch (error) {
        console.error('Failed to fetch prices:', error)
      } finally {
        setPricesLoading(false)
      }
    }
    fetchPrices()
    const interval = setInterval(fetchPrices, 30000)
    return () => clearInterval(interval)
  }, [])

  const cryptoOptions = [
    { symbol: 'BTC', name: 'Bitcoin', icon: '₿', network: 'Bitcoin Network', confirmations: '3 confirmations required' },
    { symbol: 'ETH', name: 'Ethereum', icon: 'Ξ', network: 'Ethereum Mainnet', confirmations: '12 confirmations required' },
    { symbol: 'USDT', name: 'Tether', icon: '₮', network: 'Ethereum (ERC-20)', confirmations: '12 confirmations required' },
    { symbol: 'USDC', name: 'USD Coin', icon: '$', network: 'Ethereum (ERC-20)', confirmations: '12 confirmations required' },
    { symbol: 'SOL', name: 'Solana', icon: '◎', network: 'Solana Network', confirmations: '32 confirmations required' },
    { symbol: 'BNB', name: 'Binance Coin', icon: 'BNB', network: 'BNB Smart Chain', confirmations: '15 confirmations required' },
  ]

  const totalWithFee = total * 1.05
  const selectedOption = cryptoOptions.find(c => c.symbol === selectedCrypto)!
  const cryptoPrice = cryptoPrices[selectedCrypto]?.price || 1
  const cryptoAmount = totalWithFee / cryptoPrice

  const handleGenerateAddress = async () => {
    setLoading(true)
    try {
      const user = JSON.parse(localStorage.getItem('user') || '{}')
      
      const response = await fetch('http://127.0.0.1:8080/api/orders', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          user_id: user.id || 'guest',
          items: items.map(item => ({
            product_id: item.id,
            name: item.name,
            price: item.price,
            quantity: item.quantity
          })),
          total: total,
          crypto_method: selectedCrypto,
          sender_wallet: '',
          tx_hash: null
        })
      })

      if (response.ok) {
        const data = await response.json()
        setOrderData(data)
      }
    } catch (error) {
      console.error('Error:', error)
    }
    setLoading(false)
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const confirmPayment = async () => {
    setLoading(true)
    try {
      if (orderData?.order_id) {
        await fetch(`http://127.0.0.1:8080/api/orders/${orderData.order_id}`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            sender_wallet: senderWallet,
            public_key: publicKey,
            tx_hash: txHash
          })
        })
      }
      clearCart()
      router.push('/dashboard')
    } catch (error) {
      console.error('Error:', error)
      alert('Failed to confirm payment. Please try again.')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen">
      <Navbar />
      
      <main className="pt-32 pb-20 px-4">
        <div className="container mx-auto max-w-4xl">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="mb-12"
          >
            <h1 className="font-display text-5xl font-bold mb-4">
              Checkout with <span className="text-gold">Crypto</span>
            </h1>
            <p className="text-xl text-muted">Select your preferred cryptocurrency</p>
          </motion.div>

          <div className="space-y-8">
            {/* Order Summary */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.1 }}
              className="glass-card p-8"
            >
              <h2 className="font-display text-2xl font-bold mb-6">Order Summary</h2>
              
              <div className="space-y-4 mb-6">
                {items.map((item) => (
                  <div key={item.id} className="flex justify-between">
                    <span className="text-muted">{item.name}</span>
                    <span className="font-semibold">${item.price.toLocaleString()}</span>
                  </div>
                ))}
              </div>
              
              <div className="space-y-3 border-t border-white/10 pt-4">
                <div className="flex justify-between text-lg">
                  <span className="text-muted">Subtotal</span>
                  <span className="font-semibold">${total.toLocaleString()}</span>
                </div>
                <div className="flex justify-between text-lg">
                  <span className="text-muted">Service Fee (5%)</span>
                  <span className="font-semibold">${(total * 0.05).toLocaleString()}</span>
                </div>
                <div className="border-t border-white/10 pt-3">
                  <div className="flex justify-between text-3xl font-bold">
                    <span>Total</span>
                    <span className="text-gold">${totalWithFee.toLocaleString()}</span>
                  </div>
                </div>
              </div>
            </motion.div>

            {/* Crypto Selection */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.2 }}
              className="glass-card p-8"
            >
              <h2 className="font-display text-2xl font-bold mb-6">Select Cryptocurrency</h2>
              {pricesLoading ? (
                <div className="flex justify-center py-12">
                  <Loader2 className="h-12 w-12 animate-spin text-gold" />
                </div>
              ) : (
                <div className="grid md:grid-cols-3 gap-4">
                  {cryptoOptions.map((crypto) => {
                    const price = cryptoPrices[crypto.symbol]?.price || 1
                    const amount = totalWithFee / price
                    return (
                      <button
                        key={crypto.symbol}
                        onClick={() => setSelectedCrypto(crypto.symbol)}
                        className={`glass-card p-6 text-left transition-all hover:bg-white/10 ${
                          selectedCrypto === crypto.symbol ? 'ring-2 ring-gold' : ''
                        }`}
                      >
                        <div className="flex items-start justify-between mb-3">
                          <div className="text-4xl">{crypto.icon}</div>
                          {selectedCrypto === crypto.symbol && (
                            <div className="bg-gold rounded-full p-1">
                              <Check className="h-4 w-4 text-primary" />
                            </div>
                          )}
                        </div>
                        <div className="font-bold text-lg mb-1">{crypto.symbol}</div>
                        <div className="text-sm text-muted mb-3">{crypto.name}</div>
                        <div className="text-xl font-bold text-gold mb-2">
                          {amount.toFixed(crypto.symbol === 'BTC' ? 4 : crypto.symbol === 'ETH' ? 3 : 2)} {crypto.symbol}
                        </div>
                        <div className="text-xs text-muted">{crypto.network}</div>
                      </button>
                    )
                  })}
                </div>
              )}
            </motion.div>

            {/* Payment Details */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3 }}
              className="glass-card p-8"
            >
              <h2 className="font-display text-2xl font-bold mb-6">Complete Payment</h2>
              
              {!orderData ? (
                <div className="space-y-6">
                  <div className="glass-card p-6 bg-gold/10 border-gold/30">
                    <div className="flex items-start gap-4">
                      <Wallet className="h-6 w-6 text-gold flex-shrink-0 mt-1" />
                      <div>
                        <h3 className="font-bold text-lg mb-2">Payment Instructions</h3>
                        <ul className="space-y-2 text-sm text-muted">
                          <li>✓ You will receive our wallet address</li>
                          <li>✓ Send the exact {selectedOption.symbol} amount shown</li>
                          <li>✓ Network: {selectedOption.network}</li>
                          <li>✓ {selectedOption.confirmations}</li>
                          <li>✓ Enter your wallet address (sender)</li>
                          <li>✓ Paste transaction hash after sending</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                  <Button onClick={handleGenerateAddress} disabled={loading} className="w-full py-6 text-lg">
                    {loading ? 'Generating...' : 'Generate Payment Address'}
                  </Button>
                </div>
              ) : (
                <div className="space-y-6">
                  {/* Step 1: Amount Required */}
                  <div className="glass-card p-6 bg-gold/10 border-gold/30">
                    <div className="flex items-center gap-3 mb-4">
                      <div className="bg-gold text-primary font-bold w-8 h-8 rounded-full flex items-center justify-center">1</div>
                      <h3 className="font-bold text-lg">Amount Required</h3>
                    </div>
                    <div className="text-5xl font-display font-bold text-gold mb-3">
                      {cryptoAmount.toFixed(selectedCrypto === 'BTC' ? 8 : selectedCrypto === 'ETH' ? 6 : 2)} {selectedCrypto}
                    </div>
                    <div className="text-lg text-muted mb-4">
                      Equivalent to ${totalWithFee.toLocaleString()} USD
                    </div>
                    <div className="text-sm glass-card p-3 bg-red-500/10 border-red-500/30">
                      ⚠️ Send EXACTLY this amount. Incorrect amounts may result in payment failure.
                    </div>
                  </div>

                  {/* Step 2: Our Wallet Address */}
                  <div className="glass-card p-6 bg-blue-500/10 border-blue-500/30">
                    <div className="flex items-center gap-3 mb-4">
                      <div className="bg-gold text-primary font-bold w-8 h-8 rounded-full flex items-center justify-center">2</div>
                      <h3 className="font-bold text-lg">Send {selectedCrypto} to Our Address</h3>
                    </div>
                    <div className="mb-3">
                      <div className="text-sm text-muted mb-2">Network: {selectedOption.network}</div>
                      <div className="text-sm text-muted mb-3">{selectedOption.confirmations}</div>
                    </div>
                    <div className="glass-card p-4 relative group bg-black/20">
                      <div className="font-mono text-sm break-all pr-12 text-gold font-bold">{orderData.wallet_address}</div>
                      <button
                        onClick={() => copyToClipboard(orderData.wallet_address)}
                        className="absolute right-3 top-1/2 -translate-y-1/2 glass-card p-2 hover:bg-white/10 transition-colors"
                      >
                        {copied ? <Check className="h-4 w-4 text-gold" /> : <Copy className="h-4 w-4" />}
                      </button>
                    </div>
                    <div className="text-xs text-muted mt-2">
                      Click the copy icon to copy the address
                    </div>
                  </div>

                  {/* Step 3: Your Wallet Details */}
                  <div className="glass-card p-6 bg-purple-500/10 border-purple-500/30">
                    <div className="flex items-center gap-3 mb-4">
                      <div className="bg-gold text-primary font-bold w-8 h-8 rounded-full flex items-center justify-center">3</div>
                      <h3 className="font-bold text-lg">Enter Your Wallet Details</h3>
                    </div>
                    
                    <div className="space-y-4">
                      <div>
                        <label className="flex items-center gap-2 text-sm text-muted mb-2 uppercase tracking-wider">
                          <Wallet className="h-4 w-4" />
                          Your {selectedCrypto} Wallet Address (Sender)
                        </label>
                        <input
                          type="text"
                          value={senderWallet}
                          onChange={(e) => setSenderWallet(e.target.value)}
                          placeholder={`Enter your ${selectedCrypto} wallet address`}
                          className="w-full glass-card px-4 py-4 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold font-mono text-sm"
                        />
                        <p className="text-xs text-muted mt-2">
                          This is YOUR wallet address where you're sending from
                        </p>
                      </div>

                      <div>
                        <label className="flex items-center gap-2 text-sm text-muted mb-2 uppercase tracking-wider">
                          <CreditCard className="h-4 w-4" />
                          Your Public Key (Optional)
                        </label>
                        <input
                          type="text"
                          value={publicKey}
                          onChange={(e) => setPublicKey(e.target.value)}
                          placeholder="Enter your public key if required"
                          className="w-full glass-card px-4 py-4 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold font-mono text-sm"
                        />
                        <p className="text-xs text-muted mt-2">
                          Some networks require a public key for verification
                        </p>
                      </div>
                    </div>
                  </div>

                  {/* Step 4: Transaction Hash */}
                  <div className="glass-card p-6 bg-green-500/10 border-green-500/30">
                    <div className="flex items-center gap-3 mb-4">
                      <div className="bg-gold text-primary font-bold w-8 h-8 rounded-full flex items-center justify-center">4</div>
                      <h3 className="font-bold text-lg">Confirm Transaction</h3>
                    </div>
                    
                    <div>
                      <label className="flex items-center gap-2 text-sm text-muted mb-2 uppercase tracking-wider">
                        <Hash className="h-4 w-4" />
                        Transaction Hash (TX ID)
                      </label>
                      <input
                        type="text"
                        value={txHash}
                        onChange={(e) => setTxHash(e.target.value)}
                        placeholder="Paste your transaction hash here after sending"
                        className="w-full glass-card px-4 py-4 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold font-mono text-sm"
                      />
                      <p className="text-xs text-muted mt-2">
                        After sending the payment, paste the transaction hash (TX ID) from your wallet
                      </p>
                    </div>
                  </div>

                  {/* Submit Button */}
                  <Button 
                    onClick={confirmPayment} 
                    disabled={!txHash || !senderWallet || loading}
                    className="w-full py-6 text-lg"
                  >
                    {loading ? 'Processing...' : (!txHash || !senderWallet ? 'Fill All Required Fields' : 'Confirm Payment & Complete Order')}
                  </Button>

                  <div className="glass-card p-4 bg-blue-500/10 border-blue-500/30 text-sm">
                    <div className="font-semibold mb-2">⏱️ Processing Time</div>
                    <div className="text-muted">
                      Your order will be processed after {selectedOption.confirmations.toLowerCase()}. 
                      This typically takes 10-60 minutes depending on network congestion.
                    </div>
                  </div>
                </div>
              )}
            </motion.div>
          </div>
        </div>
      </main>
    </div>
  )
}
