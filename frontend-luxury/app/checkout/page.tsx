'use client'

import { useState, useEffect } from 'react'
import { Navbar } from '@/components/navbar/Navbar'
import { Button } from '@/components/ui/button'
import { motion } from 'framer-motion'
import { useCart } from '@/context/CartContext'
import { Check, Copy, Wallet, CreditCard, Hash, Loader2 } from 'lucide-react'
import { useRouter } from 'next/navigation'
import { api, pendingBookingStore, session } from '@/lib/api'

const CRYPTO_OPTIONS = [
  { method: 'btc', symbol: 'BTC', name: 'Bitcoin', icon: 'BTC', network: 'Bitcoin Network', confirmations: '3 confirmations required' },
  { method: 'eth', symbol: 'ETH', name: 'Ethereum', icon: 'ETH', network: 'Ethereum Mainnet', confirmations: '12 confirmations required' },
  { method: 'usdt', symbol: 'USDT', name: 'Tether', icon: 'USDT', network: 'Ethereum (ERC-20)', confirmations: '12 confirmations required' },
  { method: 'usdc', symbol: 'USDC', name: 'USD Coin', icon: 'USDC', network: 'Ethereum (ERC-20)', confirmations: '12 confirmations required' },
  { method: 'sol', symbol: 'SOL', name: 'Solana', icon: 'SOL', network: 'Solana Network', confirmations: '32 confirmations required' },
  { method: 'bnb', symbol: 'BNB', name: 'Binance Coin', icon: 'BNB', network: 'BNB Smart Chain', confirmations: '15 confirmations required' },
]

