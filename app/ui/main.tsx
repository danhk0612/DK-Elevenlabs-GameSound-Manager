import React, { useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';
import './style.css';

type Item = Record<string, string | number | null>;
type Page = { items: Item[]; next_after: string | null };
type Reply = { ok: boolean; data?: unknown; error?: { code: string } };
type Review = {project_id: string; review_id: string; state: string; snapshot: {candidate: {asset_id: string; version_id: string; sha256: string}; summary: string}};
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
  const [comparison, setComparison] = useState<{source: string; label: string} | null>(null);
  const [operation, setOperation] = useState('');
  const [task, setTask] = useState<{project: string; id: string; state: string; phase: string; cancelling: boolean} | null>(null);
  const [waveform, setWaveform] = useState<number[][]>([]);
  const [measurement, setMeasurement] = useState('');
  const [reviews, setReviews] = useState<Page>(empty);
  const [review, setReview] = useState<Review | null>(null);
  const [history, setHistory] = useState<Page>(empty);

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
  function resetAssets() { setHistory(empty); setWaveform([]); setMeasurement(''); setAssets(empty); setAsset(''); setVersions(empty); setVersion(''); setAudio(''); }
  function selectProject(id: string) { setProject(id); setReviews(empty); setReview(null); setDetails(''); setComparison(null); setWaveform([]); setMeasurement(''); resetAssets(); setMessage('파일 목록을 조회하세요.'); }
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
      setHistory(empty); setAsset(id); setVersion(''); setAudio(''); setVersions(empty); setWaveform([]); setMeasurement('');
      const page = await query('asset.versions', { project_id: project, asset_id: id, after }) as Page;
      setVersions(page); setMessage('ready 버전을 선택해 불러온 뒤 재생하세요.');
    });
  }
  async function loadReviews(after = '') {
    await run(async () => {
      setReviews(await query('review.list', {project_id: project, after}) as Page);
      setMessage('검토 기록을 조회했습니다. 대기는 승인을 의미하지 않습니다.');
    });
  }
  async function loadHistory(after = '') {
    await run(async () => {
      const page = await query('asset.history', {project_id: project, asset_id: asset, version_id: version, after}) as Page;
      setHistory(page); setMessage('선택 버전 이력 · 종류/ID 순서');
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
      {asset && <div className="line"><label>{asset} 버전 <select disabled={busy} value={version} onChange={e => { setHistory(empty); setVersion(e.target.value); setAudio(''); setWaveform([]); setMeasurement(''); }}>
        <option value="">선택</option>{versions.items.map(row => <option key={String(row.version_id)} value={String(row.version_id)} disabled={row.state !== 'ready'}>{row.version_id} · {row.role} · {row.state}</option>)}
      </select></label><button disabled={busy || !versions.next_after} onClick={() => loadVersions(asset, versions.next_after!)}>버전 다음 10개</button>
        <button disabled={busy || !version} onClick={() => run(async () => {
          setAudio('');
          const source = await invoke<string>('audio_source', { projectId: project, assetId: asset, versionId: version });
          setAudio(source); setMessage('해시 확인 완료. 아래 재생 버튼을 사용하세요.');
        })}>선택 버전 불러오기</button></div>}
      <div className="line"><button disabled={busy || !project || !asset || !version} onClick={() => loadHistory()}>선택 버전 이력</button>
        <button disabled={busy || !history.next_after} onClick={() => loadHistory(history.next_after!)}>이력 다음 10개</button></div>
      <ul>{history.items.map(row => <li key={String(row.cursor)}>{row.kind} / {row.id} · {row.state} <button disabled={busy} onClick={() => run(async () => {
        const op = row.kind === 'review' ? 'review.get' : row.kind === 'generation' ? 'job.get' : 'audio.get';
        const idKey = row.kind === 'review' ? 'review_id' : row.kind === 'generation' ? 'job_id' : 'operation_id';
        setDetails(JSON.stringify(await query(op, {project_id: project, [idKey]: row.id}), null, 2)); setMessage('이력 상세를 조회했습니다.');
      })}>상세</button></li>)}</ul>
      {audio && <audio key={`${project}/${asset}/${version}`} controls src={audio} onError={() => setMessage('재생할 수 없는 형식이거나 파일입니다. 해당 코덱의 WebView2 지원을 확인하세요.')} />}
      {audio && <button disabled={busy} onClick={() => setComparison({source: audio, label: `${project} / ${asset} / ${version}`})}>비교용으로 보관</button>}
      {comparison && <div><p>비교 버전: {comparison.label}</p><audio controls src={comparison.source} /><button disabled={busy} onClick={() => setComparison(null)}>비교 닫기</button></div>}
      <div className="line"><label>작업 ID <input disabled={busy} value={operation} onChange={e => { setOperation(e.target.value); setTask(null); }} /></label>
        <button disabled={busy || !operation || !project} onClick={() => run(async () => {
          const record = await query('audio.get', { project_id: project, operation_id: operation }) as {state: string; phase: string; cancel_requested: boolean};
          setTask({project, id: operation, state: record.state, phase: record.phase, cancelling: record.cancel_requested});
          setDetails(JSON.stringify(record, null, 2)); setMessage(`작업 ${record.state} · ${record.phase}${record.cancel_requested ? ' · 취소 요청됨' : ''}`);
        })}>작업 상태</button>
        <button disabled={busy || !task || task.project !== project || task.id !== operation || task.cancelling || !['queued','processing','interrupted'].includes(task.state)} onClick={() => run(async () => {
          const reply = await invoke<Reply>('cancel_audio', { projectId: project, operationId: operation });
          if (!reply.ok) throw new Error(reply.error?.code ?? '취소 요청 실패');
          const record = reply.data as {state: string; phase: string; cancel_requested: boolean};
          setTask({project, id: operation, state: record.state, phase: record.phase, cancelling: record.cancel_requested});
          setMessage(record.state === 'cancelled' ? '작업을 취소했습니다.' : '취소를 요청했습니다. 작업 상태에서 완료 여부를 확인하세요.');
        })}>취소 요청</button>
        <button disabled={busy || !operation || !project || !asset || !version} onClick={() => run(async () => {
          setWaveform([]); setMeasurement('');
          const record = await query('audio.get', { project_id: project, operation_id: operation }) as {state: string; request: {asset_id: string; version_id: string}; result?: {metadata?: {sample_rate: number; channels: number}; metrics?: {duration_ms: number; sample_peak_dbfs: number | null; rms_dbfs: number | null; waveform: number[][]}}};
          if (record.state !== 'succeeded' || record.request.asset_id !== asset || record.request.version_id !== version || !record.result?.metrics) throw new Error('선택한 파일 버전의 완료된 분석 ID를 입력하세요.');
          const metrics = record.result.metrics;
          setWaveform(metrics.waveform);
          setMeasurement(`${metrics.duration_ms.toFixed(1)} ms · ${record.result.metadata?.sample_rate} Hz · ${record.result.metadata?.channels} ch · peak ${metrics.sample_peak_dbfs?.toFixed(2) ?? '−∞'} dBFS · RMS ${metrics.rms_dbfs?.toFixed(2) ?? '−∞'} dBFS`);
          setDetails(JSON.stringify(record, null, 2)); setMessage('검사 수치를 조회했습니다. 음질 평가와 적용 승인은 별도입니다.');
        })}>검사 결과 조회</button></div>
      {waveform.length > 0 && <figure><svg role="img" aria-label="채널을 합친 최소·최대 파형" viewBox={`0 0 ${waveform.length} 100`} preserveAspectRatio="none" style={{width:'100%',height:120,background:'#fff'}}>
        <line x1="0" x2={waveform.length} y1="50" y2="50" stroke="#c3c8c0" strokeWidth="0.5" />
        {waveform.map(([min,max],index) => <line key={index} x1={index+0.5} x2={index+0.5} y1={50-Math.max(-1,Math.min(1,max))*48} y2={50-Math.max(-1,Math.min(1,min))*48} stroke="#303833" strokeWidth="0.8" />)}
      </svg><figcaption>{measurement} · 표시 범위 ±1</figcaption></figure>}

    </section>
    <section aria-label="검토 대기">
      <h2>검토 기록</h2>
      <div className="line"><button disabled={busy || !project} onClick={() => loadReviews()}>검토 목록</button>
        <button disabled={busy || !reviews.next_after} onClick={() => loadReviews(reviews.next_after!)}>검토 다음 10개</button></div>
      <ul>{reviews.items.map(row => <li key={String(row.review_id)}><button disabled={busy} onClick={() => run(async () => {
        setReview(null);
        const record = await query('review.get', {project_id: project, review_id: row.review_id}) as Review;
        setReview(record); setDetails(JSON.stringify(record, null, 2)); setMessage('제출 당시 기록입니다. 현재 자료 일치는 별도로 확인하세요.');
      })}>{row.review_id} · {row.state} · {row.asset_id}/{row.version_id}</button></li>)}</ul>
      {review && review.project_id === project && <div>
        <p>{review.review_id} · {review.state} · {review.snapshot.summary}</p>
        <p><small>후보 SHA-256: {review.snapshot.candidate.sha256}</small></p>
        <button disabled={busy} onClick={() => run(async () => {
          const result = await query('review.check', {project_id: project, review_id: review.review_id}) as {matches: boolean};
          setDetails(JSON.stringify(result, null, 2)); setMessage(result.matches ? '현재 파일·연결 자료가 제출 기록과 일치합니다. 적용 승인은 별도입니다.' : '제출 이후 자료가 달라졌거나 파일을 확인할 수 없습니다. 상세 응답을 확인하세요.');
        })}>현재 자료 일치 확인</button>
        <button disabled={busy} onClick={() => run(async () => {
          setComparison(null);
          const candidate = review.snapshot.candidate;
          const source = await invoke<string>('audio_source', {projectId: project, assetId: candidate.asset_id, versionId: candidate.version_id, expectedSha256: candidate.sha256});
          setComparison({source, label: `${project} / ${candidate.asset_id} / ${candidate.version_id} · 검토 ${review.review_id}`});
          setMessage('검토 후보 해시를 확인했습니다. 비교 버전에서 재생하세요.');
        })}>검토 후보 재생 준비</button>
      </div>}
    </section>
    <p role="status" aria-live="polite">{message}</p>
    {details && <details><summary>상세 응답</summary><pre>{details}</pre></details>}
    <p className="note">미리듣기는 파일당 16 MiB까지 지원합니다. 생성·편집·검사는 CLI/MCP에서 수행합니다. 게임 적용은 후속 단계입니다.</p>
  </main>;
}
createRoot(document.getElementById('root')!).render(<App />);
