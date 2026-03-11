'use client'

import { useState, useEffect } from 'react'
import { Navbar } from '@/components/navbar/Navbar'
import { Button } from '@/components/ui/button'
import { motion } from 'framer-motion'
import { Search, Plane, Calendar, Users, X, Loader2 } from 'lucide-react'
import { api } from '@/lib/api'
import { v4 as uuidv4 } from 'uuid'

const PAYMENT_METHODS = [
  { id: 'btc', name: 'Bitcoin', symbol: 'BTC' },
  { id: 'eth', name: 'Ethereum', symbol: 'ETH' },
  { id: 'usdt', name: 'Tether', symbol: 'USDT' },
  { id: 'usdc', name: 'USD Coin', symbol: 'USDC' },
  { id: 'sol', name: 'Solana', symbol: 'SOL' },
  { id: 'binance', name: 'BNB', symbol: 'BNB' },
]

interface Flight {
  id: string
  route: string
  from: string
  to: string
  aircraft: string
  price: number
  duration: string
  seats: number
  departure_time: string
}

export default function FlightsPage() {
  const [from, setFrom] = useState('')
  const [to, setTo] = useState('')
  const [date, setDate] = useState('')
  const [passengers, setPassengers] = useState(1)
  const [flights, setFlights] = useState<Flight[]>([])
  const [searchLoading, setSearchLoading] = useState(false)
  const [selectedFlight, setSelectedFlight] = useState<Flight | null>(null)
  const [selectedPayment, setSelectedPayment] = useState('')
  const [loading, setLoading] = useState(false)
  const [bookingSuccess, setBookingSuccess] = useState<{ bookingId: string; totalPrice: number } | null>(null)

  useEffect(() => {
    handleSearch()
  }, [])

  const handleSearch = async () => {
    setSearchLoading(true)
    try {
      const response = await api.flights.search({
        from: from || undefined,
        to: to || undefined,
        date: date || undefined,
        passengers: passengers || undefined,
      })
      console.log('Flight search response:', response.data)
      
      // Map the response to handle both old and new backend structures
      const mappedFlights = response.data.map((flight: any) => ({
        id: flight.id || '',
        route: flight.route || `${flight.origin || flight.from || ''} → ${flight.destination || flight.to || ''}`,
        from: flight.from || flight.origin || '',
        to: flight.to || flight.destination || '',
        aircraft: flight.aircraft || 'Private Jet',
        price: Number(flight.price || flight.price_usd || 0),
        duration: flight.duration || 'N/A',
        seats: Number(flight.seats || 0),
        departure_time: flight.departure_time || ''
      }))
      
      console.log('Mapped flights:', mappedFlights)
      setFlights(mappedFlights)
    } catch (error) {
      console.error('Search failed:', error)
      setFlights([])
    } finally {
      setSearchLoading(false)
    }
  }

  return (
    <div className="min-h-screen">
      <Navbar />
      
      <main className="pt-32 pb-20 px-4">
        <div className="container mx-auto max-w-6xl">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="mb-12 text-center"
          >
            <h1 className="font-display text-5xl md:text-6xl font-bold mb-4">
              Book Your <span className="text-gold">Private Flight</span>
            </h1>
            <p className="text-xl text-muted">
              Pay with BTC, ETH, USDT, USDC, SOL, or BNB
            </p>
          </motion.div>

          {/* Search Form */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
            className="glass-card p-8 mb-12"
          >
            <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div>
                <label className="block text-sm text-muted mb-2">From</label>
                <div className="relative">
                  <Plane className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-muted" />
                  <input
                    type="text"
                    placeholder="New York (JFK)"
                    value={from}
                    onChange={(e) => setFrom(e.target.value)}
                    className="w-full glass-card pl-12 pr-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm text-muted mb-2">To</label>
                <div className="relative">
                  <Plane className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-muted rotate-90" />
                  <input
                    type="text"
                    placeholder="London (LHR)"
                    value={to}
                    onChange={(e) => setTo(e.target.value)}
                    className="w-full glass-card pl-12 pr-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm text-muted mb-2">Date</label>
                <div className="relative">
                  <Calendar className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-muted" />
                  <input
                    type="date"
                    value={date}
                    onChange={(e) => setDate(e.target.value)}
                    className="w-full glass-card pl-12 pr-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm text-muted mb-2">Passengers</label>
                <div className="relative">
                  <Users className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-muted" />
                  <input
                    type="number"
                    min="1"
                    max="19"
                    value={passengers}
                    onChange={(e) => setPassengers(parseInt(e.target.value))}
                    className="w-full glass-card pl-12 pr-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold"
                  />
                </div>
              </div>
            </div>

            <Button className="w-full mt-6 py-6 text-lg" onClick={handleSearch} disabled={searchLoading}>
              {searchLoading ? (
                <><Loader2 className="mr-2 h-5 w-5 animate-spin" /> Searching...</>
              ) : (
                <><Search className="mr-2 h-5 w-5" /> Search Flights</>
              )}
            </Button>
          </motion.div>

          {/* Available Flights */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
          >
            <h2 className="font-display text-3xl font-bold mb-6">
              {flights.length > 0 ? `Available Flights (${flights.length})` : 'No Flights Found'}
            </h2>
            
            {searchLoading ? (
              <div className="flex justify-center items-center py-20">
                <Loader2 className="h-12 w-12 animate-spin text-gold" />
              </div>
            ) : (
              <div className="space-y-4">
                {flights.map((flight, index) => (
                <motion.div
                  key={index}
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: 0.3 + index * 0.1 }}
                  className="glass-card p-6 hover:bg-white/10 transition-all"
                >
                  <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                    <div className="flex-1">
                      <h3 className="font-display text-2xl font-bold mb-2">{flight.route || 'Unknown Route'}</h3>
                      <div className="flex flex-wrap gap-4 text-sm text-muted">
                        <span>✈️ {flight.aircraft || 'N/A'}</span>
                        <span>⏱️ {flight.duration || 'N/A'}</span>
                        <span>👥 {flight.seats || 0} seats</span>
                      </div>
                    </div>
                    <div className="flex items-center gap-4">
                      <div className="text-right">
                        <div className="text-sm text-muted mb-1">From</div>
                        <div className="text-3xl font-bold text-gold">${(flight.price || 0).toLocaleString()}</div>
                      </div>
                      <Button size="lg" onClick={() => setSelectedFlight(flight)}>Book Now</Button>
                    </div>
                  </div>
                </motion.div>
                ))}
              </div>
            )}
          </motion.div>
        </div>
      </main>

      {/* Booking Modal */}
      {selectedFlight && (
        <div className="fixed inset-0 bg-black/80 flex items-center justify-center p-4 z-50">
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            className="glass-card p-8 max-w-md w-full relative"
          >
            <button
              onClick={() => { setSelectedFlight(null); setSelectedPayment(''); setBookingSuccess(null); }}
              className="absolute top-4 right-4 text-muted hover:text-white"
            >
              <X className="h-6 w-6" />
            </button>

            {!bookingSuccess ? (
              <>
                <h2 className="font-display text-2xl font-bold mb-4">Complete Booking</h2>
                <div className="mb-6">
                  <p className="text-muted mb-2">{selectedFlight.route}</p>
                  <p className="text-3xl font-bold text-gold">${(selectedFlight.price || 0).toLocaleString()}</p>
                  <p className="text-sm text-muted mt-1">Base price (15% markup will be added)</p>
                </div>

                <div className="mb-6">
                  <label className="block text-sm text-muted mb-3">Select Payment Method</label>
                  <div className="grid grid-cols-2 gap-3">
                    {PAYMENT_METHODS.map((method) => (
                      <button
                        key={method.id}
                        onClick={() => setSelectedPayment(method.id)}
                        className={`glass-card p-4 rounded-lg text-center transition-all ${
                          selectedPayment === method.id ? 'ring-2 ring-gold bg-gold/10' : 'hover:bg-white/5'
                        }`}
                      >
                        <div className="font-bold">{method.symbol}</div>
                        <div className="text-xs text-muted">{method.name}</div>
                      </button>
                    ))}
                  </div>
                </div>

                <Button
                  className="w-full"
                  disabled={!selectedPayment || loading}
                  onClick={async () => {
                    setLoading(true)
                    try {
                      const userId = localStorage.getItem('userId') || uuidv4()
                      if (!localStorage.getItem('userId')) {
                        localStorage.setItem('userId', userId)
                      }

                      const response = await api.bookings.create({
                        user_id: userId,
                        flight_id: selectedFlight.id,
                        base_price: selectedFlight.price,
                        payment_method: selectedPayment,
                      })

                      setBookingSuccess({
                        bookingId: response.data.booking_id,
                        totalPrice: response.data.total_price,
                      })
                    } catch (error) {
                      console.error('Booking failed:', error)
                      alert('Booking failed. Please try again.')
                    } finally {
                      setLoading(false)
                    }
                  }}
                >
                  {loading ? 'Processing...' : 'Confirm Booking'}
                </Button>
              </>
            ) : (
              <>
                <h2 className="font-display text-2xl font-bold mb-4 text-gold">Booking Confirmed!</h2>
                <div className="space-y-3 mb-6">
                  <div>
                    <p className="text-sm text-muted">Booking ID</p>
                    <p className="font-mono text-sm">{bookingSuccess.bookingId}</p>
                  </div>
                  <div>
                    <p className="text-sm text-muted">Total Price (with 15% markup)</p>
                    <p className="text-2xl font-bold text-gold">${(bookingSuccess.totalPrice || 0).toLocaleString()}</p>
                  </div>
                  <div className="glass-card p-4 rounded-lg">
                    <p className="text-sm text-muted mb-2">Next Steps:</p>
                    <ol className="text-sm space-y-1 list-decimal list-inside">
                      <li>Send payment to the provided wallet address</li>
                      <li>Submit your transaction hash</li>
                      <li>Wait for confirmation</li>
                    </ol>
                  </div>
                </div>
                <Button
                  className="w-full"
                  onClick={() => {
                    setSelectedFlight(null)
                    setSelectedPayment('')
                    setBookingSuccess(null)
                  }}
                >
                  Close
                </Button>
              </>
            )}
          </motion.div>
        </div>
      )}
    </div>
  )
}
