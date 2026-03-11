import axios from 'axios';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://127.0.0.1:8080/api';

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

apiClient.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

export interface Asset {
  id: string;
  name: string;
  type: 'jet' | 'yacht' | 'real-estate' | 'car';
  description: string;
  image: string;
  totalValue: number;
  tokenPrice: number;
  availableTokens: number;
  totalTokens: number;
  roi: number;
  location: string;
  specifications: Record<string, string>;
}

export interface User {
  id: string;
  email: string;
  name: string;
  role: string;
  token: string;
}

export interface Booking {
  id: string;
  user_id: string;
  flight_id: string;
  status: string;
  total_price: number;
  payment_method: string;
  created_at: string;
}

export interface Payment {
  id: string;
  booking_id: string;
  chain: string;
  token: string;
  amount: number;
  status: string;
  tx_hash?: string;
  created_at: string;
}

export interface Portfolio {
  totalValue: number;
  totalAssets: number;
  roi: number;
  monthlyYield: number;
  assets: PortfolioAsset[];
}

export interface PortfolioAsset {
  assetId: string;
  assetName: string;
  assetType: string;
  tokensOwned: number;
  investmentValue: number;
  currentValue: number;
  roi: number;
}

export const api = {
  auth: {
    signup: (data: { email: string; name: string; password: string }) =>
      apiClient.post<User>('/users/signup', data),
    login: (data: { email: string; password: string }) =>
      apiClient.post<User>('/users/login', data),
  },
  
  assets: {
    getAll: () => apiClient.get<Asset[]>('/assets'),
    getById: (id: string) => apiClient.get<Asset>(`/assets/${id}`),
  },
  
  products: {
    getAll: () => apiClient.get('/products'),
    getById: (id: number) => apiClient.get(`/products/${id}`),
  },
  
  bookings: {
    create: (data: {
      user_id: string;
      flight_id: string;
      base_price: number;
      payment_method: string;
    }) => apiClient.post<{ booking_id: string; total_price: number }>('/bookings', data),
    getAll: () => apiClient.get<Booking[]>('/admin/bookings'),
  },
  
  flights: {
    search: (params: {
      from?: string;
      to?: string;
      date?: string;
      passengers?: number;
    }) => apiClient.get('/flights', { params }),
    getById: (id: string) => apiClient.get(`/flights/${id}`),
  },
  
  orders: {
    create: (data: {
      user_id: string;
      items: Array<{
        product_id: number;
        name: string;
        price: number;
        quantity: number;
      }>;
      total: number;
      crypto_method: string;
      sender_wallet: string;
      tx_hash?: string;
    }) => apiClient.post('/orders', data),
    getById: (id: string) => apiClient.get(`/orders/${id}`),
    getUserOrders: (userId: string) => apiClient.get(`/orders/user/${userId}`),
  },
  
  payments: {
    init: (data: { booking_id: string; method: string }) =>
      apiClient.post('/payments/init', data),
    confirm: (data: { booking_id: string; tx_hash: string }) =>
      apiClient.post('/payments/confirm', data),
    getAll: () => apiClient.get<Payment[]>('/admin/payments'),
  },
  
  portfolio: {
    getUserPortfolio: (userId: string) => apiClient.get(`/portfolio/${userId}`),
  },
  
  transactions: {
    getUserTransactions: (userId: string) => apiClient.get(`/transactions/${userId}`),
  },
  
  calculator: {
    convert: (data: {
      amount: number;
      from_currency: string;
      to_currency: string;
    }) => apiClient.post('/calculator/convert', data),
    getPrices: () => apiClient.get('/calculator/prices'),
  },
  
  admin: {
    getStats: () =>
      apiClient.get<{
        bookings: number;
        revenue: number;
        users: number;
        flights: number;
      }>('/admin/stats'),
  },
};

export default apiClient;
