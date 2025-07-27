import { useState } from 'react';
import { Terminal, Shield, Eye, EyeOff, ArrowRight } from 'lucide-react';

interface AuthProps {
  onLogin: () => void;
}

const Auth = ({ onLogin }: AuthProps) => {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    
    // Simulate API call
    setTimeout(() => {
      if (username === 'admin' && password === 'password') {
        onLogin();
      }
      setIsLoading(false);
    }, 1000);
  };

  return (
    <div className="min-h-screen bg-terminal-900 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        {/* Terminal Header */}
        <div className="bg-terminal-800 rounded-t-lg border border-terminal-700 p-3">
          <div className="flex items-center space-x-2">
            <div className="flex space-x-1">
              <div className="w-3 h-3 bg-red-500 rounded-full"></div>
              <div className="w-3 h-3 bg-yellow-500 rounded-full"></div>
              <div className="w-3 h-3 bg-devops-green-500 rounded-full"></div>
            </div>
            <div className="flex-1 text-center">
              <span className="text-sm text-terminal-400">SandCrate Terminal - Authentication</span>
            </div>
          </div>
        </div>

        {/* Login Form */}
        <div className="bg-terminal-800 border border-terminal-700 rounded-b-lg p-6">
          <div className="text-center mb-6">
            <div className="flex items-center justify-center space-x-2 mb-4">
              <Terminal className="h-8 w-8 text-devops-green-400" />
              <Shield className="h-8 w-8 text-devops-green-400" />
            </div>
            <h1 className="text-2xl font-bold text-devops-green-400 mb-2">SandCrate</h1>
            <p className="text-terminal-400">DevOps Management Platform</p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-terminal-300 mb-2">
                Username
              </label>
              <div className="relative">
                <input
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="w-full bg-terminal-700 border border-terminal-600 rounded-lg px-4 py-3 text-terminal-100 placeholder-terminal-400 focus:outline-none focus:ring-2 focus:ring-devops-green-500 focus:border-transparent"
                  placeholder="Enter username"
                  required
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-terminal-300 mb-2">
                Password
              </label>
              <div className="relative">
                <input
                  type={showPassword ? 'text' : 'password'}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="w-full bg-terminal-700 border border-terminal-600 rounded-lg px-4 py-3 pr-12 text-terminal-100 placeholder-terminal-400 focus:outline-none focus:ring-2 focus:ring-devops-green-500 focus:border-transparent"
                  placeholder="Enter password"
                  required
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-3 top-1/2 transform -translate-y-1/2 text-terminal-400 hover:text-terminal-300"
                >
                  {showPassword ? <EyeOff className="h-5 w-5" /> : <Eye className="h-5 w-5" />}
                </button>
              </div>
            </div>

            <button
              type="submit"
              disabled={isLoading}
              className="w-full bg-devops-green-600 hover:bg-devops-green-700 disabled:bg-terminal-600 text-white font-medium py-3 px-4 rounded-lg transition-colors duration-200 flex items-center justify-center space-x-2"
            >
              {isLoading ? (
                <>
                  <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                  <span>Authenticating...</span>
                </>
              ) : (
                <>
                  <span>Login</span>
                  <ArrowRight className="h-4 w-4" />
                </>
              )}
            </button>
          </form>

          {/* Demo Credentials */}
          <div className="mt-6 p-4 bg-terminal-700 rounded-lg border border-terminal-600">
            <h3 className="text-sm font-medium text-terminal-300 mb-2">Demo Credentials</h3>
            <div className="space-y-1 text-xs">
              <div className="flex justify-between">
                <span className="text-terminal-400">Username:</span>
                <span className="text-devops-green-400 font-mono">admin</span>
              </div>
              <div className="flex justify-between">
                <span className="text-terminal-400">Password:</span>
                <span className="text-devops-green-400 font-mono">password</span>
              </div>
            </div>
          </div>

          {/* Terminal Footer */}
          <div className="mt-6 text-center">
            <div className="inline-flex items-center space-x-2 text-xs text-terminal-500">
              <div className="w-2 h-2 bg-devops-green-400 rounded-full animate-pulse"></div>
              <span>System Ready</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Auth; 