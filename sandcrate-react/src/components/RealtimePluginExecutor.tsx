import React, { useState, useEffect, useRef } from 'react';
import { 
  PlayIcon, 
  StopIcon, 
  XMarkIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  ArrowPathIcon
} from '@heroicons/react/24/outline';

interface Plugin {
  id: string;
  name: string;
  filename: string;
  size: number;
  created_at: string;
}

interface RealtimeExecutionProps {
  plugin: Plugin | null;
  isOpen: boolean;
  onClose: () => void;
}

interface WebSocketMessage {
  type: 'connected' | 'status' | 'update' | 'result' | 'error' | 'subscribed';
  session_id?: string;
  plugin_id?: string;
  status?: string;
  message?: string;
  output?: string;
  error?: string;
  success?: boolean;
}

export const RealtimePluginExecutor: React.FC<RealtimeExecutionProps> = ({
  plugin,
  isOpen,
  onClose
}) => {
  const [isConnected, setIsConnected] = useState(false);
  const [isExecuting, setIsExecuting] = useState(false);
  const [output, setOutput] = useState<string[]>([]);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [executionStatus, setExecutionStatus] = useState<string>('idle');
  const [parameters, setParameters] = useState('');
  const [error, setError] = useState<string | null>(null);
  
  const wsRef = useRef<WebSocket | null>(null);
  const outputEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    outputEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [output]);

  useEffect(() => {
    return () => {
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, []);

  const connectWebSocket = () => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `ws://127.0.0.1:3000/ws/plugins`;
    
    console.log('Attempting to connect to WebSocket:', wsUrl);
    
    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      setIsConnected(true);
      setError(null);
      console.log('✅ WebSocket connected successfully');
      setOutput(prev => [...prev, '🔗 Connecting to WebSocket...']);
    };

    ws.onmessage = (event) => {
      console.log('📥 WebSocket message received:', event.data);
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        handleWebSocketMessage(message);
      } catch (err) {
        console.error('❌ Failed to parse WebSocket message:', err);
        setOutput(prev => [...prev, `❌ Failed to parse message: ${event.data}`]);
      }
    };

    ws.onclose = (event) => {
      setIsConnected(false);
      console.log('🔌 WebSocket disconnected:', event.code, event.reason);
      setOutput(prev => [...prev, `🔌 WebSocket disconnected (${event.code})`]);
    };

    ws.onerror = (error) => {
      console.error('❌ WebSocket error:', error);
      setError('WebSocket connection failed');
      setOutput(prev => [...prev, '❌ WebSocket connection failed']);
    };
  };

  const handleWebSocketMessage = (message: WebSocketMessage) => {
    switch (message.type) {
      case 'connected':
        setOutput(prev => [...prev, `🔗 ${message.message || 'WebSocket connected'}`]);
        break;
        
      case 'status':
        if (message.status === 'starting') {
          setExecutionStatus('starting');
          setOutput(prev => [...prev, `🚀 ${message.message || 'Plugin execution started'}`]);
        }
        break;
      
      case 'update':
        if (message.output) {
          setOutput(prev => [...prev, message.output]);
        }
        if (message.status) {
          setExecutionStatus(message.status);
        }
        break;
      
      case 'result':
        if (message.success) {
          setExecutionStatus('completed');
          setOutput(prev => [...prev, `✅ Plugin execution completed successfully`]);
          if (message.output) {
            setOutput(prev => [...prev, `📄 Final output: ${message.output}`]);
          }
        } else {
          setExecutionStatus('error');
          setError(message.error || 'Plugin execution failed');
          setOutput(prev => [...prev, `❌ Plugin execution failed: ${message.error}`]);
        }
        setIsExecuting(false);
        break;
      
      case 'error':
        setError(message.message || 'An error occurred');
        setOutput(prev => [...prev, `❌ Error: ${message.message}`]);
        break;
      
      case 'subscribed':
        setOutput(prev => [...prev, `📡 ${message.message || 'Subscribed to session updates'}`]);
        break;
    }
  };

  const executePlugin = () => {
    if (!plugin || !isConnected || !wsRef.current) return;

    // Clear previous output
    setOutput([]);
    setError(null);
    setIsExecuting(true);
    setExecutionStatus('starting');

    // Parse parameters
    let parsedParams = {};
    if (parameters.trim()) {
      try {
        parsedParams = JSON.parse(parameters);
      } catch (e) {
        setError('Invalid JSON parameters');
        setIsExecuting(false);
        return;
      }
    }

    // Send execution command
    const command = {
      command: 'execute_plugin',
      plugin_id: plugin.id,
      parameters: parsedParams,
      timeout: 30000 // 30 seconds timeout
    };

    wsRef.current.send(JSON.stringify(command));
  };

  const stopExecution = () => {
    if (wsRef.current) {
      wsRef.current.close();
      setIsExecuting(false);
      setExecutionStatus('stopped');
      setOutput(prev => [...prev, '🛑 Execution stopped by user']);
    }
  };

  const clearOutput = () => {
    setOutput([]);
    setError(null);
    setExecutionStatus('idle');
  };

  // Connect to WebSocket when modal opens
  useEffect(() => {
    if (isOpen && !isConnected) {
      console.log('Modal opened, attempting WebSocket connection...');
      connectWebSocket();
    }
  }, [isOpen, isConnected]);

  useEffect(() => {
    if (isOpen && wsRef.current) {
      const checkConnection = () => {
        if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
          setIsConnected(true);
        } else if (wsRef.current && wsRef.current.readyState === WebSocket.CLOSED) {
          setIsConnected(false);
        }
      };
      
      const interval = setInterval(checkConnection, 1000);
      return () => clearInterval(interval);
    }
  }, [isOpen]);

  const handleClose = () => {
    if (wsRef.current) {
      wsRef.current.close();
    }
    setOutput([]);
    setError(null);
    setExecutionStatus('idle');
    setIsExecuting(false);
    setParameters('');
    onClose();
  };

  if (!isOpen || !plugin) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden">
        <div className="flex items-center justify-between p-6 border-b">
          <div>
            <h2 className="text-xl font-semibold text-gray-900">Real-time Plugin Execution</h2>
            <p className="text-sm text-gray-600">{plugin.name}</p>
          </div>
          <div className="flex items-center space-x-2">
            <div className={`flex items-center px-2 py-1 rounded-full text-xs ${
              isConnected 
                ? 'bg-green-100 text-green-800' 
                : 'bg-red-100 text-red-800'
            }`}>
              <div className={`w-2 h-2 rounded-full mr-1 ${
                isConnected ? 'bg-green-500' : 'bg-red-500'
              }`} />
              {isConnected ? 'Connected' : 'Disconnected'}
            </div>
            <button
              onClick={handleClose}
              className="p-2 text-gray-400 hover:text-gray-600 rounded-md"
            >
              <XMarkIcon className="w-5 h-5" />
            </button>
          </div>
        </div>

        <div className="flex flex-col h-[calc(90vh-120px)]">
          <div className="p-6 border-b">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Parameters (JSON)
            </label>
            <textarea
              value={parameters}
              onChange={(e) => setParameters(e.target.value)}
              placeholder='{"input": "test data", "options": {"verbose": true}}'
              className="w-full h-20 p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 font-mono text-sm"
              disabled={isExecuting}
            />
          </div>

          <div className="p-6 border-b bg-gray-50">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <button
                  onClick={executePlugin}
                  disabled={!isConnected || isExecuting}
                  className="flex items-center px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <PlayIcon className="w-4 h-4 mr-2" />
                  Execute
                </button>
                <button
                  onClick={stopExecution}
                  disabled={!isExecuting}
                  className="flex items-center px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <StopIcon className="w-4 h-4 mr-2" />
                  Stop
                </button>
                <button
                  onClick={clearOutput}
                  className="flex items-center px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700"
                >
                  <ArrowPathIcon className="w-4 h-4 mr-2" />
                  Clear
                </button>
              </div>
              
              <div className="flex items-center space-x-2">
                <span className="text-sm text-gray-600">Status:</span>
                <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                  executionStatus === 'idle' ? 'bg-gray-100 text-gray-800' :
                  executionStatus === 'starting' ? 'bg-yellow-100 text-yellow-800' :
                  executionStatus === 'running' ? 'bg-blue-100 text-blue-800' :
                  executionStatus === 'completed' ? 'bg-green-100 text-green-800' :
                  executionStatus === 'error' ? 'bg-red-100 text-red-800' :
                  'bg-gray-100 text-gray-800'
                }`}>
                  {executionStatus}
                </span>
              </div>
            </div>
          </div>

          <div className="flex-1 p-6 overflow-hidden">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-medium text-gray-900">Real-time Output</h3>
              <span className="text-sm text-gray-500">{output.length} lines</span>
            </div>
            
            <div className="bg-black text-green-400 p-4 rounded-md h-full overflow-y-auto font-mono text-sm">
              {output.length === 0 ? (
                <div className="text-gray-500 italic">
                  No output yet. Click "Execute" to start the plugin...
                </div>
              ) : (
                output.map((line, index) => (
                  <div key={index} className="mb-1">
                    <span className="text-gray-500">[{new Date().toLocaleTimeString()}]</span> {line}
                  </div>
                ))
              )}
              <div ref={outputEndRef} />
            </div>
          </div>

          {error && (
            <div className="p-6 border-t bg-red-50">
              <div className="flex items-center">
                <ExclamationTriangleIcon className="w-5 h-5 text-red-600 mr-2" />
                <span className="text-red-800 font-medium">Error:</span>
                <span className="text-red-700 ml-2">{error}</span>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}; 