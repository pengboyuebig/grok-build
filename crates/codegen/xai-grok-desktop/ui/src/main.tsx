/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from './App';

const rootElement = document.getElementById('root');
if (!rootElement) {
  throw new Error('未找到应用根节点');
}

createRoot(rootElement).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
