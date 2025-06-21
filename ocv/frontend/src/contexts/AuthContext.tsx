import React, { createContext, useContext, useState, useEffect } from 'react';
import { OcvClient } from '@ocv/sdk';
import { User, AuthToken } from '@ocv/sdk';

interface AuthContextType {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  client: OcvClient;
}

const defaultClient = new OcvClient({
  endpoint: process.env.REACT_APP_API_URL || 'http://localhost:8000/api/graphql',
  clientId: 'ocv-consent-ui',
});

const AuthContext = createContext<AuthContextType>({
  user: null,
  token: null,
  isAuthenticated: false,
  login: async () => {},
  logout: () => {},
  client: defaultClient,
});

export const useAuth = () => useContext(AuthContext);

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [client] = useState<OcvClient>(defaultClient);
  
  // Initialize from localStorage
  useEffect(() => {
    const storedToken = localStorage.getItem('authToken');
    const storedUser = localStorage.getItem('user');
    
    if (storedToken && storedUser) {
      setToken(storedToken);
      setUser(JSON.parse(storedUser));
      client.setToken(storedToken);
    }
  }, [client]);
  
  const login = async (email: string, password: string) => {
    const authResult = await client.identity.login({ email, password });
    
    if (authResult) {
      setUser(authResult.user);
      setToken(authResult.token);
      
      // Store in localStorage
      localStorage.setItem('authToken', authResult.token);
      localStorage.setItem('user', JSON.stringify(authResult.user));
      
      // Update client
      client.setToken(authResult.token);
    } else {
      throw new Error('Authentication failed');
    }
  };
  
  const logout = () => {
    setUser(null);
    setToken(null);
    
    // Clear localStorage
    localStorage.removeItem('authToken');
    localStorage.removeItem('user');
    
    // Update client
    client.clearToken();
  };
  
  return (
    <AuthContext.Provider
      value={{
        user,
        token,
        isAuthenticated: !!user,
        login,
        logout,
        client,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};
