import React from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { useAuth } from './contexts/AuthContext';

// Layout
import MainLayout from './components/layout/MainLayout';

// Pages
import LoginPage from './pages/LoginPage';
import RegisterPage from './pages/RegisterPage';
import DashboardPage from './pages/DashboardPage';
import GrantsPage from './pages/GrantsPage';
import AuditLogPage from './pages/AuditLogPage';
import ConsentRequestPage from './pages/ConsentRequestPage';

const App: React.FC = () => {
  const { isAuthenticated } = useAuth();

  return (
    <Routes>
      {/* Public routes */}
      <Route path="/login" element={<LoginPage />} />
      <Route path="/register" element={<RegisterPage />} />
      <Route path="/consent-request/:requestId" element={<ConsentRequestPage />} />
      
      {/* Protected routes */}
      <Route path="/" element={
        isAuthenticated ? <MainLayout /> : <Navigate to="/login" />
      }>
        <Route index element={<DashboardPage />} />
        <Route path="grants" element={<GrantsPage />} />
        <Route path="audit-log" element={<AuditLogPage />} />
      </Route>
    </Routes>
  );
};

export default App;
