
import importlib.util, os, subprocess, sys, time, re

spec = importlib.util.spec_from_file_location('ac', '.agents/skills/autoui-verifier/scripts/acceptance_channel.py')
ac = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ac)

out_dir = 'docs/plans/evidence/p709'
os.makedirs(out_dir, exist_ok=True)
storage = os.path.join(out_dir, 'p709b-notepad-storage.json')
ac.write_storage(storage, {})

def shot_retry(s, name, tries=6):
    for attempt in range(tries):
        try:
            return s.shot(name)
        except RuntimeError as e:
            if attempt == tries - 1 or 'not produced' not in str(e):
                raise
            time.sleep(2.0)

s = ac.DesktopSession(out_dir, storage)
shots = []
notepad = None
try:
    # ① 真 notepad 启动（B1 同路径 pid 发现的原料）。
    notepad = subprocess.Popen(['notepad.exe'])
    time.sleep(1.5)
    # ② dock：bus dock_native pid → 宿主执行臂（发现/剥离/落槽）。
    out = s.bus(f'dock_native\tpid={notepad.pid}')
    print('DOCK:', out.strip())
    s.settle(6)
    shots.append(shot_retry(s, 'p709b-01-notepad-docked'))
    # ③ 投影：native 条目在场（N<pid-slot> 形态）。
    st = s.mcp.text('autoui_state', {'fields': ['__wm_wins', '__wm_fp']})
    print('WINS:', st.replace(chr(10), ' | ')[:500])
    m = re.search(r'N(\d+)', st)
    assert m, f'p709b: dock 后 __wm_wins 应含 native 条目: {st[:300]}'
    slot_id = m.group(1)
    # ④ send_to 槽位（AC-05 词表面）——修前预期 no-op（发现 R1 的实证面）。
    out = s.bus(f'send_to\tN{slot_id}\t1')
    print('SEND_TO:', out.strip())
    s.settle(4)
    shots.append(shot_retry(s, 'p709b-02-sendto-ws1'))
    st2 = s.mcp.text('autoui_state', {'fields': ['__wm_wins']})
    print('WINS-AFTER-SENDTO:', st2.replace(chr(10), ' | ')[:400])
    # ⑤ 切分区：当前分区非槽位分区 → 槽位隐（OS SW_HIDE + chrome 缺席）。
    s.bus('workspace\t1')
    s.settle(4)
    shots.append(shot_retry(s, 'p709b-03-ws1-slot-hidden'))
    # ⑥ 切回 → 复显。
    s.bus('workspace\t0')
    s.settle(4)
    shots.append(shot_retry(s, 'p709b-04-ws0-slot-reshown'))
    # ⑦ focus_native（任务栏条目点击等价）→ OS 前台 + WM 置顶。
    out = s.bus(f'focus_native\tN{slot_id}')
    print('FOCUS:', out.strip())
    s.settle(4)
    shots.append(shot_retry(s, 'p709b-05-focus-native'))
    # ⑧ undock → pre-dock 恢复。
    out = s.bus(f'undock_native\t{slot_id}')
    print('UNDOCK:', out.strip())
    s.settle(6)
    shots.append(shot_retry(s, 'p709b-06-undocked-restored'))
    print('[p709b] chain complete —', len(shots), 'shots')
finally:
    s.close()
    if notepad:
        try:
            notepad.kill()
        except Exception:
            pass