export default function CheckoutPage() {
  const router = useRouter()
  const { total, items, clearCart } = useCart()
  const [selectedMethod, setSelectedMethod] = useState('btc')
  const [orderData, setOrderData] = useState<any>(null)
  const [txHash, setTxHash] = useState('')
  const [senderWallet, setSenderWallet] = useState('')
  const [publicKey, setPublicKey] = useState('')
  const [copied, setCopied] = useState(false)
  const [loading, setLoading] = useState(false)
  const [cryptoPrices, setCryptoPrices] = useState<any>({})
  const [pricesLoading, setPricesLoading] = useState(true)
  const [pendingBooking, setPendingBooking] = useState<any>(null)
  const [recoveringBooking, setRecoveringBooking] = useState(true)

  useEffect(() => {
    const restorePendingBooking = async () => {
      const localBooking = pendingBookingStore.get()
      if (localBooking) {
        setPendingBooking(localBooking)
        setSelectedMethod(localBooking.paymentMethod || 'btc')
        setRecoveringBooking(false)
        return
      }

      if (!session.getUserId()) {
        setRecoveringBooking(false)
        return
      }

      try {
        const response = await api.bookings.getLatestPending()
        pendingBookingStore.set(response.data)
        setPendingBooking(response.data)
        setSelectedMethod(response.data.paymentMethod || 'btc')
      } catch (error) {
        console.error('Failed to recover latest booking:', error)
      } finally {
        setRecoveringBooking(false)
      }
    }

    restorePendingBooking()
  }, [])

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

  const summaryTotal = pendingBooking?.totalPrice || total * 1.05
  const selectedOption = CRYPTO_OPTIONS.find((crypto) => crypto.method === selectedMethod) || CRYPTO_OPTIONS[0]
  const cryptoPrice = cryptoPrices[selectedOption.symbol]?.price || 1
  const cryptoAmount = summaryTotal / cryptoPrice

  const handleGenerateAddress = async () => {
    if (!pendingBooking?.bookingId) {
      alert('Create a booking on the flights page first.')
      router.push('/flights')
      return
    }

    setLoading(true)
    try {
      const response = await api.payments.init({
        booking_id: pendingBooking.bookingId,
        method: selectedMethod,
      })

      const updatedBooking = {
        ...pendingBooking,
        paymentMethod: selectedMethod,
        walletAddress: response.data.wallet_address,
        cryptoAmount: response.data.amount,
      }

      pendingBookingStore.set(updatedBooking)
      setPendingBooking(updatedBooking)
      setOrderData(response.data)
    } catch (error: any) {
      console.error('Error:', error)
      const message = error?.response?.data?.error || 'Unable to initialize payment.'
      alert(message)
    } finally {
      setLoading(false)
    }
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const confirmPayment = async () => {
    if (!pendingBooking?.bookingId) {
      alert('No pending booking was found.')
      return
    }

    if (!session.getUserId()) {
      alert('Please sign in again before confirming payment.')
      router.push('/signin')
      return
    }

    setLoading(true)
    try {
      await api.payments.confirm({
        booking_id: pendingBooking.bookingId,
        tx_hash: txHash,
      })
      pendingBookingStore.clear()
      clearCart()
      router.push('/transactions')
    } catch (error: any) {
      console.error('Error:', error)
      const message = error?.response?.data?.error || 'Failed to confirm payment. Please try again.'
      alert(message)
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
              Checkout with <span className="text-white">Crypto</span>
            </h1>
            <p className="text-xl text-muted">
              {pendingBooking ? `Complete payment for ${pendingBooking.flightRoute}` : 'Select your preferred cryptocurrency'}
            </p>
          </motion.div>

          <div className="space-y-8">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.1 }}
              className="glass-card p-8"
            >
              <h2 className="font-display text-2xl font-bold mb-6">Order Summary</h2>

              {pendingBooking ? (
                <div className="space-y-4 mb-6">
                  <div className="flex justify-between">
                    <span className="text-muted">Flight</span>
                    <span className="font-semibold">{pendingBooking.flightRoute}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted">Booking ID</span>
                    <span className="font-mono text-sm">{pendingBooking.bookingId}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted">Base Price</span>
                    <span className="font-semibold">${pendingBooking.basePrice.toLocaleString()}</span>
                  </div>
                </div>
              ) : recoveringBooking ? (
                <div className="flex justify-center py-6">
                  <Loader2 className="h-8 w-8 animate-spin text-white" />
                </div>
              ) : (
                <div className="space-y-4 mb-6">
                  {items.map((item) => (
                    <div key={item.id} className="flex justify-between">
                      <span className="text-muted">{item.name}</span>
                      <span className="font-semibold">${item.price.toLocaleString()}</span>
                    </div>
                  ))}
                </div>
              )}

              <div className="space-y-3 border-t border-[rgba(var(--accent),0.30)] pt-4">
                {!pendingBooking && (
                  <div className="flex justify-between text-lg">
                    <span className="text-muted">Subtotal</span>
                    <span className="font-semibold">${total.toLocaleString()}</span>
                  </div>
                )}
                <div className="flex justify-between text-lg">
                  <span className="text-muted">Service Fee</span>
                  <span className="font-semibold">{pendingBooking ? 'Included in booking total' : `$${(total * 0.05).toLocaleString()}`}</span>
                </div>
                <div className="border-t border-[rgba(var(--accent),0.30)] pt-3">
                  <div className="flex justify-between text-3xl font-bold">
                    <span>Total</span>
                    <span className="text-white">${summaryTotal.toLocaleString()}</span>
                  </div>
                </div>
              </div>
            </motion.div>

            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.2 }}
              className="glass-card p-8"
            >
              <h2 className="font-display text-2xl font-bold mb-6">Select Cryptocurrency</h2>
              {pricesLoading ? (
                <div className="flex justify-center py-12">
                  <Loader2 className="h-12 w-12 animate-spin text-white" />
                </div>
              ) : (
                <div className="grid md:grid-cols-3 gap-4">
                  {CRYPTO_OPTIONS.map((crypto) => {
                    const price = cryptoPrices[crypto.symbol]?.price || 1
                    const amount = summaryTotal / price
                    return (
                      <button
                        key={crypto.method}
                        onClick={() => setSelectedMethod(crypto.method)}
                        className={`glass-card p-6 text-left transition-all hover:bg-white/10 ${
                          selectedMethod === crypto.method ? 'ring-2 ring-[rgb(var(--accent))]' : ''
                        }`}
                      >
                        <div className="flex items-start justify-between mb-3">
                          <div className="text-3xl font-bold">{crypto.icon}</div>
                          {selectedMethod === crypto.method && (
                            <div className="bg-[rgb(var(--accent))] rounded-full p-1">
                              <Check className="h-4 w-4 text-white" />
                            </div>
                          )}
                        </div>
                        <div className="font-bold text-lg mb-1">{crypto.symbol}</div>
                        <div className="text-sm text-muted mb-3">{crypto.name}</div>
                        <div className="text-xl font-bold text-white mb-2">
                          {amount.toFixed(crypto.symbol === 'BTC' ? 4 : crypto.symbol === 'ETH' ? 3 : 2)} {crypto.symbol}
                        </div>
                        <div className="text-xs text-muted">{crypto.network}</div>
                      </button>
                    )
                  })}
                </div>
              )}
            </motion.div>

            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3 }}
              className="glass-card p-8"
            >
              <h2 className="font-display text-2xl font-bold mb-6">Complete Payment</h2>

              {!pendingBooking && !orderData ? (
                <div className="glass-card p-6 bg-[rgba(var(--accent),0.10)] border-[rgba(var(--accent),0.30)] text-sm text-muted">
                  {recoveringBooking
                    ? 'Recovering your latest booking from the backend...'
                    : 'Create a booking on the flights page first, then return here to initialize payment.'}
                </div>
              ) : !orderData ? (
                <div className="space-y-6">
                  <div className="glass-card p-6 bg-[rgba(var(--accent),0.10)] border-[rgba(var(--accent),0.30)]">
                    <div className="flex items-start gap-4">
                      <Wallet className="h-6 w-6 text-white flex-shrink-0 mt-1" />
                      <div>
                        <h3 className="font-bold text-lg mb-2">Payment Instructions</h3>
                        <ul className="space-y-2 text-sm text-muted">
                          <li>You will receive our wallet address</li>
                          <li>Send the exact {selectedOption.symbol} amount shown</li>
                          <li>Network: {selectedOption.network}</li>
                          <li>{selectedOption.confirmations}</li>
                          <li>Enter your wallet address (sender)</li>
                          <li>Paste transaction hash after sending</li>
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
                  <div className="glass-card p-6 bg-[rgba(var(--accent),0.10)] border-[rgba(var(--accent),0.30)]">
                    <div className="flex items-center gap-3 mb-4">
                      <div className="bg-[rgb(var(--accent))] text-white font-bold w-8 h-8 rounded-full flex items-center justify-center">1</div>
                      <h3 className="font-bold text-lg">Amount Required</h3>
                    </div>
                    <div className="text-5xl font-display font-bold text-white mb-3">
                      {(orderData.amount || cryptoAmount).toFixed(selectedOption.symbol === 'BTC' ? 8 : selectedOption.symbol === 'ETH' ? 6 : 2)} {selectedOption.symbol}
                    </div>
                    <div className="text-lg text-muted mb-4">
                      Equivalent to ${summaryTotal.toLocaleString()} USD
                    </div>
                    <div className="text-sm glass-card p-3 bg-red-500/10 border-red-500/30">
                      Send exactly this amount. Incorrect amounts may result in payment failure.
                    </div>
                  </div>

                  <div className="glass-card p-6 bg-blue-500/10 border-blue-500/30">
                    <div className="flex items-center gap-3 mb-4">
                      <div className="bg-[rgb(var(--accent))] text-white font-bold w-8 h-8 rounded-full flex items-center justify-center">2</div>
                      <h3 className="font-bold text-lg">Send {selectedOption.symbol} to Our Address</h3>
                    </div>
                    <div className="mb-3">
                      <div className="text-sm text-muted mb-2">Network: {selectedOption.network}</div>
                      <div className="text-sm text-muted mb-3">{selectedOption.confirmations}</div>
                    </div>
                    <div className="glass-card p-4 relative group bg-black/20">
                      <div className="font-mono text-sm break-all pr-12 text-white font-bold">{orderData.wallet_address}</div>
                      <button
                        onClick={() => copyToClipboard(orderData.wallet_address)}
                        className="absolute right-3 top-1/2 -translate-y-1/2 glass-card p-2 hover:bg-white/10 transition-colors"
                      >
                        {copied ? <Check className="h-4 w-4 text-white" /> : <Copy className="h-4 w-4" />}
                      </button>
                    </div>
                    <div className="text-xs text-muted mt-2">
                      Click the copy icon to copy the address
                    </div>
                  </div>

                  <div className="glass-card p-6 bg-purple-500/10 border-purple-500/30">
                    <div className="flex items-center gap-3 mb-4">
                      <div className="bg-[rgb(var(--accent))] text-white font-bold w-8 h-8 rounded-full flex items-center justify-center">3</div>
                      <h3 className="font-bold text-lg">Enter Your Wallet Details</h3>
                    </div>

                    <div className="space-y-4">
                      <div>
                        <label className="flex items-center gap-2 text-sm text-muted mb-2 uppercase tracking-wider">
                          <Wallet className="h-4 w-4" />
                          Your {selectedOption.symbol} Wallet Address (Sender)
                        </label>
                        <input
                          type="text"
                          value={senderWallet}
                          onChange={(e) => setSenderWallet(e.target.value)}
                          placeholder={`Enter your ${selectedOption.symbol} wallet address`}
                          className="w-full glass-card px-4 py-4 rounded-lg focus:outline-none focus:ring-2 focus:ring-[rgb(var(--accent))] font-mono text-sm"
                        />
                        <p className="text-xs text-muted mt-2">
                          This is your wallet address where you're sending from
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
                          className="w-full glass-card px-4 py-4 rounded-lg focus:outline-none focus:ring-2 focus:ring-[rgb(var(--accent))] font-mono text-sm"
                        />
                        <p className="text-xs text-muted mt-2">
                          Some networks require a public key for verification
                        </p>
                      </div>
                    </div>
                  </div>

                  <div className="glass-card p-6 bg-green-500/10 border-green-500/30">
                    <div className="flex items-center gap-3 mb-4">
                      <div className="bg-[rgb(var(--accent))] text-white font-bold w-8 h-8 rounded-full flex items-center justify-center">4</div>
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
                        className="w-full glass-card px-4 py-4 rounded-lg focus:outline-none focus:ring-2 focus:ring-[rgb(var(--accent))] font-mono text-sm"
                      />
                      <p className="text-xs text-muted mt-2">
                        After sending the payment, paste the transaction hash from your wallet.
                      </p>
                    </div>
                  </div>

                  <Button
                    onClick={confirmPayment}
                    disabled={!txHash || !senderWallet || loading}
                    className="w-full py-6 text-lg"
                  >
                    {loading ? 'Processing...' : (!txHash || !senderWallet ? 'Fill All Required Fields' : 'Confirm Payment')}
                  </Button>

                  <div className="glass-card p-4 bg-blue-500/10 border-blue-500/30 text-sm">
                    <div className="font-semibold mb-2">Processing Time</div>
                    <div className="text-muted">
                      Your booking will be processed after {selectedOption.confirmations.toLowerCase()}.
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




