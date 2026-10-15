"""Real process/SQLite/file/FFmpeg checks; no remote API or user assets."""
import argparse, concurrent.futures, hashlib, json, os, pathlib, platform, socket
import sqlite3, subprocess, tempfile, time, traceback, wave

p = argparse.ArgumentParser()
p.add_argument('--bin', required=True)
p.add_argument('--sha', required=True)
p.add_argument('--out', required=True)
a = p.parse_args()
bin_dir = pathlib.Path(a.bin).resolve()
root = pathlib.Path(tempfile.mkdtemp(prefix='T00 한글 공백 '))
env = dict(os.environ, T00_SPIKE_ROOT=str(root))
suffix = '.exe' if os.name == 'nt' else ''
report = {'source_sha': a.sha, 'os': platform.platform(), 'python': platform.python_version(),
          'scope': 'isolated synthetic fixtures', 'checks': {}, 'manual_gui': 'unverified',
          'listening': 'unverified', 'ipc_user_acl': 'unverified', 'root_basename': root.name}
engine = None

def run(args, **kw):
    return subprocess.run([str(x) for x in args], env=env, capture_output=True, text=True,
                          encoding='utf-8', timeout=30, **kw)

def cli(c):
    r = run([bin_dir / ('spike-cli'+suffix), json.dumps(c)])
    v = json.loads(r.stdout)
    assert (r.returncode == 0) == v['ok'], (r.returncode, v)
    assert not r.stderr, r.stderr
    return v

def wait_for(fn, timeout=8):
    end = time.monotonic()+timeout
    while time.monotonic()<end:
        try:
            v = fn()
            if v: return v
        except (AssertionError, OSError, ValueError): pass
        time.sleep(.05)
    raise AssertionError('timed out')

