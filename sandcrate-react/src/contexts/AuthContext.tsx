import React, { createContext, useContext, useState, useEffect } from 'react';
import type { ReactNode } from 'react';

interface User {
  id: string;
  username: string;
  name: string;
  role: string;
  is_admin: boolean;
}

interface AuthContextType {
  user: User | null;
  loading: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

interface AuthProviderProps {
  children: ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Check for existing session on app load
    const checkAuth = async () => {
      try {
        const token = localStorage.getItem('authToken');
        console.log('Checking auth, token exists:', !!token);
        
        if (token) {
          // Validate token with Rust backend
          const response = await fetch('/auth/validate', {
            headers: {
              'Authorization': `Bearer ${token}`,
            },
          });

          console.log('Validation response status:', response.status);

          if (response.ok) {
            const data = await response.json();
            console.log('User data received:', data);
            setUser(data);
          } else {
            console.log('Token validation failed, clearing storage');
            // Token is invalid, clear storage
            localStorage.removeItem('authToken');
            localStorage.removeItem('user');
          }
        }
      } catch (error) {
        console.error('Auth check failed:', error);
        // Clear invalid tokens on error
        localStorage.removeItem('authToken');
        localStorage.removeItem('user');
      } finally {
        setLoading(false);
      }
    };

    checkAuth();
  }, []);

  const login = async (username: string, password: string) => {
    setLoading(true);
    try {
      // Call Rust backend API for PAM Linux authentication
      const response = await fetch('/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          username: username,
          password: password,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || 'Authentication failed');
      }

      const data = await response.json();
      
      // Store authentication token and user data
      localStorage.setItem('authToken', data.token);
      localStorage.setItem('user', JSON.stringify(data.user));
      setUser(data.user);
      
      // Login successful - user will be redirected to dashboard
    } catch (error) {
      console.error('Login failed:', error);
      throw new Error(error instanceof Error ? error.message : 'Login failed. Please check your credentials.');
    } finally {
      setLoading(false);
    }
  };

  const logout = () => {
    localStorage.removeItem('authToken');
    localStorage.removeItem('user');
    setUser(null);
  };

  const value: AuthContextType = {
    user,
    loading,
    login,
    logout
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
}; 