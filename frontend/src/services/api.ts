import axios from "axios";

const API_BASE = "http://127.0.0.1:8080/api";

export const searchFlights = async (query: string) => {
  return axios.get(`${API_BASE}/flights?search=${query}`);
};

export const createBooking = async (bookingData: any) => {
  return axios.post(`${API_BASE}/bookings`, bookingData);
};

export const initPayment = async (paymentData: any) => {
  return axios.post(`${API_BASE}/payments/init`, paymentData);
};

export const confirmPayment = async (confirmData: any) => {
  return axios.post(`${API_BASE}/payments/confirm`, confirmData);
};

export const getAdminData = async () => {
  return axios.get(`${API_BASE}/admin/stats`);
};
