import React, { useState, useEffect } from 'react';
import { useAuth } from '../../contexts/AuthContext';
import { ContextShard } from '@ocv/sdk';

const DashboardPage: React.FC = () => {
  const { user, client } = useAuth();
  const [contextShards, setContextShards] = useState<ContextShard[]>([]);
  const [domainStats, setDomainStats] = useState<{ domain: string; count: number }[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      if (!user) return;
      
      try {
        setIsLoading(true);
        
        // This is a simplified version. In a real app, you'd have a dedicated API for stats
        const allShards = await client.context.searchShards(user.id, '', undefined, 1000);
        setContextShards(allShards);
        
        // Calculate domain statistics
        const domains = allShards.reduce<Record<string, number>>((acc, shard) => {
          acc[shard.domain] = (acc[shard.domain] || 0) + 1;
          return acc;
        }, {});
        
        const stats = Object.entries(domains).map(([domain, count]) => ({
          domain,
          count,
        }));
        
        setDomainStats(stats.sort((a, b) => b.count - a.count));
        setError(null);
      } catch (err) {
        console.error('Failed to fetch dashboard data:', err);
        setError('Failed to load dashboard data. Please try again.');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, [user, client]);

  if (isLoading) {
    return (
      <div className="text-center py-12">
        <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-indigo-600 border-r-transparent"></div>
        <p className="mt-4 text-gray-700">Loading dashboard data...</p>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
      </div>

      {error && (
        <div className="rounded-md bg-red-50 p-4 mb-6">
          <div className="text-sm text-red-700">{error}</div>
        </div>
      )}

      <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
        {/* Summary Card */}
        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg font-medium text-gray-900">Context Summary</h3>
            <div className="mt-5">
              <dl className="grid grid-cols-1 gap-5">
                <div className="bg-gray-50 px-4 py-5 sm:p-6 rounded-lg">
                  <dt className="text-sm font-medium text-gray-500">Total Shards</dt>
                  <dd className="mt-1 text-3xl font-semibold text-indigo-600">
                    {contextShards.length}
                  </dd>
                </div>
                <div className="bg-gray-50 px-4 py-5 sm:p-6 rounded-lg">
                  <dt className="text-sm font-medium text-gray-500">Domains</dt>
                  <dd className="mt-1 text-3xl font-semibold text-indigo-600">
                    {domainStats.length}
                  </dd>
                </div>
              </dl>
            </div>
          </div>
        </div>

        {/* Domain Distribution Card */}
        <div className="bg-white overflow-hidden shadow rounded-lg md:col-span-2">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg font-medium text-gray-900">Domain Distribution</h3>
            <div className="mt-5">
              {domainStats.length === 0 ? (
                <p className="text-gray-500 text-center py-4">No context shards found.</p>
              ) : (
                <ul className="divide-y divide-gray-200">
                  {domainStats.map((stat) => (
                    <li key={stat.domain} className="py-3 flex justify-between items-center">
                      <div className="flex items-center">
                        <span className="h-4 w-4 rounded-full bg-indigo-600 mr-3"></span>
                        <span className="text-sm font-medium text-gray-900">{stat.domain}</span>
                      </div>
                      <div className="text-sm text-gray-500">{stat.count} shards</div>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </div>
        </div>

        {/* Recent Updates Card */}
        <div className="bg-white overflow-hidden shadow rounded-lg md:col-span-3">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg font-medium text-gray-900">Recent Context Updates</h3>
            <div className="mt-5">
              {contextShards.length === 0 ? (
                <p className="text-gray-500 text-center py-4">No context shards found.</p>
              ) : (
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Domain
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Type
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Updated
                        </th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {contextShards
                        .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
                        .slice(0, 5)
                        .map((shard) => (
                          <tr key={shard.id}>
                            <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                              {shard.domain}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                              {shard.contentType}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                              {new Date(shard.updatedAt).toLocaleString()}
                            </td>
                          </tr>
                        ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default DashboardPage;
