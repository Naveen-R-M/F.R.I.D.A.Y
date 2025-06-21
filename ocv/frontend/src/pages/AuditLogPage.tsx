import React, { useState, useEffect } from 'react';
import { useAuth } from '../../contexts/AuthContext';
import { ConsentAuditLog } from '@ocv/sdk';

const AuditLogPage: React.FC = () => {
  const { user, client } = useAuth();
  const [auditLogs, setAuditLogs] = useState<ConsentAuditLog[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const pageSize = 20;

  useEffect(() => {
    const fetchAuditLogs = async () => {
      if (!user) return;
      
      try {
        setIsLoading(true);
        const logs = await client.consent.getAuditLogs(
          user.id,
          pageSize,
          page * pageSize
        );
        
        if (logs.length < pageSize) {
          setHasMore(false);
        }
        
        if (page === 0) {
          setAuditLogs(logs);
        } else {
          setAuditLogs(prev => [...prev, ...logs]);
        }
        
        setError(null);
      } catch (err) {
        console.error('Failed to fetch audit logs:', err);
        setError('Failed to load audit logs. Please try again.');
      } finally {
        setIsLoading(false);
      }
    };

    fetchAuditLogs();
  }, [user, client, page]);

  const formatDateTime = (dateStr: string) => {
    const date = new Date(dateStr);
    return `${date.toLocaleDateString()} ${date.toLocaleTimeString()}`;
  };

  const getActionColor = (action: string) => {
    switch (action) {
      case 'grant':
        return 'bg-green-100 text-green-800';
      case 'revoke':
        return 'bg-red-100 text-red-800';
      case 'access':
        return 'bg-blue-100 text-blue-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const loadMore = () => {
    if (!isLoading && hasMore) {
      setPage(page + 1);
    }
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Audit Log</h1>
      </div>

      {error && (
        <div className="rounded-md bg-red-50 p-4 mb-6">
          <div className="text-sm text-red-700">{error}</div>
        </div>
      )}

      {auditLogs.length === 0 && !isLoading ? (
        <div className="bg-white shadow overflow-hidden sm:rounded-lg p-6 text-center">
          <p className="text-gray-500">No audit log entries found.</p>
        </div>
      ) : (
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <ul className="divide-y divide-gray-200">
            {auditLogs.map((log) => (
              <li key={log.id} className="p-4">
                <div className="flex items-start">
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span
                        className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getActionColor(
                          log.action
                        )}`}
                      >
                        {log.action.toUpperCase()}
                      </span>
                      <p className="text-sm font-medium text-gray-900">
                        {log.clientId}
                      </p>
                      <p className="text-sm text-gray-500">
                        {formatDateTime(log.timestamp)}
                      </p>
                    </div>
                    <div className="mt-2 text-sm text-gray-700">
                      <pre className="bg-gray-50 p-2 rounded overflow-auto text-xs">
                        {JSON.stringify(log.details, null, 2)}
                      </pre>
                    </div>
                  </div>
                </div>
              </li>
            ))}
          </ul>

          {hasMore && (
            <div className="px-4 py-4 border-t border-gray-200 sm:px-6">
              <button
                onClick={loadMore}
                disabled={isLoading}
                className="w-full flex justify-center items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-indigo-700 bg-indigo-100 hover:bg-indigo-200 disabled:opacity-50"
              >
                {isLoading ? (
                  <>
                    <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-solid border-current border-r-transparent mr-2"></span>
                    Loading...
                  </>
                ) : (
                  'Load More'
                )}
              </button>
            </div>
          )}
        </div>
      )}

      {isLoading && page === 0 && (
        <div className="text-center py-12">
          <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-indigo-600 border-r-transparent"></div>
          <p className="mt-4 text-gray-700">Loading audit logs...</p>
        </div>
      )}
    </div>
  );
};

export default AuditLogPage;
