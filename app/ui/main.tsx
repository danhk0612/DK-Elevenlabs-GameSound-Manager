import React, { useState } from 'react';
import { createRoot } from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';
import './style.css';

function App() {
  const [result, setResult] = useState('상태 조회 시 공통 엔진에 연결합니다.');
  const [busy, setBusy] = useState(false);
  async function refresh() {
    setBusy(true);
    try { setResult(JSON.stringify(await invoke('sound_status'), null, 2)); }
    catch { setResult('상태를 조회하지 못했습니다. 엔진 실행 파일과 접근 권한을 확인하세요.'); }
    finally { setBusy(false); }
  }
  return <main>
    <h1>DK GameSound Manager</h1>
    <p>자동화 엔진 연결 상태</p>
    <button disabled={busy} onClick={refresh}>{busy ? '연결 중…' : '상태 조회 / 재연결'}</button>
    <pre role="status" aria-live="polite">{result}</pre>
    <p className="note">사운드 관리·생성 기능은 아직 제공하지 않습니다. 창을 닫아도 엔진은 유지됩니다.</p>
  </main>;
}
createRoot(document.getElementById('root')!).render(<App />);
