import React, { useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';
import './style.css';

type Item = Record<string, string | number | null>;
type Page = { items: Item[]; next_after: string | null };
type Reply = { ok: boolean; data?: unknown; error?: { code: string } };
const empty: Page = { items: [], next_after: null };

function App() {
  const [message, setMessage] = useState('로컬 라이브러리. 등록·수정은 CLI 또는 MCP에서 수행합니다.');
  const [busy, setBusy] = useState(false);
  const lock = useRef(false);
  const [projects, setProjects] = useState<Page>(empty);
  const [project, setProject] = useState('');
  const [search, setSearch] = useState('');
  const [kind, setKind] = useState('');
  const [assets, setAssets] = useState<Page>(empty);
  const [asset, setAsset] = useState('');
  const [versions, setVersions] = useState<Page>(empty);
  const [version, setVersion] = useState('');
  const [audio, setAudio] = useState('');
  const [details, setDetails] = useState('');
  const [account, setAccount] = useState('');
  const [key, setKey] = useState('');
  const [enabled, setEnabled] = useState(false);
  const [calls, setCalls] = useState('');
  const [credits, setCredits] = useState('');

  async function run(action: () => Promise<void>) {
    if (lock.current) return;
    lock.current = true; setBusy(true); setMessage('처리 중…');
    try { await action(); }
    catch (error) { setMessage(error instanceof Error ? error.message : typeof error === 'string' ? error : '작업을 완료하지 못했습니다.'); }
    finally { lock.current = false; setBusy(false); }
  }
  async function query(op: string, args: Record<string, unknown> = {}) {
    const reply = await invoke<Reply>('local_query', { command: { protocol_version: 1, op, ...args } });
    if (!reply.ok) throw new Error(reply.error?.code ?? '조회 실패');
    return reply.data;
  }
  function resetAssets() { setAssets(empty); setAsset(''); setVersions(empty); setVersion(''); setAudio(''); }
  function selectProject(id: string) { setProject(id); resetAssets(); setMessage('파일 목록을 조회하세요.'); }
  async function loadProjects(after = '') {
    await run(async () => {
      const page = await query('project.list', { after }) as Page;
      setProjects(page); setMessage(`프로젝트 ${page.items.length}개`);
    });
  }
  async function loadAssets(after = '') {
    await run(async () => {
      resetAssets();
      const page = await query('asset.list', { project_id: project, search, after, ...(kind ? { kind } : {}) }) as Page;
      setAssets(page); setMessage(`파일 ${page.items.length}개. 파일을 선택하면 버전을 조회합니다.`);
    });
  }
  async function loadVersions(id: string, after = '') {
    await run(async () => {
      setAsset(id); setVersion(''); setAudio(''); setVersions(empty);
      const page = await query('asset.versions', { project_id: project, asset_id: id, after }) as Page;
      setVersions(page); setMessage('ready 버전을 선택해 불러온 뒤 재생하세요.');
    });
  }
  return <main>
    <header><h1>DK GameSound Manager</h1><button disabled={busy} onClick={() => run(async () => {
      const reply = await invoke<Reply>('sound_status');
      setDetails(JSON.stringify(reply, null, 2));
      setMessage(reply.ok ? '엔진에 연결했습니다.' : reply.error?.code ?? '연결 실패');
    })}>엔진 상태</button></header>
    <p className="note">오프라인 목록 · 원본/후보 확인 · 재생</p>
    <section aria-label="ElevenLabs 연결">
      <h2>ElevenLabs 연결</h2>
      <label>계정 ID <input disabled={busy} value={account} onChange={e => setAccount(e.target.value)} autoComplete="off" /></label>
      <label>API Key <input disabled={busy} type="password" value={key} onChange={e => setKey(e.target.value)} autoComplete="off" /></label>
      <label>생성 요청 총한도 <input disabled={busy} type="number" min="0" max="1000000" value={calls} onChange={e => setCalls(e.target.value)} /></label>
      <label>예약 크레딧 총한도 <input disabled={busy} type="number" min="0" max="1000000000" value={credits} onChange={e => setCredits(e.target.value)} /></label>
      <label><input disabled={busy} type="checkbox" checked={enabled} onChange={e => setEnabled(e.target.checked)} /> 유료 생성 허용</label>
      <p>한도는 이 PC의 동일 workspace 누적 기록에 적용되며 자동 초기화되지 않습니다. 실제 청구액 상한은 보장하지 않습니다. 서비스 측 Key 한도도 설정하세요.</p>
      <button disabled={busy || !account || !key || (enabled && (!calls || !credits))} onClick={() => run(async () => {
        try {
          const reply = await invoke<Reply>('save_connection', { accountId: account, key, enabled, requestLimit: Number(calls || 0), creditLimit: Number(credits || 0) });
          if (!reply.ok) throw new Error(reply.error?.code ?? '저장 실패');
          setMessage('Windows Credential Manager에 저장했습니다. 생성 작업은 CLI/MCP에서 제출하세요.');
        } finally { setKey(''); }
      })}>연결 설정 저장</button>
      <button disabled={busy || !account} onClick={() => run(async () => {
        setKey('');
        const reply = await invoke<Reply>('disconnect_connection', { accountId: account });
        if (!reply.ok) throw new Error(reply.error?.code ?? '해제 실패');
        setMessage('저장된 Key와 생성 허용 설정을 삭제했습니다.');
      })}>연결 해제</button>
    </section>
    <section aria-label="프로젝트">
      <div className="line"><button disabled={busy} onClick={() => loadProjects()}>프로젝트 목록</button>
        <button disabled={busy || !projects.next_after} onClick={() => loadProjects(projects.next_after!)}>다음 10개</button></div>
      <ul>{projects.items.map(row => <li key={String(row.project_id)}><button disabled={busy} onClick={() => selectProject(String(row.project_id))}>{row.name} <small>({row.project_id})</small></button></li>)}</ul>
      <label>프로젝트 ID <input disabled={busy} value={project} onChange={e => selectProject(e.target.value)} placeholder="CLI에서 만든 ID" /></label>
    </section>
    <section aria-label="사운드">
      <div className="line"><label>검색 <input disabled={busy} value={search} onChange={e => { setSearch(e.target.value); resetAssets(); }} /></label>
        <label>종류 <select disabled={busy} value={kind} onChange={e => { setKind(e.target.value); resetAssets(); }}>
          <option value="">전체</option><option value="bgm">BGM</option><option value="sfx">SFX</option><option value="voice">Voices</option>
        </select></label>
        <button disabled={busy || !project} onClick={() => loadAssets()}>파일 조회</button></div>
      <ul>{assets.items.map(row => <li key={String(row.asset_id)}><button disabled={busy} onClick={() => loadVersions(String(row.asset_id))}>{row.name} <small>{row.kind} / {row.asset_id}</small></button></li>)}</ul>
      <button disabled={busy || !assets.next_after} onClick={() => loadAssets(assets.next_after!)}>파일 다음 10개</button>
      {asset && <div className="line"><label>{asset} 버전 <select disabled={busy} value={version} onChange={e => { setVersion(e.target.value); setAudio(''); }}>
        <option value="">선택</option>{versions.items.map(row => <option key={String(row.version_id)} value={String(row.version_id)} disabled={row.state !== 'ready'}>{row.version_id} · {row.role} · {row.state}</option>)}
      </select></label><button disabled={busy || !versions.next_after} onClick={() => loadVersions(asset, versions.next_after!)}>버전 다음 10개</button>
        <button disabled={busy || !version} onClick={() => run(async () => {
          setAudio('');
          const source = await invoke<string>('audio_source', { projectId: project, assetId: asset, versionId: version });
          setAudio(source); setMessage('해시 확인 완료. 아래 재생 버튼을 사용하세요.');
        })}>선택 버전 불러오기</button></div>}
      {audio && <audio key={`${project}/${asset}/${version}`} controls src={audio} onError={() => setMessage('재생할 수 없는 형식이거나 파일입니다. 해당 코덱의 WebView2 지원을 확인하세요.')} />}
    </section>
    <p role="status" aria-live="polite">{message}</p>
    {details && <details><summary>엔진 응답</summary><pre>{details}</pre></details>}
    <p className="note">미리듣기는 파일당 16 MiB까지 지원합니다. 생성·편집·게임 적용 기능은 후속 단계입니다.</p>
  </main>;
}
createRoot(document.getElementById('root')!).render(<App />);
