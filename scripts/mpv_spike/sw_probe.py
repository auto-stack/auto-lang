"""T-15 gate spike (PLAN-617): validate libmpv-2.dll runtime load + SW render path.

Measures, per the acceptance criteria of AC-18:
  * frames/sec achievable at a given target resolution (1080p / 4K)
  * per-frame cost of mpv_render_context_render() (the SW scale + blit into our buffer)
  * process RSS before/after
  * whether frames actually advance (i.e. it is real playback, not a frozen first frame)

Usage:
  python mpv_sw_probe.py <video> <width> <height> [--untimed] [--seconds N] [--hwdec MODE]
"""

import argparse
import ctypes
import hashlib
import mmap
import os
import statistics
import struct
import sys
import time

DLL = os.environ.get("AUTO_MPV_LIB") or r"libmpv-2.dll"

# ---- mpv constants (from include/mpv/render.h, client.h) -------------------
PARAM_INVALID = 0
PARAM_API_TYPE = 1
PARAM_ADVANCED_CONTROL = 10
PARAM_BLOCK_FOR_TARGET_TIME = 12
PARAM_SKIP_RENDERING = 13
PARAM_SW_SIZE = 17
PARAM_SW_FORMAT = 18
PARAM_SW_STRIDE = 19
PARAM_SW_POINTER = 20

UPDATE_FRAME = 1  # MPV_RENDER_UPDATE_FRAME
UPDATE_OVERLAY = 2

EVENT_NONE = 0
EVENT_SHUTDOWN = 1
EVENT_LOG_MESSAGE = 2
EVENT_END_FILE = 7
EVENT_FILE_LOADED = 8
EVENT_VIDEO_RECONFIG = 17
EVENT_PLAYBACK_RESTART = 21

FORMAT_ERROR = -1
FORMAT_SUCCESS = 0


class RenderParam(ctypes.Structure):
    _fields_ = [("type", ctypes.c_int), ("data", ctypes.c_void_p)]


class Event(ctypes.Structure):
    _fields_ = [
        ("event_id", ctypes.c_int),
        ("error", ctypes.c_int),
        ("reply_userdata", ctypes.c_uint64),
        ("data", ctypes.c_void_p),
    ]


class ProcessMemoryCounters(ctypes.Structure):
    _fields_ = [
        ("cb", ctypes.c_ulong),
        ("PageFaultCount", ctypes.c_ulong),
        ("PeakWorkingSetSize", ctypes.c_size_t),
        ("WorkingSetSize", ctypes.c_size_t),
        ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
        ("QuotaPagedPoolUsage", ctypes.c_size_t),
        ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
        ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
        ("PagefileUsage", ctypes.c_size_t),
        ("PeakPagefileUsage", ctypes.c_size_t),
    ]


class FileTime(ctypes.Structure):
    _fields_ = [("low", ctypes.c_ulong), ("high", ctypes.c_ulong)]


_k32 = ctypes.windll.kernel32
_k32.GetCurrentProcess.restype = ctypes.c_void_p

try:
    _k32.K32GetProcessMemoryInfo.argtypes = [
        ctypes.c_void_p, ctypes.POINTER(ProcessMemoryCounters), ctypes.c_ulong
    ]
    _k32.K32GetProcessMemoryInfo.restype = ctypes.c_int
    _GETMEM = _k32.K32GetProcessMemoryInfo
except AttributeError:  # pragma: no cover - older Windows
    _psapi = ctypes.windll.psapi
    _psapi.GetProcessMemoryInfo.argtypes = [
        ctypes.c_void_p, ctypes.POINTER(ProcessMemoryCounters), ctypes.c_ulong
    ]
    _psapi.GetProcessMemoryInfo.restype = ctypes.c_int
    _GETMEM = _psapi.GetProcessMemoryInfo

_k32.GetProcessTimes.argtypes = [
    ctypes.c_void_p,
    ctypes.POINTER(FileTime), ctypes.POINTER(FileTime),
    ctypes.POINTER(FileTime), ctypes.POINTER(FileTime),
]
_k32.GetProcessTimes.restype = ctypes.c_int
_GETTIMES = _k32.GetProcessTimes


def rss_mb() -> float:
    pmc = ProcessMemoryCounters()
    pmc.cb = ctypes.sizeof(ProcessMemoryCounters)
    ok = _GETMEM(_k32.GetCurrentProcess(), ctypes.byref(pmc), pmc.cb)
    if not ok:
        return float("nan")
    return pmc.WorkingSetSize / (1024 * 1024)


def _ft_secs(ft) -> float:
    return ((ft.high << 32) | ft.low) / 1e7