def start():
    global engine
    engine = subprocess.Popen([str(bin_dir / ('spike-engine'+suffix))], env=env,
                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    wait_for(lambda: cli({'op':'status'}).get('pid') == engine.pid)

def race_start():
    global engine
    contenders = [subprocess.Popen([str(bin_dir/('spike-engine'+suffix))], env=env,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) for _ in range(2)]
    try:
        pid=wait_for(lambda: cli({'op':'status'}).get('pid'))
        engine=next(c for c in contenders if c.pid==pid)
        loser=next(c for c in contenders if c.pid!=pid)
        assert loser.wait(5)!=0 and engine.poll() is None
    except Exception:
        for c in contenders:
            if c.poll() is None: c.kill(); c.wait(5)
        raise

def mcp_command(c):
    lines=[{'jsonrpc':'2.0','id':1,'method':'initialize','params':{'protocolVersion':'2025-11-25'}},
           {'jsonrpc':'2.0','method':'notifications/initialized'},
           {'jsonrpc':'2.0','id':2,'method':'tools/call','params':{'name':'spike_command','arguments':c}}]
    r=run([bin_dir/('spike-mcp'+suffix)],input='\n'.join(map(json.dumps,lines))+'\n')
    assert r.returncode==0 and not r.stderr
    replies=list(map(json.loads,r.stdout.splitlines()))
    assert len(replies)==2 and replies[-1]['id']==2
    return json.loads(replies[-1]['result']['content'][0]['text'])

def mark(k): report['checks'][k] = 'passed'
def submit(id, delay=0, cost=1): return cli({'op':'submit','id':id,'delay_ms':delay,'cost':cost})

try:
    race_start()
    mark('concurrent_first_engine_start_one_owner')
    assert cli({'op':'status'})['jobs'] == []
    dup = run([bin_dir / ('spike-engine'+suffix)])
    assert dup.returncode != 0 and 'already owns' in dup.stderr
    mark('duplicate_engine_rejected')
    endpoint = json.loads((root/'endpoint.json').read_text())
    with socket.create_connection(('127.0.0.1', endpoint['port'])) as s:
        s.sendall(b'{"token":"wrong","command":{"op":"status"}}\n')
        assert json.loads(s.makefile().readline())['error']=='unauthorized'
    mark('wrong_token_rejected')
    assert submit('../escape')['ok'] is False
    assert cli({'op':'apply','approved':True})['ok'] is False
    mark('common_validation_and_no_apply')
    request = {'op':'submit','id':'same','delay_ms':1200,'cost':1}
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        results = list(pool.map(lambda _:cli(request),range(8)))
    assert all(v['ok'] for v in results) and sum(not v['duplicate'] for v in results)==1
    assert submit('same', delay=1)['error']=='id conflict'
    mark('concurrent_idempotency_and_conflict')
    # All submitter processes have exited; engine continues and a fresh CLI reconnects.
    wait_for(lambda: cli({'op':'status'})['jobs'][0]['state']=='succeeded')
    assert (root/'same.fixture').read_text()=='fixture:same'
    mark('client_exit_job_continues_and_reconnect')
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        results = list(pool.map(lambda i: (submit(f'budget-{i}',cost=3) if i%2==0 else mcp_command({'op':'submit','id':f'budget-{i}','delay_ms':0,'cost':3})),range(8)))
    assert sum(v['ok'] for v in results)==3
    assert cli({'op':'status'})['reserved']==10
    mark('concurrent_cli_mcp_fixture_budget')
    lines = [
      {'jsonrpc':'2.0','id':1,'method':'initialize','params':{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'fixture','version':'0'}}},
      {'jsonrpc':'2.0','method':'notifications/initialized'},
      {'jsonrpc':'2.0','id':2,'method':'tools/list'},
      {'jsonrpc':'2.0','id':3,'method':'tools/call','params':{'name':'spike_command','arguments':{'op':'status'}}},
      {'jsonrpc':'2.0','id':4,'method':'tools/call','params':{'name':'spike_command','arguments':{'op':'submit','id':'denied','delay_ms':0,'cost':1}}},
      {'jsonrpc':'2.0','id':5,'method':'bogus'}]
    r = run([bin_dir/('spike-mcp'+suffix)], input='\n'.join(map(json.dumps,lines))+'\n')
    replies = list(map(json.loads,r.stdout.splitlines()))
    assert not r.stderr and len(replies)==5 and [v['id'] for v in replies]==[1,2,3,4,5]
    assert json.loads(replies[2]['result']['content'][0]['text'])==cli({'op':'status'})
    assert replies[3]['result']['isError'] and replies[4]['error']['code']==-32601
    mark('stdio_mcp_handshake_stdout_common_status_budget')
    assert cli({'op':'shutdown'})['ok']
    assert engine.wait(5)==0
    assert not (root/'endpoint.json').exists()
    # New isolated DB for termination cases, never erase a user's DB.
    recovery = root/'복구 경로'
    env['T00_SPIKE_ROOT']=str(recovery)
    start()
    assert submit('killed',delay=30000)['ok']
    engine.kill(); engine.wait(5)
    start()
    st=cli({'op':'status'})
    assert st['jobs'][0]['state']=='interrupted' and st['reserved']==1
    assert not (recovery/'killed.fixture').exists()
    assert submit('killed',delay=30000)['duplicate']
    assert submit('normal-stop',delay=30000)['ok']
    cli({'op':'shutdown'}); assert engine.wait(5)==0
    start()
    assert all(j['state']=='interrupted' for j in cli({'op':'status'})['jobs'])
    mark('kill_restart_no_auto_retry_reservation_retained')
    mark('explicit_shutdown_marks_interrupted')
    db=sqlite3.connect(recovery/'spike.sqlite')
    assert db.execute('pragma integrity_check').fetchone()[0]=='ok'
    db.close(); mark('sqlite_integrity_after_termination')
    cli({'op':'shutdown'}); engine.wait(5)
    for point in ('after_temp','after_rename'):
        case=root/point
        env['T00_SPIKE_ROOT']=str(case)
        env['T00_SPIKE_CRASH_POINT']=point
        start()
        assert submit('gap', delay=200)['ok']
        assert engine.wait(8)==86
        del env['T00_SPIKE_CRASH_POINT']
        start()
        assert cli({'op':'status'})['jobs'][0]['state']=='interrupted'
        assert (case/('gap.tmp' if point=='after_temp' else 'gap.fixture')).exists()
        cli({'op':'shutdown'}); engine.wait(5)
    report['file_db_gap']='observed: interrupted + leftover file; automatic reconciliation NOT implemented'
    mark('file_db_gap_fault_injection_detected_limitation')
    # Real generated audio, separate output, argument arrays and original hash preservation.
    tone=root/'합성 음원.wav'; converted=root/'변환 후보.wav'
    for tool in ('ffmpeg','ffprobe'):
        v=run([tool,'-version']); assert v.returncode==0
        report[tool]=v.stdout.splitlines()[0]
    r=run(['ffmpeg','-v','error','-f','lavfi','-i','sine=frequency=440:duration=1','-c:a','pcm_s16le',tone]); assert r.returncode==0,r.stderr
    before=hashlib.sha256(tone.read_bytes()).hexdigest()
    r=run(['ffmpeg','-v','error','-i',tone,'-af','afade=t=out:st=0.7:d=0.3','-ar','24000',converted]); assert r.returncode==0,r.stderr
    assert hashlib.sha256(tone.read_bytes()).hexdigest()==before
    r=run(['ffprobe','-v','error','-show_streams','-show_format','-of','json',converted]); assert r.returncode==0
    probe=json.loads(r.stdout)
    assert probe['streams'][0]['sample_rate']=='24000' and .99<=float(probe['format']['duration'])<=1.01
    report['synthetic_source_sha256']=before
    mark('ffmpeg_ffprobe_unicode_paths_original_preserved')
    # Fixture-only direct access comparison: SQLite writer exclusion, not a product adapter.
    f=root/'direct.sqlite'; one=sqlite3.connect(f); two=sqlite3.connect(f,timeout=.05)
    one.execute('create table budget(n integer)'); one.commit()
    one.execute('begin immediate')
    try:
        two.execute('begin immediate')
        raise AssertionError('second writer unexpectedly acquired')
    except sqlite3.OperationalError as e:
        assert 'locked' in str(e)
    finally:
        one.rollback(); one.close(); two.close()
    mark('direct_sqlite_writer_exclusion_fixture')
    if os.name=='nt':
        r=run(['powershell','-NoProfile','-ExecutionPolicy','Bypass','-File',pathlib.Path(__file__).with_name('credentials.ps1')])
        assert r.returncode==0, r.stderr
        report['credentials']=json.loads(r.stdout)
        mark('windows_dummy_credentials_roundtrip_delete')
        # This proves process survival only; no UI or audible success assertion.
        gui_env=dict(env, T00_SPIKE_ROOT=str(root))
        gui=subprocess.Popen([str(bin_dir/'spike-gui.exe')], env=gui_env)
        try:
            time.sleep(3)
            assert gui.poll() is None, 'GUI process exited early'
            mark('gui_process_survived_3_seconds_only')
        finally:
            if gui.poll() is None: gui.terminate(); gui.wait(5)
    report['result']='passed_automated_only'
except Exception:
    report['result']='failed'
    report['failure']=traceback.format_exc().replace(str(root),'<temporary-root>')
finally:
    if engine and engine.poll() is None:
        try: cli({'op':'shutdown'}); engine.wait(5)
        except Exception: engine.kill(); engine.wait(5)
    pathlib.Path(a.out).write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps(report,ensure_ascii=False,indent=2))
    # Preserve only synthetic tone for manual GUI; no endpoint/token/DB exported.
    if (root/'합성 음원.wav').exists():
        import shutil
        shutil.copyfile(root/'합성 음원.wav',pathlib.Path(a.out).with_suffix('.wav'))
raise SystemExit(0 if report['result']=='passed_automated_only' else 1)
