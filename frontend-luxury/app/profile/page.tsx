"use client";

import { useState, useEffect } from "react";
import { api, session } from "@/lib/api";

interface Profile {
  id: string;
  email: string;
  name: string;
  role: string;
  wallet_address: string | null;
  created_at: string;
}

export default function ProfilePage() {
  const [profile, setProfile] = useState<Profile | null>(null);
  const [editing, setEditing] = useState(false);
  const [changingPassword, setChangingPassword] = useState(false);
  const [loading, setLoading] = useState(true);

  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [walletAddress, setWalletAddress] = useState("");

  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");

  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    const fetchProfile = async () => {
      const userId = session.getUserId();
      if (!userId) {
        setLoading(false);
        return;
      }

      try {
        const response = await api.profile.getById(userId);
        const data = response.data;
        setProfile(data);
        setName(data.name);
        setEmail(data.email);
        setWalletAddress(data.wallet_address || "");
      } catch {
        setError("Failed to load profile");
      } finally {
        setLoading(false);
      }
    };

    fetchProfile();
  }, []);

  const handleUpdateProfile = async () => {
    const userId = session.getUserId();
    if (!userId) return;

    try {
      const response = await api.profile.update(userId, {
        name,
        email,
        wallet_address: walletAddress,
      });

      setProfile(response.data);
      setMessage("Profile updated successfully");
      setError("");
      setEditing(false);
      setTimeout(() => setMessage(""), 3000);
    } catch (err: any) {
      setError(err?.response?.data?.error || "Failed to update profile");
      setMessage("");
    }
  };

  const handleChangePassword = async () => {
    if (newPassword !== confirmPassword) {
      setError("Passwords do not match");
      return;
    }

    const userId = session.getUserId();
    if (!userId) return;

    try {
      const response = await api.profile.updatePassword(userId, {
        current_password: currentPassword,
        new_password: newPassword,
      });

      setMessage(response.data.message || "Password changed successfully");
      setError("");
      setChangingPassword(false);
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
      setTimeout(() => setMessage(""), 3000);
    } catch (err: any) {
      setError(err?.response?.data?.error || "Failed to change password");
      setMessage("");
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-black text-white flex items-center justify-center">
        <div className="text-2xl">Loading profile...</div>
      </div>
    );
  }

  if (!profile) {
    return (
      <div className="min-h-screen bg-black text-white flex items-center justify-center">
        <div className="text-2xl">Profile not found</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-black text-white p-8">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-4xl font-bold mb-8">Profile Settings</h1>

        {message && (
          <div className="bg-green-500/20 border border-green-500 text-green-500 p-4 rounded-lg mb-6">
            {message}
          </div>
        )}

        {error && (
          <div className="bg-red-500/20 border border-red-500 text-red-500 p-4 rounded-lg mb-6">
            {error}
          </div>
        )}

        <div className="bg-zinc-900 p-8 rounded-lg border border-[rgba(var(--accent),0.30)] mb-6">
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-2xl font-semibold">Personal Information</h2>
            {!editing && (
              <button
                onClick={() => setEditing(true)}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg"
              >
                Edit Profile
              </button>
            )}
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Name</label>
              {editing ? (
                <input
                  type="text"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full bg-zinc-800 border border-[rgba(var(--accent),0.30)] rounded-lg px-4 py-2"
                />
              ) : (
                <p className="text-lg">{profile.name}</p>
              )}
            </div>

            <div>
              <label className="block text-sm text-gray-400 mb-2">Email</label>
              {editing ? (
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="w-full bg-zinc-800 border border-[rgba(var(--accent),0.30)] rounded-lg px-4 py-2"
                />
              ) : (
                <p className="text-lg">{profile.email}</p>
              )}
            </div>

            <div>
              <label className="block text-sm text-gray-400 mb-2">Wallet Address</label>
              {editing ? (
                <input
                  type="text"
                  value={walletAddress}
                  onChange={(e) => setWalletAddress(e.target.value)}
                  className="w-full bg-zinc-800 border border-[rgba(var(--accent),0.30)] rounded-lg px-4 py-2"
                  placeholder="0x..."
                />
              ) : (
                <p className="text-lg font-mono">{profile.wallet_address || "Not set"}</p>
              )}
            </div>

            <div>
              <label className="block text-sm text-gray-400 mb-2">Role</label>
              <p className="text-lg capitalize">{profile.role}</p>
            </div>

            <div>
              <label className="block text-sm text-gray-400 mb-2">Member Since</label>
              <p className="text-lg">{new Date(profile.created_at).toLocaleDateString()}</p>
            </div>
          </div>

          {editing && (
            <div className="flex gap-4 mt-6">
              <button
                onClick={handleUpdateProfile}
                className="px-6 py-2 bg-green-600 hover:bg-green-700 rounded-lg"
              >
                Save Changes
              </button>
              <button
                onClick={() => {
                  setEditing(false);
                  setName(profile.name);
                  setEmail(profile.email);
                  setWalletAddress(profile.wallet_address || "");
                }}
                className="px-6 py-2 bg-zinc-700 hover:bg-zinc-600 rounded-lg"
              >
                Cancel
              </button>
            </div>
          )}
        </div>

        <div className="bg-zinc-900 p-8 rounded-lg border border-[rgba(var(--accent),0.30)]">
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-2xl font-semibold">Security</h2>
            {!changingPassword && (
              <button
                onClick={() => setChangingPassword(true)}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg"
              >
                Change Password
              </button>
            )}
          </div>

          {changingPassword && (
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-2">Current Password</label>
                <input
                  type="password"
                  value={currentPassword}
                  onChange={(e) => setCurrentPassword(e.target.value)}
                  className="w-full bg-zinc-800 border border-[rgba(var(--accent),0.30)] rounded-lg px-4 py-2"
                />
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">New Password</label>
                <input
                  type="password"
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  className="w-full bg-zinc-800 border border-[rgba(var(--accent),0.30)] rounded-lg px-4 py-2"
                />
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">Confirm New Password</label>
                <input
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  className="w-full bg-zinc-800 border border-[rgba(var(--accent),0.30)] rounded-lg px-4 py-2"
                />
              </div>

              <div className="flex gap-4 mt-6">
                <button
                  onClick={handleChangePassword}
                  className="px-6 py-2 bg-green-600 hover:bg-green-700 rounded-lg"
                >
                  Update Password
                </button>
                <button
                  onClick={() => {
                    setChangingPassword(false);
                    setCurrentPassword("");
                    setNewPassword("");
                    setConfirmPassword("");
                  }}
                  className="px-6 py-2 bg-zinc-700 hover:bg-zinc-600 rounded-lg"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

