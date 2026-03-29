import axios, { AxiosResponse } from 'axios'

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://127.0.0.1:8080/api'

const MOCK_CRYPTO_PRICES = {
  BTC: { price: 68350, symbol: 'BTC', name: 'Bitcoin' },
  ETH: { price: 3520, symbol: 'ETH', name: 'Ethereum' },
  USDT: { price: 1, symbol: 'USDT', name: 'Tether' },
  USDC: { price: 1, symbol: 'USDC', name: 'USD Coin' },
  SOL: { price: 185, symbol: 'SOL', name: 'Solana' },
  BNB: { price: 590, symbol: 'BNB', name: 'Binance Coin' },
} as const

const MOCK_FIAT_RATES = {
  USD: { price: 1, name: 'US Dollar' },
  EUR: { price: 0.92, name: 'Euro' },
  GBP: { price: 0.79, name: 'British Pound' },
  AED: { price: 3.67, name: 'UAE Dirham' },
  EGP: { price: 49.35, name: 'Egyptian Pound' },
  SAR: { price: 3.75, name: 'Saudi Riyal' },
} as const

const MOCK_PRODUCTS = [
  {
    id: 1,
    name: 'Gulfstream G700',
    category: 'Private Jet',
    price: 25000000,
    image: 'https://images.unsplash.com/photo-1436491865332-7a61a109cc05?auto=format&fit=crop&w=1200&q=80',
    specs: 'Ultra-long-range flagship jet with 19-passenger cabin',
    description: 'A flagship private jet designed for long-haul executive travel with a quiet cabin and top-tier avionics.',
    features: ['19-passenger layout', 'Nonstop transatlantic range', 'High-speed Wi-Fi', 'Dedicated lounge and suite'],
  },
  {
    id: 2,
    name: 'Benetti Oasis 40M',
    category: 'Yacht',
    price: 24500000,
    image: 'https://images.unsplash.com/photo-1567899378494-47b22a2ae96a?auto=format&fit=crop&w=1200&q=80',
    specs: '40m tri-deck yacht with beach club and infinity pool',
    description: 'A modern superyacht with open social spaces, custom interiors, and extended cruising comfort.',
    features: ['Beach club deck', 'Infinity pool', 'Five guest suites', 'Crew service quarters'],
  },
  {
    id: 3,
    name: 'Mayfair Sky Residence',
    category: 'Real Estate',
    price: 18200000,
    image: 'https://images.unsplash.com/photo-1502672260266-1c1ef2d93688?auto=format&fit=crop&w=1200&q=80',
    specs: 'Penthouse residence with skyline terrace and private concierge',
    description: 'A landmark residence in central London tailored for buyers looking for prestige, privacy, and views.',
    features: ['Private concierge', 'Panoramic terrace', 'Smart home system', 'Valet and wellness access'],
  },
  {
    id: 4,
    name: 'Ferrari SF90 XX',
    category: 'Car',
    price: 1250000,
    image: 'https://images.unsplash.com/photo-1494976388531-d1058494cdd8?auto=format&fit=crop&w=1200&q=80',
    specs: 'Hybrid hypercar with track-ready aero package',
    description: 'A limited-production Ferrari combining hybrid performance, aggressive aerodynamics, and daily drivability.',
    features: ['1030 hp hybrid powertrain', 'Track telemetry', 'Carbon aero package', 'Collector-grade allocation'],
  },
] as const

const MOCK_ASSETS = [
  {
    id: 'asset-g650',
    name: 'Gulfstream G650 Fractional Ownership',
    type: 'jet' as const,
    description: 'Invest in one of the world\'s most desirable long-range jets with revenue-backed charter utilization.',
    image: 'https://images.unsplash.com/photo-1540962351504-03099e0a754b?auto=format&fit=crop&w=1200&q=80',
    totalValue: 65000000,
    tokenPrice: 10000,
    availableTokens: 3250,
    totalTokens: 6500,
    roi: 18.5,
    location: 'Miami, FL',
    specifications: {
      'Max Range': '7,000 nm',
      'Max Speed': 'Mach 0.925',
      Passengers: '19',
      Year: '2023',
    },
  },
  {
    id: 'asset-yacht-oasis',
    name: 'Oasis 34M Charter Yield Pool',
    type: 'yacht' as const,
    description: 'Tokenized exposure to luxury Mediterranean charter revenue with high-season demand upside.',
    image: 'https://images.unsplash.com/photo-1567899378494-47b22a2ae96a?auto=format&fit=crop&w=1200&q=80',
    totalValue: 21000000,
    tokenPrice: 5000,
    availableTokens: 2100,
    totalTokens: 4200,
    roi: 14.2,
    location: 'Monaco',
    specifications: {
      Length: '34m',
      Cabins: '5',
      Builder: 'Benetti',
      Region: 'Mediterranean',
    },
  },
  {
    id: 'asset-dubai',
    name: 'Dubai Marina Residences Fund',
    type: 'real-estate' as const,
    description: 'A professionally managed luxury residential portfolio with rental distributions and capital appreciation.',
    image: 'https://images.unsplash.com/photo-1512917774080-9991f1c4c750?auto=format&fit=crop&w=1200&q=80',
    totalValue: 32000000,
    tokenPrice: 2500,
    availableTokens: 4800,
    totalTokens: 12800,
    roi: 11.8,
    location: 'Dubai, UAE',
    specifications: {
      Units: '18',
      Occupancy: '94%',
      Market: 'Prime residential',
      Strategy: 'Income + appreciation',
    },
  },
] as const

