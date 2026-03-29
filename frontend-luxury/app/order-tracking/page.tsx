"use client";

import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { pendingBookingStore, session } from "@/lib/api";

interface OrderHistoryItem {
  status: string;
  notes: string | null;
  created_at: string;
}

interface OrderTracking {
  order_id: string;
  status: string;
  created_at: string;
  updated_at: string;
  history: OrderHistoryItem[];
}

interface UserOrder {
  id: string;
  status: string;
  total_amount: string;
  created_at: string;
  item_count: number;
}

export default function OrderTrackingPage() {
  const [orders, setOrders] = useState<UserOrder[]>([]);
  const [selectedOrder, setSelectedOrder] = useState<OrderTracking | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const userId = session.getUserId();
    const pendingBooking = pendingBookingStore.get();

    if (userId && pendingBooking) {
      setOrders([
        {
          id: pendingBooking.bookingId,
          status: pendingBooking.walletAddress ? "processing" : "pending",
          total_amount: pendingBooking.totalPrice.toFixed(2),
          created_at: pendingBooking.createdAt,
          item_count: 1,
        },
      ]);
    }

    setLoading(false);
  }, []);

  const loadOrderTracking = (orderId: string) => {
    const pendingBooking = pendingBookingStore.get();
    if (!pendingBooking || pendingBooking.bookingId !== orderId) {
      return;
    }

    setSelectedOrder({
      order_id: pendingBooking.bookingId,
      status: pendingBooking.walletAddress ? "processing" : "pending",
      created_at: pendingBooking.createdAt,
      updated_at: new Date().toISOString(),
      history: [
        {
          status: "booking_created",
          notes: `Flight booked for ${pendingBooking.flightRoute}`,
          created_at: pendingBooking.createdAt,
        },
        {
          status: pendingBooking.walletAddress ? "payment_initialized" : "awaiting_payment",
          notes: pendingBooking.walletAddress
            ? `Payment wallet generated for ${pendingBooking.paymentMethod}`
            : "Waiting for checkout to initialize payment.",
          created_at: new Date().toISOString(),
        },
      ],
    });
  };

  const getStatusColor = (status: string) => {
    const colors: Record<string, string> = {
      pending: "bg-blue-500",
      processing: "bg-blue-500",
      confirmed: "bg-green-500",
      shipped: "bg-purple-500",
      delivered: "bg-emerald-500",
      cancelled: "bg-red-500",
      booking_created: "bg-blue-500",
      awaiting_payment: "bg-blue-500",
      payment_initialized: "bg-blue-500",
    };
    return colors[status.toLowerCase()] || "bg-gray-500";
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-black text-white flex items-center justify-center">
        <div className="text-2xl">Loading orders...</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-black text-white p-8">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-4xl font-bold mb-8">Order Tracking</h1>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          <div>
            <h2 className="text-2xl font-semibold mb-4">Your Orders</h2>
            {orders.length === 0 ? (
              <p className="text-gray-400">No orders found</p>
            ) : (
              <div className="space-y-4">
                {orders.map((order) => (
                  <motion.div
                    key={order.id}
                    whileHover={{ scale: 1.02 }}
                    className="bg-zinc-900 p-6 rounded-lg cursor-pointer border border-[rgba(var(--accent),0.30)] hover:border-[rgb(var(--accent))]"
                    onClick={() => loadOrderTracking(order.id)}
                  >
                    <div className="flex justify-between items-start mb-2">
                      <div>
                        <p className="text-sm text-gray-400">Order #{order.id.slice(0, 8)}</p>
                        <p className="text-xl font-semibold">${order.total_amount}</p>
                      </div>
                      <span className={`px-3 py-1 rounded-full text-sm ${getStatusColor(order.status)}`}>
                        {order.status}
                      </span>
                    </div>
                    <p className="text-sm text-gray-400">{order.item_count} item</p>
                    <p className="text-xs text-gray-500 mt-2">
                      {new Date(order.created_at).toLocaleDateString()}
                    </p>
                  </motion.div>
                ))}
              </div>
            )}
          </div>

          <div>
            {selectedOrder ? (
              <div>
                <h2 className="text-2xl font-semibold mb-4">Order Details</h2>
                <div className="bg-zinc-900 p-6 rounded-lg border border-[rgba(var(--accent),0.30)]">
                  <div className="mb-6">
                    <p className="text-sm text-gray-400">Order ID</p>
                    <p className="text-lg font-mono">{selectedOrder.order_id}</p>
                  </div>

                  <div className="mb-6">
                    <p className="text-sm text-gray-400 mb-2">Current Status</p>
                    <span className={`px-4 py-2 rounded-full ${getStatusColor(selectedOrder.status)}`}>
                      {selectedOrder.status}
                    </span>
                  </div>

                  <div>
                    <p className="text-sm text-gray-400 mb-4">Status History</p>
                    <div className="space-y-4">
                      {selectedOrder.history.map((item, index) => (
                        <div key={index} className="flex gap-4">
                          <div className="flex flex-col items-center">
                            <div className={`w-3 h-3 rounded-full ${getStatusColor(item.status)}`} />
                            {index < selectedOrder.history.length - 1 && (
                              <div className="w-0.5 h-full bg-zinc-700 mt-2" />
                            )}
                          </div>
                          <div className="flex-1 pb-4">
                            <p className="font-semibold">{item.status}</p>
                            {item.notes && <p className="text-sm text-gray-400">{item.notes}</p>}
                            <p className="text-xs text-gray-500 mt-1">
                              {new Date(item.created_at).toLocaleString()}
                            </p>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            ) : (
              <div className="bg-zinc-900 p-12 rounded-lg border border-[rgba(var(--accent),0.30)] text-center">
                <p className="text-gray-400">Select an order to view tracking details</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

