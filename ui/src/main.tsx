import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

// Inter 可变字体（SIL OFL-1.1）随包分发，避免依赖系统是否装了 Inter
import '@fontsource-variable/inter';
import './styles/tokens.css';
import './styles/app.css';

import { App } from './App';

const container = document.getElementById('root');
if (!container) throw new Error('index.html 缺少 #root 容器');

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