const MOCK_FLIGHTS = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    from: 'Dubai',
    to: 'London',
    aircraft: 'Boeing 777',
    price: 9500,
    duration: '7h 30m',
    seats: 8,
    departure_time: '2026-04-04T09:30:00Z',
  },
  {
    id: '00000000-0000-0000-0000-000000000002',
    from: 'New York',
    to: 'Los Angeles',
    aircraft: 'Gulfstream G700',
    price: 25000,
    duration: '5h 45m',
    seats: 12,
    departure_time: '2026-04-06T13:15:00Z',
  },
  {
    id: '00000000-0000-0000-0000-000000000003',
    from: 'London',
    to: 'Dubai',
    aircraft: 'Airbus A380',
    price: 8500,
    duration: '7h 15m',
    seats: 6,
    departure_time: '2026-04-02T16:45:00Z',
  },
] as const

const PENDING_BOOKING_KEY = 'pendingBooking'
const USER_KEY = 'user'
const TOKEN_KEY = 'token'
const USER_ID_KEY = 'userId'

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
})

apiClient.interceptors.request.use((config) => {
  if (typeof window !== 'undefined') {
    const token = window.localStorage.getItem(TOKEN_KEY)
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
  }
  return config
})

export interface Asset {
  id: string
  name: string
  type: 'jet' | 'yacht' | 'real-estate' | 'car'
  description: string
  image: string
  totalValue: number
  tokenPrice: number
  availableTokens: number
  totalTokens: number
  roi: number
  location: string
  specifications: Record<string, string>
}

export interface User {
  id: string
  email: string
  name: string
  role: string
  token: string
}

export interface Booking {
  id: string
  user_id: string
  flight_id: string
  status: string
  total_price: number
  payment_method: string
  created_at: string
}

export interface Payment {
  id: string
  booking_id: string
  chain: string
  token: string
  amount: number
  status: string
  tx_hash?: string
  created_at: string
}

export interface Portfolio {
  totalValue: number
  totalAssets: number
  roi: number
  monthlyYield: number
  assets: PortfolioAsset[]
}

export interface PortfolioAsset {
  assetId: string
  assetName: string
  assetType: string
  tokensOwned: number
  investmentValue: number
  currentValue: number
  roi: number
}

export interface PendingBooking {
  bookingId: string
  flightId: string
  flightRoute: string
  basePrice: number
  totalPrice: number
  paymentMethod: string
  createdAt: string
  walletAddress?: string
  cryptoAmount?: number
}

export interface LatestBookingResponse {
  booking_id: string
  flight_id: string
  status: string
  base_price: number
  total_price: number
  payment_method: string
  created_at: string
}

function toResponse<T>(data: T): Promise<AxiosResponse<T>> {
  return Promise.resolve({
    data,
    status: 200,
    statusText: 'OK',
    headers: {},
    config: { headers: {} as never },
  } as AxiosResponse<T>)
}

function isFallbackEligible(error: unknown): boolean {
  if (!axios.isAxiosError(error)) {
    return true
  }

  return !error.response || (error.response.status >= 404 && error.response.status < 500)
}

function readJson<T>(key: string): T | null {
  if (typeof window === 'undefined') {
    return null
  }

  const raw = window.localStorage.getItem(key)
  if (!raw) {
    return null
  }

  try {
    return JSON.parse(raw) as T
  } catch {
    return null
  }
}

function writeJson<T>(key: string, value: T) {
  if (typeof window === 'undefined') {
    return
  }

  window.localStorage.setItem(key, JSON.stringify(value))
}

function getStoredUser(): User | null {
  return readJson<User>(USER_KEY)
}

function setStoredUser(user: User) {
  writeJson(USER_KEY, user)
  if (typeof window !== 'undefined') {
    window.localStorage.setItem(TOKEN_KEY, user.token)
    window.localStorage.setItem(USER_ID_KEY, user.id)
    document.cookie = `user=${encodeURIComponent(JSON.stringify(user))}; path=/; max-age=86400; samesite=lax`
  }
}

