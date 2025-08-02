import React, { useState, useEffect } from 'react';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { RealtimePluginExecutor } from '../components/RealtimePluginExecutor';
import { useAuth } from '../contexts/AuthContext';
import { 
  MagnifyingGlassIcon,
  PuzzlePieceIcon,
  PlayIcon,
  XMarkIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  BoltIcon,
  ArrowUpTrayIcon,
  TrashIcon
} from '@heroicons/react/24/outline';

interface Plugin {
  id: string;
  name: string;
  filename: string;
  size: number;
  created_at: string;
}

interface ExecutionResult {
  success: boolean;
  result: string;
  execution_time_ms: number;
  error?: string;
}

interface ExecutionModalProps {
  plugin: Plugin | null;
  isOpen: boolean;
  onClose: () => void;
  onExecute: (pluginId: string, parameters: any) => Promise<ExecutionResult>;
}

const ExecutionModal: React.FC<ExecutionModalProps> = ({ plugin, isOpen, onClose, onExecute }) => {
  const [parameters, setParameters] = useState('');
  const [isExecuting, setIsExecuting] = useState(false);
  const [result, setResult] = useState<ExecutionResult | null>(null);

  const handleExecute = async () => {
    if (!plugin) return;
    
    setIsExecuting(true);
    setResult(null);
    
    try {
      let parsedParams = {};
      if (parameters.trim()) {
        try {
          parsedParams = JSON.parse(parameters);
        } catch (e) {
          throw new Error('Invalid JSON parameters');
        }
      }
      
      const result = await onExecute(plugin.id, parsedParams);
      setResult(result);
    } catch (error) {
      setResult({
        success: false,
        result: '',
        execution_time_ms: 0,
        error: error instanceof Error ? error.message : 'Execution failed'
      });
    } finally {
      setIsExecuting(false);
    }
  };

  const handleClose = () => {
    setParameters('');
    setResult(null);
    onClose();
  };

  if (!isOpen || !plugin) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-hidden">
        <div className="flex items-center justify-between p-6 border-b">
          <div>
            <h2 className="text-xl font-semibold text-gray-900">Execute Plugin</h2>
            <p className="text-sm text-gray-600">{plugin.name}</p>
          </div>
          <button
            onClick={handleClose}
            className="p-2 text-gray-400 hover:text-gray-600 rounded-md"
          >
            <XMarkIcon className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Parameters (JSON)
            </label>
            <textarea
              value={parameters}
              onChange={(e) => setParameters(e.target.value)}
              placeholder='{"input": "test data", "options": {"verbose": true}}'
              className="w-full h-24 p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 font-mono text-sm"
              disabled={isExecuting}
            />
            <p className="text-xs text-gray-500 mt-1">
              Optional: Enter JSON parameters to pass to the plugin
            </p>
          </div>

          <div className="flex justify-end">
            <button
              onClick={handleExecute}
              disabled={isExecuting}
              className="flex items-center px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isExecuting ? (
                <>
                  <LoadingSpinner size="sm" />
                  <span className="ml-2">Executing...</span>
                </>
              ) : (
                <>
                  <PlayIcon className="w-4 h-4 mr-2" />
                  Execute Plugin
                </>
              )}
            </button>
          </div>

          {result && (
            <div className="border rounded-md p-4">
              <div className="flex items-center mb-3">
                {result.success ? (
                  <CheckCircleIcon className="w-5 h-5 text-green-600 mr-2" />
                ) : (
                  <ExclamationTriangleIcon className="w-5 h-5 text-red-600 mr-2" />
                )}
                <h3 className="font-medium">
                  {result.success ? 'Execution Successful' : 'Execution Failed'}
                </h3>
              </div>
              
              {result.success && (
                <div className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500">Execution Time:</span>
                    <span className="font-medium">{result.execution_time_ms}ms</span>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Output:
                    </label>
                    <pre className="bg-gray-50 p-3 rounded-md text-sm font-mono overflow-x-auto">
                      {result.result}
                    </pre>
                  </div>
                </div>
              )}
              
              {result.error && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Error:
                  </label>
                  <pre className="bg-red-50 p-3 rounded-md text-sm font-mono text-red-700 overflow-x-auto">
                    {result.error}
                  </pre>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export const Plugins: React.FC = () => {
  const { user } = useAuth();
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [filteredPlugins, setFilteredPlugins] = useState<Plugin[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedPlugin, setSelectedPlugin] = useState<Plugin | null>(null);
  const [isExecutionModalOpen, setIsExecutionModalOpen] = useState(false);
  const [isRealtimeModalOpen, setIsRealtimeModalOpen] = useState(false);
  const [isUploadModalOpen, setIsUploadModalOpen] = useState(false);
  const [executionHistory, setExecutionHistory] = useState<Map<string, ExecutionResult[]>>(new Map());

  useEffect(() => {
    fetchPlugins();
  }, []);

  useEffect(() => {
    filterPlugins();
  }, [plugins, searchTerm]);

  const fetchPlugins = async () => {
    try {
      setLoading(true);
      
      const token = localStorage.getItem('authToken');
      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
      };
      
      if (token) {
        headers['Authorization'] = `Bearer ${token}`;
      }
      
      const response = await fetch('/api/plugins', {
        headers,
      });
      
      if (!response.ok) {
        throw new Error('Failed to fetch plugins');
      }
      
      const data = await response.json();
      if (data.success && data.data) {
        setPlugins(data.data.plugins);
      } else {
        throw new Error(data.error || 'Failed to fetch plugins');
      }
      
    } catch (err) {
      console.error('Plugins fetch error:', err);
      setError('Failed to load plugins');
    } finally {
      setLoading(false);
    }
  };

  const filterPlugins = () => {
    let filtered = plugins || [];

    if (searchTerm) {
      filtered = filtered.filter(plugin =>
        plugin.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        plugin.filename.toLowerCase().includes(searchTerm.toLowerCase())
      );
    }

    setFilteredPlugins(filtered);
  };

  const executePlugin = async (pluginId: string, parameters: any): Promise<ExecutionResult> => {
    try {
      const token = localStorage.getItem('authToken');
      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
      };
      
      if (token) {
        headers['Authorization'] = `Bearer ${token}`;
      }
      
      const response = await fetch(`/api/plugins/${pluginId}/execute`, {
        method: 'POST',
        headers,
        body: JSON.stringify({
          parameters,
          timeout: 10000
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      
      if (!data.success || !data.data) {
        throw new Error(data.error || 'Execution failed');
      }
      
      const result: ExecutionResult = data.data;
      
      const history = executionHistory.get(pluginId) || [];
      history.unshift(result);
      setExecutionHistory(new Map(executionHistory.set(pluginId, history.slice(0, 5))));
      
      return result;
    } catch (error) {
      console.error('Plugin execution error:', error);
      throw error;
    }
  };

  const handleExecutePlugin = (plugin: Plugin) => {
    setSelectedPlugin(plugin);
    setIsExecutionModalOpen(true);
  };

  const handleRealtimeExecutePlugin = (plugin: Plugin) => {
    setSelectedPlugin(plugin);
    setIsRealtimeModalOpen(true);
  };

  const handleUploadPlugin = async (file: File) => {
    try {
      const token = localStorage.getItem('authToken');
      const formData = new FormData();
      formData.append('plugin', file);
      
      const response = await fetch('/api/plugins/upload', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
        },
        body: formData,
      });

      if (!response.ok) {
        throw new Error('Failed to upload plugin');
      }

      const data = await response.json();
      if (data.success) {
        await fetchPlugins();
        setIsUploadModalOpen(false);
      } else {
        throw new Error(data.error || 'Upload failed');
      }
    } catch (error) {
      console.error('Upload error:', error);
      setError('Failed to upload plugin');
    }
  };

  const handleDeletePlugin = async (pluginId: string) => {
    if (!confirm(`Are you sure you want to delete plugin '${pluginId}'?`)) {
      return;
    }

    try {
      const token = localStorage.getItem('authToken');
      const response = await fetch(`/api/plugins/${pluginId}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to delete plugin');
      }

      const data = await response.json();
      if (data.success) {
        await fetchPlugins();
      } else {
        throw new Error(data.error || 'Delete failed');
      }
    } catch (error) {
      console.error('Delete error:', error);
      setError('Failed to delete plugin');
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getLastExecutionResult = (pluginId: string): ExecutionResult | null => {
    const history = executionHistory.get(pluginId);
    return history && history.length > 0 ? history[0] : null;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Plugins</h1>
          <p className="text-gray-600">WASM plugins from assets/plugins directory</p>
        </div>
        <button
          onClick={() => setIsUploadModalOpen(true)}
          className="flex items-center px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 transition-colors"
        >
          <ArrowUpTrayIcon className="w-4 h-4 mr-2" />
          Upload Plugin
        </button>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-md p-4">
          <p className="text-red-600">{error}</p>
        </div>
      )}

      <div className="card">
        <div className="flex flex-col md:flex-row gap-4">
          <div className="flex-1">
            <div className="relative">
              <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
              <input
                type="text"
                placeholder="Search plugins..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500"
              />
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {(filteredPlugins || []).map((plugin) => {
          const lastExecution = getLastExecutionResult(plugin.id);
          
          return (
            <div key={plugin.id} className="card hover:shadow-lg transition-shadow">
              <div className="flex items-start justify-between mb-4">
                <div className="flex-1">
                  <h3 className="text-lg font-semibold text-gray-900 mb-1">{plugin.name}</h3>
                  <p className="text-sm text-gray-600 mb-2">WASM Plugin</p>
                  <div className="flex items-center space-x-2">
                    <span className="inline-flex px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800">
                      Active
                    </span>
                    {lastExecution && (
                      <span className={`inline-flex px-2 py-1 text-xs font-medium rounded-full ${
                        lastExecution.success 
                          ? 'bg-green-100 text-green-800' 
                          : 'bg-red-100 text-red-800'
                      }`}>
                        {lastExecution.success ? 'Last Run: Success' : 'Last Run: Failed'}
                      </span>
                    )}
                  </div>
                </div>
              </div>

              <div className="space-y-2 mb-4">
                <div className="flex justify-between text-sm">
                  <span className="text-gray-500">Filename:</span>
                  <span className="font-medium">{plugin.filename}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-500">Size:</span>
                  <span className="font-medium">{formatFileSize(plugin.size)}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-500">Created:</span>
                  <span className="font-medium">{plugin.created_at}</span>
                </div>
                {lastExecution && (
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500">Last Execution:</span>
                    <span className="font-medium">{lastExecution.execution_time_ms}ms</span>
                  </div>
                )}
              </div>

              <div className="flex items-center justify-between pt-4 border-t border-gray-200">
                <div className="flex space-x-2">
                  <button
                    onClick={() => handleExecutePlugin(plugin)}
                    className="flex items-center px-3 py-1 bg-green-600 text-white text-sm rounded-md hover:bg-green-700 transition-colors"
                    title="Execute Plugin"
                  >
                    <PlayIcon className="w-4 h-4 mr-1" />
                    Execute
                  </button>
                  <button
                    onClick={() => handleRealtimeExecutePlugin(plugin)}
                    className="flex items-center px-3 py-1 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors"
                    title="Real-time Execution"
                  >
                    <BoltIcon className="w-4 h-4 mr-1" />
                    Real-time
                  </button>
                  <button
                    onClick={() => {}}
                    className="p-2 text-blue-600 hover:bg-blue-50 rounded-md transition-colors"
                    title="View Details"
                  >
                    <PuzzlePieceIcon className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => handleDeletePlugin(plugin.id)}
                    className="p-2 text-red-600 hover:bg-red-50 rounded-md transition-colors"
                    title="Delete Plugin"
                  >
                    <TrashIcon className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {(filteredPlugins || []).length === 0 && (
        <div className="text-center py-12">
          <PuzzlePieceIcon className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900 mb-2">No plugins found</h3>
          <p className="text-gray-500">
            {searchTerm
              ? 'Try adjusting your search terms'
              : 'No WASM plugins found in assets/plugins directory'
            }
          </p>
        </div>
      )}

      <ExecutionModal
        plugin={selectedPlugin}
        isOpen={isExecutionModalOpen}
        onClose={() => setIsExecutionModalOpen(false)}
        onExecute={executePlugin}
      />

      <RealtimePluginExecutor
        plugin={selectedPlugin}
        isOpen={isRealtimeModalOpen}
        onClose={() => setIsRealtimeModalOpen(false)}
      />

      {isUploadModalOpen && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-md w-full mx-4">
            <div className="flex items-center justify-between p-6 border-b">
              <h2 className="text-xl font-semibold text-gray-900">Upload Plugin</h2>
              <button
                onClick={() => setIsUploadModalOpen(false)}
                className="p-2 text-gray-400 hover:text-gray-600 rounded-md"
              >
                <XMarkIcon className="w-5 h-5" />
              </button>
            </div>
            <div className="p-6">
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Select WASM Plugin File
                  </label>
                  <input
                    type="file"
                    accept=".wasm"
                    onChange={(e) => {
                      const file = e.target.files?.[0];
                      if (file) {
                        handleUploadPlugin(file);
                      }
                    }}
                    className="w-full p-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500"
                  />
                </div>
                <p className="text-sm text-gray-600">
                  Upload a compiled WASM plugin file. The file will be saved to the assets/plugins directory.
                </p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}; 