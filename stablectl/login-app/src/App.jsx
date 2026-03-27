import React, { useEffect, useState } from 'react';
import { PrivyProvider, usePrivy } from '@privy-io/react-auth';

// The Rust server injects this into the HTML before serving
const APP_ID = window.__STABLECTL_APP_ID__ || 'missing-app-id';

function LoginPanel() {
  const { ready, authenticated, login, logout, getAccessToken, user } = usePrivy();
  const [status, setStatus] = useState('Initializing Privy…');
  const [submitting, setSubmitting] = useState(false);
  const [done, setDone] = useState(false);

  useEffect(() => {
    if (ready && !authenticated) {
      setStatus('Ready. Click "Login with Privy" to authenticate.');
    }
  }, [ready, authenticated]);

  useEffect(() => {
    async function submitToken() {
      if (!ready || !authenticated || submitting || done) return;
      setSubmitting(true);
      try {
        setStatus('Authenticated! Fetching access token…');
        const token = await getAccessToken();
        if (!token) throw new Error('Privy returned empty token');

        setStatus('Token acquired. Sending to stablectl…');
        const res = await fetch('/token', {
          method: 'POST',
          headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
          body: new URLSearchParams({ jwt: token }).toString(),
        });
        const text = await res.text();
        if (!res.ok) throw new Error(text || 'Failed to submit token');

        setStatus('Session saved! You can close this tab and return to terminal.');
        setDone(true);
      } catch (err) {
        const msg = err?.message || String(err);
        setStatus('Error: ' + msg);
        setSubmitting(false);
      }
    }
    submitToken();
  }, [ready, authenticated, submitting, done]);

  return (
    <div style={{ fontFamily: '-apple-system, BlinkMacSystemFont, Segoe UI, Roboto, sans-serif', maxWidth: 720, margin: '2rem auto', padding: '0 1rem' }}>
      <h1 style={{ marginBottom: '0.2rem' }}>stablectl signer login</h1>
      <p style={{ color: '#666', marginBottom: '1.5rem' }}>
        Privy browser login with automatic token capture for <code>privy-user</code> signer.
      </p>

      <div style={{ border: '1px solid #e7e7e7', borderRadius: 10, padding: '1.2rem', marginBottom: '1rem' }}>
        {authenticated && (
          <p style={{ color: '#666', margin: '0 0 0.5rem' }}>
            Logged in as {user?.id || 'user'}.
          </p>
        )}

        <p style={{
          color: done ? '#0a7a2a' : status.startsWith('Error') ? '#8a5a00' : '#666',
          fontWeight: done ? 600 : 400,
          margin: '0 0 1rem',
        }}>
          {done ? '✅ ' : ''}{status}
        </p>

        {!ready && <p style={{ color: '#999' }}>Loading Privy SDK…</p>}

        {ready && !authenticated && (
          <button
            onClick={() => login()}
            style={{
              padding: '0.7rem 1.4rem',
              borderRadius: 8,
              border: '1px solid #d8d8d8',
              background: '#f9f9f9',
              cursor: 'pointer',
              fontSize: '1rem',
            }}
          >
            Login with Privy
          </button>
        )}

        {authenticated && !done && (
          <button
            onClick={() => logout()}
            style={{
              padding: '0.5rem 1rem',
              borderRadius: 8,
              border: '1px solid #d8d8d8',
              background: '#f9f9f9',
              cursor: 'pointer',
              fontSize: '0.9rem',
            }}
          >
            Logout
          </button>
        )}
      </div>

      <details style={{ border: '1px solid #e7e7e7', borderRadius: 10, padding: '1rem' }}>
        <summary style={{ cursor: 'pointer' }}><strong>Manual fallback</strong></summary>
        <p style={{ color: '#666', marginTop: '0.5rem' }}>Paste a Privy access token if browser login does not work.</p>
        <form method="post" action="/token">
          <label htmlFor="jwt">User JWT / access token</label><br />
          <textarea
            id="jwt"
            name="jwt"
            placeholder="eyJ..."
            required
            style={{ width: '100%', minHeight: 100, fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace', marginTop: '0.3rem' }}
          /><br />
          <button
            type="submit"
            style={{ marginTop: '0.5rem', padding: '0.5rem 1rem', borderRadius: 8, border: '1px solid #d8d8d8', background: '#f9f9f9', cursor: 'pointer' }}
          >
            Validate &amp; Save Session
          </button>
        </form>
      </details>
    </div>
  );
}

export default function App() {
  return (
    <PrivyProvider
      appId={APP_ID}
      config={{
        appearance: { theme: 'light' },
        loginMethods: ['email', 'wallet'],
        embeddedWallets: { createOnLogin: 'off' },
      }}
    >
      <LoginPanel />
    </PrivyProvider>
  );
}
