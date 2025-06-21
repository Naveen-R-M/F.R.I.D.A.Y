import React, { useState, useEffect } from 'react';
import { useAuth } from '../../contexts/AuthContext';
import { AccessGrant } from '@ocv/sdk';

const GrantsPage: React.FC = () => {
  const { user, client } = useAuth();
  const [grants, setGrants] = useState<AccessGrant[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchGrants = async () => {
      if (!user) return;
      
      try {
        setIsLoading(true);
        const activeGrants = await client.consent.getActiveGrants(user.id);
        setGrants(activeGrants);
        setError(null);
      } catch (err) {
        console.error('Failed to fetch grants:', err);
        setError('Failed to load your access grants. Please try again.');
      } finally {
        setIsLoading(false);
      }
    };

    fetchGrants();
  }, [user, client]);

  const handleRevokeGrant = async (grantId: string) => {
    if (!user) return;
    
    try {
      await client.consent.revokeAccess(grantId, user.id, client.options.clientId);
      // Update the list
      setGrants(grants.filter(grant => grant.id !== grantId));
    } catch (err) {
      console.error('Failed to revoke grant:', err);
      setError('Failed to revoke access. Please try again.');
    }
  };

  if (isLoading) {
    return (
      <div className="text-center py-12">
        <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-indigo-600 border-r-transparent"></div>
        <p className="mt-4 text-gray-700">Loading access grants...</p>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Access Grants</h1>
      </div>

      {error && (
        <div className="rounded-md bg-red-50 p-4 mb-6">
          <div className="text-sm text-red-700">{error}</div>
        </div>
      )}

      {grants.length === 0 ? (
        <div className="bg-white shadow overflow-hidden sm:rounded-lg p-6 text-center">
          <p className="text-gray-500">You haven't granted access to any applications yet.</p>
        </div>
      ) : (
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <ul className="divide-y divide-gray-200">
            {grants.map((grant) => (
              <li key={grant.id} className="p-6">
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-lg font-medium text-gray-900">{grant.clientId}</h3>
                    <div className="mt-1 text-sm text-gray-500">
                      <p>Granted on: {new Date(grant.createdAt).toLocaleDateString()}</p>
                      {grant.expiresAt && (
                        <p>Expires on: {new Date(grant.expiresAt).toLocaleDateString()}</p>
                      )}
                    </div>
                    <div className="mt-2">
                      <span className="text-sm font-medium text-gray-700">Access to:</span>
                      <div className="mt-1 flex flex-wrap gap-2">
                        {grant.contextDomains.map((domain) => (
                          <span
                            key={domain}
                            className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-800"
                          >
                            {domain}
                          </span>
                        ))}
                      </div>
                    </div>
                    <div className="mt-2">
                      <span className="text-sm font-medium text-gray-700">Permissions:</span>
                      <div className="mt-1 flex flex-wrap gap-2">
                        {grant.scopes.map((scope) => (
                          <span
                            key={scope}
                            className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800"
                          >
                            {scope}
                          </span>
                        ))}
                      </div>
                    </div>
                  </div>
                  <div>
                    <button
                      type="button"
                      onClick={() => handleRevokeGrant(grant.id)}
                      className="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-red-600 hover:bg-red-700"
                    >
                      Revoke Access
                    </button>
                  </div>
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
};

export default GrantsPage;