def cpu_secs() -> float:
    """Process CPU time (kernel+user) in seconds - a decode-cost proxy."""
    c, e, k, u = FileTime(), FileTime(), FileTime(), FileTime()
    if not _GETTIMES(
        _k32.GetCurrentProcess(),
        ctypes.byref(c), ctypes.byref(e), ctypes.byref(k), ctypes.byref(u),
    ):
        return float("nan")
    return _ft_secs(k) + _ft_secs(u)


def bind(mpv):
    mpv.mpv_create.restype = ctypes.c_void_p
    mpv.mpv_initialize.argtypes = [ctypes.c_void_p]
    mpv.mpv_initialize.restype = ctypes.c_int
    mpv.mpv_set_option_string.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    mpv.mpv_set_option_string.restype = ctypes.c_int
    mpv.mpv_command.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_char_p)]
    mpv.mpv_command.restype = ctypes.c_int
    mpv.mpv_wait_event.argtypes = [ctypes.c_void_p, ctypes.c_double]
    mpv.mpv_wait_event.restype = ctypes.POINTER(Event)
    mpv.mpv_terminate_destroy.argtypes = [ctypes.c_void_p]
    mpv.mpv_error_string.argtypes = [ctypes.c_int]
    mpv.mpv_error_string.restype = ctypes.c_char_p
    mpv.mpv_client_api_version.restype = ctypes.c_ulong
    mpv.mpv_get_property.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int, ctypes.c_void_p]
    mpv.mpv_get_property.restype = ctypes.c_int
    mpv.mpv_render_context_create.argtypes = [
        ctypes.POINTER(ctypes.c_void_p),
        ctypes.c_void_p,
        ctypes.POINTER(RenderParam),
    ]
    mpv.mpv_render_context_create.restype = ctypes.c_int
    mpv.mpv_render_context_render.argtypes = [ctypes.c_void_p, ctypes.POINTER(RenderParam)]
    mpv.mpv_render_context_render.restype = ctypes.c_int
    mpv.mpv_render_context_update.argtypes = [ctypes.c_void_p]
    mpv.mpv_render_context_update.restype = ctypes.c_uint64
    mpv.mpv_render_context_free.argtypes = [ctypes.c_void_p]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("video")
    ap.add_argument("width", type=int)
    ap.add_argument("height", type=int)
    ap.add_argument("--seconds", type=float, default=6.0)
    ap.add_argument("--untimed", action="store_true", help="max throughput instead of realtime pacing")
    ap.add_argument("--advanced", action="store_true",
                    help="advanced control: only render on MPV_RENDER_UPDATE_FRAME (production pattern)")
    ap.add_argument("--no-block", action="store_true",
                    help="BLOCK_FOR_TARGET_TIME=0: render() returns immediately, so the measured "
                         "call time is the pure scale+blit cost, not mpv's pacing wait")
    ap.add_argument("--hwdec", default="no")
    ap.add_argument("--ao", default=None,
                    help="null => keep the audio clock (ao=null) so playback is paced in "
                         "realtime; omit => audio disabled and mpv free-runs (throughput bound)")
    ap.add_argument("--start", default=None, help="start time before measuring (e.g. 00:03:00)")
    ap.add_argument("--label", default="")
    args = ap.parse_args()

    print(f"=== libmpv SW render probe | label={args.label or 'n/a'} ===")
    print(f"dll     : {DLL}  exists={os.path.exists(DLL)}")
    print(f"video   : {os.path.basename(args.video)}  size={os.path.getsize(args.video)/1e9:.2f} GB")
    print(f"target  : {args.width}x{args.height}   untimed={args.untimed}  hwdec={args.hwdec}")

    # ---- 1. runtime load (no import lib, no build-time dep) ----------------
    t0 = time.perf_counter()
    mpv = ctypes.CDLL(DLL)
    print(f"[1] ctypes.CDLL load : {1000*(time.perf_counter()-t0):.0f} ms")
    bind(mpv)
    print(f"    client api version : {mpv.mpv_client_api_version() >> 16}.{mpv.mpv_client_api_version() & 0xFFFF}")

    # ---- 2. handle + render context ---------------------------------------
    h = mpv.mpv_create()
    if not h:
        sys.exit("FATAL: mpv_create returned NULL")

    def opt(name, val):
        r = mpv.mpv_set_option_string(h, name.encode(), str(val).encode())
        if r < 0:
            print(f"    ! option {name}={val} -> {mpv.mpv_error_string(r).decode()}")

    opt("vo", "libmpv")          # render API owns the VO
    if args.ao:
        opt("ao", args.ao)       # ao=null keeps mpv's clock -> realtime pacing
    else:
        opt("audio", "no")       # frame pipeline only; no clock, mpv free-runs
    opt("hwdec", args.hwdec)
    opt("terminal", "no")
    opt("msg-level", "all=error")
    if args.untimed:
        opt("untimed", "yes")
    if args.start:
        opt("start", args.start)

    r = mpv.mpv_initialize(h)
    if r < 0:
        sys.exit(f"FATAL: mpv_initialize -> {mpv.mpv_error_string(r).decode()}")

    api_type = ctypes.c_char_p(b"sw")
    adv = ctypes.c_int(1)
    params = (RenderParam * 3)()
    params[0].type = PARAM_API_TYPE
    params[0].data = ctypes.cast(api_type, ctypes.c_void_p)
    if args.advanced:
        # production pattern (T-17): mpv tells us when a NEW frame is ready, so
        # every render() call we make is a real frame - clean per-frame numbers.
        params[1].type = PARAM_ADVANCED_CONTROL
        params[1].data = ctypes.cast(ctypes.byref(adv), ctypes.c_void_p)
        params[2].type = PARAM_INVALID
        params[2].data = None
    else:
        params[1].type = PARAM_INVALID
        params[1].data = None

    ctx = ctypes.c_void_p()
    r = mpv.mpv_render_context_create(ctypes.byref(ctx), h, params)
    if r < 0:
        sys.exit(f"FATAL: mpv_render_context_create(sw) -> {mpv.mpv_error_string(r).decode()}")
    print("[2] render context (sw): created")

    # ---- 3. target surface -------------------------------------------------
    w, hgt = args.width, args.height
    stride = w * 4
    stride = (stride + 63) // 64 * 64          # header: keep stride 64-aligned for SIMD
    size = stride * hgt
    buf = mmap.mmap(-1, size)
    ptr = ctypes.addressof(ctypes.c_char.from_buffer(buf))
    if ptr % 64:
        print(f"    ! buffer base not 64-aligned: {ptr % 64}")
    fmt = ctypes.c_char_p(b"rgb0")             # 4 Bpp; matches iced's Rgba8 upload
    sw_size = (ctypes.c_int * 2)(w, hgt)
    sw_stride = ctypes.c_size_t(stride)
    no_block_flag = ctypes.c_int(0)

    def render_params(block=True):
        ps = (RenderParam * 6)()
        ps[0].type = PARAM_SW_SIZE
        ps[0].data = ctypes.cast(sw_size, ctypes.c_void_p)
        ps[1].type = PARAM_SW_FORMAT
        ps[1].data = ctypes.cast(fmt, ctypes.c_void_p)
        ps[2].type = PARAM_SW_STRIDE
        ps[2].data = ctypes.cast(ctypes.byref(sw_stride), ctypes.c_void_p)
        ps[3].type = PARAM_SW_POINTER
        ps[3].data = ctypes.c_void_p(ptr)
        n = 4
        if args.no_block:
            ps[4].type = PARAM_BLOCK_FOR_TARGET_TIME
            ps[4].data = ctypes.cast(ctypes.byref(no_block_flag), ctypes.c_void_p)
            n = 5
        ps[n].type = PARAM_INVALID
        ps[n].data = None
        return ps

    # ---- 4. loadfile -------------------------------------------------------
    argv = (ctypes.c_char_p * 3)(b"loadfile", args.video.encode("utf-8"), None)
    t0 = time.perf_counter()
    r = mpv.mpv_command(h, argv)
    if r < 0:
        sys.exit(f"FATAL: loadfile -> {mpv.mpv_error_string(r).decode()}")

    saw_file_loaded = False
    first_frame_time = None
    while time.perf_counter() - t0 < 20:
        ev = mpv.mpv_wait_event(h, 0.05).contents
        if ev.event_id == EVENT_FILE_LOADED:
            saw_file_loaded = True
        elif ev.event_id == EVENT_END_FILE:
            break
        if ev.event_id in (EVENT_FILE_LOADED, EVENT_VIDEO_RECONFIG) and ctx:
            break
    print(f"[3] FILE_LOADED      : {saw_file_loaded}")

    # video params after reconfig
    MPV_FORMAT_STRING, MPV_FORMAT_INT64, MPV_FORMAT_DOUBLE = 1, 4, 5
    for prop, ctype, fmtid in (("width", ctypes.c_int64, MPV_FORMAT_INT64),
                               ("height", ctypes.c_int64, MPV_FORMAT_INT64),
                               ("estimated-vf-fps", ctypes.c_double, MPV_FORMAT_DOUBLE),
                               ("container-fps", ctypes.c_double, MPV_FORMAT_DOUBLE),
                               ("duration", ctypes.c_double, MPV_FORMAT_DOUBLE),
                               ("hwdec-current", ctypes.c_char_p, MPV_FORMAT_STRING)):
        v = ctype()
        if mpv.mpv_get_property(h, prop.encode(), fmtid, ctypes.byref(v)) == 0:
            val = v.value if not isinstance(v, ctypes.c_char_p) else (v.value or b"").decode()
            print(f"    {prop:18s}: {val}")

    # ---- 5. render loop ----------------------------------------------------
    rpm = render_params()
    rss_before = rss_mb()
    cpu_before = cpu_secs()
    frame_times = []
    render_times = []
    hashes = []
    frames = 0
    t_start = None
    last = time.perf_counter()
    deadline = args.seconds
    rss_peak = rss_before

    while True:
        now = time.perf_counter()
        if args.advanced and t_start is not None:
            # only render when mpv reports a new frame is available
            flags = mpv.mpv_render_context_update(ctx)
            if not (flags & UPDATE_FRAME):
                if now - t_start >= deadline:
                    break
                time.sleep(0.001)
                continue
        r = mpv.mpv_render_context_render(ctx, rpm)
        if r != FORMAT_SUCCESS:
            if t_start is None:
                if time.perf_counter() - t0 > 25:
                    sys.exit("FATAL: no frame within 25 s")
                time.sleep(0.002)
                continue
            break
        ft = time.perf_counter()
        if t_start is None:
            t_start = ft
            first_frame_time = t_start - t0
        frames += 1
        if frames > 1:
            frame_times.append(ft - last)
        render_times.append(ft - now)
        last = ft
        if frames % 30 == 0 or frames == 1:
            # strided sample across the WHOLE surface: hashing only the top rows
            # reports "no change" on letterboxed 4K content (those rows are black).
            hashes.append((frames, hashlib.blake2b(buf[0:size:4096], digest_size=8).digest()))
        if frames % 60 == 0:
            rss_peak = max(rss_peak, rss_mb())
        if ft - t_start >= deadline:
            break
        if frames > 200000:
            break

    elapsed = last - t_start
    rss_after = rss_mb()
    cpu_used = cpu_secs() - cpu_before

    print(f"[4] first frame      : {1000*first_frame_time:.0f} ms after loadfile")
    print(f"[5] frames rendered  : {frames} in {elapsed:.2f} s")
    print(f"    -> effective fps : {frames/elapsed:.2f}")
    if render_times:
        print(f"    render() call    : p50={1000*statistics.median(render_times):.2f} ms  "
              f"mean={1000*statistics.mean(render_times):.2f} ms  "
              f"p95={1000*sorted(render_times)[int(len(render_times)*0.95)-1]:.2f} ms  "
              f"max={1000*max(render_times):.2f} ms")
    if frame_times:
        ft_ms = [1000 * x for x in frame_times]
        print(f"    frame interval   : p50={statistics.median(ft_ms):.2f} ms  "
              f"p95={sorted(ft_ms)[int(len(ft_ms)*0.95)-1]:.2f} ms  "
              f"=> {1000/statistics.median(ft_ms):.1f} fps (p50)")
    uniq = len({hv for _, hv in hashes})
    print(f"[6] distinct frames  : {uniq} of {len(hashes)} sampled (content actually advancing: {uniq > 1})")
    print(f"[7] RSS              : before={rss_before:.1f} MB  after={rss_after:.1f} MB  "
          f"peak_seen={rss_peak:.1f} MB  delta={rss_after-rss_before:+.1f} MB")
    print(f"    per-frame buffer : {size/1e6:.2f} MB at {w}x{hgt} (stride {stride})")
    if frames:
        print(f"[8] CPU (user+krnl)  : {cpu_used:.2f} s over {frames} frames "
              f"=> {1000*cpu_used/frames:.2f} ms CPU/frame, "
              f"{cpu_used/elapsed*100:.0f}% of 1 core")
    print(f"[9] mode             : {'untimed (max throughput)' if args.untimed else 'realtime (timed)'}")
    tp = ctypes.c_double()
    if mpv.mpv_get_property(h, b"time-pos", 5, ctypes.byref(tp)) == 0:
        start_s = 0.0
        if args.start:
            parts = [int(x) for x in args.start.split(":")]
            while len(parts) < 3:
                parts.insert(0, 0)
            start_s = parts[0] * 3600 + parts[1] * 60 + parts[2]
        advanced_s = tp.value - start_s
        print(f"[10] media clock     : start={start_s}s time-pos={tp.value:.2f}s  "
              f"advanced={advanced_s:.2f}s over {elapsed:.2f}s wall "
              f"=> {advanced_s/elapsed:.3f}x realtime (1.0 = keeping up)")

    mpv.mpv_render_context_free(ctx)
    mpv.mpv_terminate_destroy(h)
    print("=== done ===")


if __name__ == "__main__":
    main()
