import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { OcvClient } from '@ocv/sdk';

interface ConsentRequest {
  requestId: string;
  clientId: string;
  clientName: string;
  scopes: string[];
  domains: string[];
  userId: string;
  redirectUri: string;
}

const ConsentRequestPage: React.FC = () => {
  const { requestId } = useParams<{ requestId: string }>();
  const navigate = useNavigate();
  const [request, setRequest] = useState<ConsentRequest | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    // In a real implementation, we would fetch the consent request details
    // For this example, we'll use mock data
    const mockRequest: ConsentRequest = {
      requestId: requestId || 'unknown',
      clientId: 'travel-demo-app',
      clientName: 'Travel Demo Application',
      scopes: ['read', 'write'],
      domains: ['travel-preferences', 'travel-history'],
      userId: '', // Will be filled after authentication
      redirectUri: 'http://localhost:3001/callback',
    };
    
    setRequest(mockRequest);
    setIsLoading(false);
  }, [requestId]);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!request) return;
    
    setIsSubmitting(true);
    setError(null);
    
    try {
      // Create a temporary client for authentication
      const client = new OcvClient({
        endpoint: process.env.REACT_APP_API_URL || 'http://localhost:8000/api/graphql',
        clientId: 'ocv-consent-ui',
      });
      
      // Authenticate the user
      const authResult = await client.identity.login({ email, password });
      
      if (!authResult) {
        throw new Error('Authentication failed');
      }
      
      // Now we have the user ID, update the request
      const updatedRequest = {
        ...request,
        userId: authResult.user.id,
      };
      
      // Grant access
      await client.consent.grantAccess({
        userId: authResult.user.id,
        clientId: updatedRequest.clientId,
        scopes: updatedRequest.scopes,
        contextDomains: updatedRequest.domains,
        expiresAt: undefined, // No expiration
      });
      
      // Redirect back to the application
      window.location.href = `${updatedRequest.redirectUri}?requestId=${updatedRequest.requestId}&granted=true`;
      
    } catch (err) {
      console.error('Consent flow error:', err);
      setError('Authentication failed. Please check your credentials and try again.');
      setIsSubmitting(false);
    }
  };

  const handleDeny = () => {
    if (!request) return;
    
    // Redirect back to the application with granted=false
    window.location.href = `${request.redirectUri}?requestId=${request.requestId}&granted=false`;
  };

  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gray-50">
        <div className="text-center">
          <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-indigo-600 border-r-transparent"></div>
          <p className="mt-4 text-gray-700">Loading request...</p>
        </div>
      </div>
    );
  }

  if (!request) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gray-50">
        <div className="text-center">
          <h2 className="text-xl font-bold text-red-600">Invalid Request</h2>
          <p className="mt-2 text-gray-600">The consent request is invalid or has expired.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div className="w-full max-w-md">
        <div className="text-center">
          <h2 className="text-3xl font-bold tracking-tight text-gray-900">
            Authorization Request
          </h2>
          <p className="mt-2 text-gray-600">
            <span className="font-semibold">{request.clientName}</span> is requesting access to your data
          </p>
        </div>

        <div className="mt-8 bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10">
          {error && (
            <div className="rounded-md bg-red-50 p-4 mb-6">
              <div className="text-sm text-red-700">{error}</div>
            </div>
          )}

          <div className="space-y-6">
            <div>
              <h3 className="text-lg font-medium text-gray-900">Requested Permissions</h3>
              <div className="mt-2 rounded-md bg-gray-50 p-4">
                <div className="text-sm text-gray-700">
                  <p className="font-medium mb-2">This app wants to:</p>
                  <ul className="list-disc pl-5 space-y-1">
                    {request.scopes.includes('read') && (
                      <li>Read your personal context data</li>
                    )}
                    {request.scopes.includes('write') && (
                      <li>Write to your personal context data</li>
                    )}
                  </ul>
                </div>
              </div>
            </div>

            <div>
              <h3 className="text-lg font-medium text-gray-900">Data Categories</h3>
              <div className="mt-2">
                <div className="flex flex-wrap gap-2">
                  {request.domains.map((domain) => (
                    <span
                      key={domain}
                      className="inline-flex items-center rounded-full bg-indigo-100 px-3 py-0.5 text-sm font-medium text-indigo-800"
                    >
                      {domain.replace(/-/g, ' ')}
                    </span>
                  ))}
                </div>
              </div>
            </div>

            <form className="space-y-6" onSubmit={handleLogin}>
              <div>
                <label htmlFor="email" className="block text-sm font-medium text-gray-700">
                  Email address
                </label>
                <div className="mt-1">
                  <input
                    id="email"
                    name="email"
                    type="email"
                    autoComplete="email"
                    required
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    className="block w-full appearance-none rounded-md border border-gray-300 px-3 py-2 placeholder-gray-400 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm"
                  />
                </div>
              </div>

              <div>
                <label htmlFor="password" className="block text-sm font-medium text-gray-700">
                  Password
                </label>
                <div className="mt-1">
                  <input
                    id="password"
                    name="password"
                    type="password"
                    autoComplete="current-password"
                    required
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="block w-full appearance-none rounded-md border border-gray-300 px-3 py-2 placeholder-gray-400 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm"
                  />
                </div>
              </div>

              <div className="flex justify-between space-x-4">
                <button
                  type="button"
                  onClick={handleDeny}
                  className="flex w-full justify-center rounded-md border border-gray-300 bg-white py-2 px-4 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
                >
                  Deny
                </button>
                <button
                  type="submit"
                  disabled={isSubmitting}
                  className="flex w-full justify-center rounded-md border border-transparent bg-indigo-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50"
                >
                  {isSubmitting ? 'Authorizing...' : 'Authorize'}
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ConsentRequestPage;
