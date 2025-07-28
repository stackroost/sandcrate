import React from 'react';

export const Dashboard: React.FC = () => {
  return (
    <div className="p-6">
      <div className="text-center">
        <h1 className="text-3xl font-bold text-gray-900 mb-4">Welcome to SandCrate</h1>
        <p className="text-lg text-gray-600 mb-8">Secure Plugin Management Platform</p>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-4xl mx-auto">
          <div className="card text-center">
            <h3 className="text-xl font-semibold text-gray-900 mb-2">Plugin Management</h3>
            <p className="text-gray-600">Manage and monitor your WASM plugins</p>
          </div>
          
          <div className="card text-center">
            <h3 className="text-xl font-semibold text-gray-900 mb-2">System Security</h3>
            <p className="text-gray-600">PAM Linux authentication for secure access</p>
          </div>
          
          <div className="card text-center">
            <h3 className="text-xl font-semibold text-gray-900 mb-2">Real-time Monitoring</h3>
            <p className="text-gray-600">Track plugin performance and system status</p>
          </div>
        </div>
      </div>
    </div>
  );
}; 