function clearStoredUser() {
  if (typeof window === 'undefined') {
    return
  }

  window.localStorage.removeItem(USER_KEY)
  window.localStorage.removeItem(TOKEN_KEY)
  window.localStorage.removeItem(USER_ID_KEY)
  document.cookie = 'user=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;'
}

function getPendingBooking(): PendingBooking | null {
  return readJson<PendingBooking>(PENDING_BOOKING_KEY)
}

function setPendingBooking(booking: PendingBooking) {
  writeJson(PENDING_BOOKING_KEY, booking)
}

function clearPendingBooking() {
  if (typeof window !== 'undefined') {
    window.localStorage.removeItem(PENDING_BOOKING_KEY)
  }
}

function deriveFlightRoute(flightId: string): string {
  const flight = MOCK_FLIGHTS.find((item) => item.id === flightId)
  return flight ? `${flight.from} -> ${flight.to}` : 'Flight booking'
}

function buildPortfolio(): Portfolio {
  const pendingBooking = getPendingBooking()
  const assets: PortfolioAsset[] = pendingBooking
    ? [{
        assetId: pendingBooking.flightId,
        assetName: pendingBooking.flightRoute,
        assetType: 'flight booking',
        tokensOwned: 1,
        investmentValue: pendingBooking.totalPrice,
        currentValue: pendingBooking.totalPrice,
        roi: 0,
      }]
    : []

  const totalValue = assets.reduce((sum, asset) => sum + asset.currentValue, 0)

  return {
    totalValue,
    totalAssets: assets.length,
    roi: 0,
    monthlyYield: 0,
    assets,
  }
}

function buildAdminStats() {
  const pendingBooking = getPendingBooking()
  return {
    bookings: pendingBooking ? 1 : 0,
    revenue: pendingBooking ? pendingBooking.totalPrice : 0,
    users: getStoredUser() ? 1 : 0,
    flights: MOCK_FLIGHTS.length,
  }
}

export const session = {
  getUser: getStoredUser,
  getUserId: () => (typeof window === 'undefined' ? null : window.localStorage.getItem(USER_ID_KEY)),
  getToken: () => (typeof window === 'undefined' ? null : window.localStorage.getItem(TOKEN_KEY)),
  setUser: setStoredUser,
  updateUser: (partial: Partial<User>) => {
    const current = getStoredUser()
    if (!current) {
      return null
    }

    const updated = { ...current, ...partial }
    setStoredUser(updated)
    return updated
  },
  clear: clearStoredUser,
  isAuthenticated: () => Boolean(getStoredUser()?.token),
}

export const pendingBookingStore = {
  get: getPendingBooking,
  set: setPendingBooking,
  clear: clearPendingBooking,
}

export const api = {
  auth: {
    signup: async (data: { email: string; name: string; password: string }) => {
      const response = await apiClient.post<User>('/users/signup', data)
      setStoredUser(response.data)
      return response
    },
    login: async (data: { email: string; password: string }) => {
      const response = await apiClient.post<User>('/users/login', data)
      setStoredUser(response.data)
      return response
    },
  },

  assets: {
    getAll: async () => {
      try {
        return await apiClient.get<Asset[]>('/assets')
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }
        return toResponse([...MOCK_ASSETS])
      }
    },
    getById: async (id: string) => {
      try {
        return await apiClient.get<Asset>(`/assets/${id}`)
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }
        const asset = MOCK_ASSETS.find((item) => item.id === id) || MOCK_ASSETS[0]
        return toResponse(asset)
      }
    },
  },

  products: {
    getAll: async () => {
      try {
        return await apiClient.get('/products')
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }
        return toResponse([...MOCK_PRODUCTS])
      }
    },
    getById: async (id: number) => {
      try {
        return await apiClient.get(`/products/${id}`)
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }
        const product = MOCK_PRODUCTS.find((item) => item.id === id) || MOCK_PRODUCTS[0]
        return toResponse(product)
      }
    },
  },

  bookings: {
    create: async (data: {
      user_id: string
      flight_id: string
      base_price: number
      payment_method: string
    }) => apiClient.post<{ booking_id: string; total_price: number }>('/bookings', data),
    getLatestPending: async () => {
      const response = await apiClient.get<LatestBookingResponse>('/bookings/latest')
      return toResponse({
        bookingId: response.data.booking_id,
        flightId: response.data.flight_id,
        flightRoute: deriveFlightRoute(response.data.flight_id),
        basePrice: response.data.base_price,
        totalPrice: response.data.total_price,
        paymentMethod: response.data.payment_method,
        createdAt: response.data.created_at,
      })
    },
    getAll: async () => {
      try {
        return await apiClient.get<Booking[]>('/admin/bookings')
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }
        const booking = getPendingBooking()
        const fallback: Booking[] = booking
          ? [{
              id: booking.bookingId,
              user_id: session.getUserId() || '',
              flight_id: booking.flightId,
              status: booking.walletAddress ? 'pending' : 'created',
              total_price: booking.totalPrice,
              payment_method: booking.paymentMethod,
              created_at: booking.createdAt,
            }]
          : []
        return toResponse(fallback)
      }
    },
  },

  flights: {
    search: async (params: {
      from?: string
      to?: string
      date?: string
      passengers?: number
    }) => {
      try {
        return await apiClient.get('/flights', { params })
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }

        const flights = MOCK_FLIGHTS
          .filter((flight) => !params.from || flight.from.toLowerCase().includes(params.from.toLowerCase()))
          .filter((flight) => !params.to || flight.to.toLowerCase().includes(params.to.toLowerCase()))
          .filter((flight) => !params.passengers || flight.seats >= params.passengers)
          .map((flight) => ({
            ...flight,
            route: `${flight.from} -> ${flight.to}`,
          }))

        return toResponse(flights)
      }
    },
    getById: async (id: string) => {
      try {
        return await apiClient.get(`/flights/${id}`)
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }
        const flight = MOCK_FLIGHTS.find((item) => item.id === id) || MOCK_FLIGHTS[0]
        return toResponse({ ...flight, route: `${flight.from} -> ${flight.to}` })
      }
    },
  },

  payments: {
    init: (data: { booking_id: string; method: string }) => apiClient.post('/payments/init', data),
    confirm: (data: { booking_id: string; tx_hash: string }) => apiClient.post('/payments/confirm', data),
    getAll: async () => {
      try {
        return await apiClient.get<Payment[]>('/admin/payments')
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }
        const booking = getPendingBooking()
        const fallback: Payment[] = booking && booking.walletAddress
          ? [{
              id: booking.bookingId,
              booking_id: booking.bookingId,
              chain: booking.paymentMethod,
              token: booking.paymentMethod,
              amount: booking.cryptoAmount || 0,
              status: booking.cryptoAmount ? 'pending' : 'created',
              tx_hash: undefined,
              created_at: booking.createdAt,
            }]
          : []
        return toResponse(fallback)
      }
    },
  },

  portfolio: {
    getUserPortfolio: async (userId: string) => {
      try {
        return await apiClient.get(`/portfolio/${userId}`)
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }
        return toResponse(buildPortfolio())
      }
    },
  },

  transactions: {
    getUserTransactions: (userId: string) => apiClient.get(`/transactions/${userId}`),
  },

  calculator: {
    convert: async (data: {
      amount: number
      from_currency: string
      to_currency: string
    }) => {
      try {
        return await apiClient.post('/calculator/convert', data)
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }

        const prices = { ...MOCK_FIAT_RATES, ...MOCK_CRYPTO_PRICES }
        const fromPrice = prices[data.from_currency as keyof typeof prices]?.price || 1
        const toPrice = prices[data.to_currency as keyof typeof prices]?.price || 1
        const amountInUsd = data.amount * fromPrice
        return toResponse({ result: amountInUsd / toPrice })
      }
    },
    getPrices: async () => {
      try {
        return await apiClient.get('/calculator/prices')
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }
        return toResponse({
          crypto: MOCK_CRYPTO_PRICES,
          fiat: MOCK_FIAT_RATES,
          updated_at: new Date().toISOString(),
        })
      }
    },
  },

  admin: {
    getStats: async () => {
      try {
        return await apiClient.get<{
          bookings: number
          revenue: number
          users: number
          flights: number
        }>('/admin/stats')
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }
        return toResponse(buildAdminStats())
      }
    },
  },

  profile: {
    getById: async (userId: string) => {
      try {
        return await apiClient.get(`/profile/${userId}`)
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }

        const user = getStoredUser()
        if (!user || user.id !== userId) {
          throw error
        }

        return toResponse({
          id: user.id,
          email: user.email,
          name: user.name,
          role: user.role,
          wallet_address: null,
          created_at: new Date().toISOString(),
        })
      }
    },
    update: async (userId: string, data: { name: string; email: string; wallet_address: string }) => {
      try {
        return await apiClient.put(`/profile/${userId}`, data)
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }

        const updated = session.updateUser({ name: data.name, email: data.email })
        return toResponse({
          id: userId,
          email: updated?.email || data.email,
          name: updated?.name || data.name,
          role: updated?.role || 'user',
          wallet_address: data.wallet_address || null,
          created_at: new Date().toISOString(),
        })
      }
    },
    updatePassword: async (userId: string, data: { current_password: string; new_password: string }) => {
      try {
        return await apiClient.put(`/profile/${userId}/password`, data)
      } catch (error) {
        if (!isFallbackEligible(error)) {
          throw error
        }
        return toResponse({ message: 'Password updated locally. Add backend profile endpoints to persist this change.' })
      }
    },
  },
}

export default apiClient
