"""T-15 补充探针：mpv 是否真能解出 Chromium 解不了的那条音轨。

Chromium 端实测：`Loki.S02E01…mkv` 的 DDP5.1/Atmos 音轨解不出来
（webkitAudioDecodedByteCount 恒 0），被判为 video-only。用户要求 VM 端能播，
主要动机就是这条音轨。故此处实测 mpv（内链 ffmpeg）对它的解码结果——
只读 mpv 自己报告的属性，不做推断。
"""

import ctypes
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sw_probe import bind, rss_mb  # noqa: E402

DLL = os.environ.get("AUTO_MPV_LIB") or r"libmpv-2.dll"
VIDEO = os.environ.get("AUTO_SPIKE_VIDEO") or (
    r"E:\Video\TV\Loki\Loki.S02E01.2021.2160p.DSNP.WEB-DL.DDP5.1.Atmos.HDR.H.265-FLUX.mkv"
)

EVENT_FILE_LOADED = 8
EVENT_VIDEO_RECONFIG = 17
EVENT_AUDIO_RECONFIG = 18


def get(h, mpv, name, ctype, fmtid):
    v = ctype()
    r = mpv.mpv_get_property(h, name.encode(), fmtid, ctypes.byref(v))
    if r != 0:
        return f"<{name} unavailable (code {r})>"
    if isinstance(v, ctypes.c_char_p):
        return (v.value or b"").decode() or "<empty>"
    return v.value


def main():
    mpv = ctypes.CDLL(DLL)
    bind(mpv)
    mpv.mpv_observe_property.argtypes = [
        ctypes.c_void_p, ctypes.c_uint64, ctypes.c_char_p, ctypes.c_int
    ]

    h = mpv.mpv_create()
    for k, v in [
        ("vo", "libmpv"),
        ("ao", "null"),      # 保留音频管线（要真解码），但不要求声卡
        ("terminal", "no"),
        ("msg-level", "all=error"),
        ("untimed", "yes"),
    ]:
        mpv.mpv_set_option_string(h, k.encode(), str(v).encode())
    if mpv.mpv_initialize(h) < 0:
        sys.exit("mpv_initialize failed")

    argv = (ctypes.c_char_p * 3)(b"loadfile", VIDEO.encode("utf-8"), None)
    mpv.mpv_command(h, argv)

    saw = {}
    t0 = time.time()
    while time.time() - t0 < 20:
        ev = mpv.mpv_wait_event(h, 0.05).contents
        if ev.event_id in (EVENT_FILE_LOADED, EVENT_VIDEO_RECONFIG, EVENT_AUDIO_RECONFIG):
            saw[ev.event_id] = saw.get(ev.event_id, 0) + 1
        if EVENT_AUDIO_RECONFIG in saw and EVENT_VIDEO_RECONFIG in saw:
            break
    # 让音频解码真的跑起来
    time.sleep(2.0)

    print("=== mpv 对 DDP5.1/Atmos 音轨的实测解码情况 ===")
    print(f"AUDIO_RECONFIG 事件 : {saw.get(EVENT_AUDIO_RECONFIG, 0)} 次"
          f"（0 次说明 mpv 根本没建音频轨）")
    for name, ctype, fmtid in [
        ("audio-codec-name", ctypes.c_char_p, 1),
        ("audio-params", ctypes.c_char_p, 1),
        ("audio-out-params", ctypes.c_char_p, 1),
        ("current-ao", ctypes.c_char_p, 1),
        ("aid", ctypes.c_int64, 4),
        ("audio-bitrate", ctypes.c_double, 5),
        ("audio-samplerate", ctypes.c_int64, 4),
        ("audio-channels", ctypes.c_char_p, 1),
        ("video-codec", ctypes.c_char_p, 1),
        ("hwdec-current", ctypes.c_char_p, 1),
    ]:
        print(f"{name:20s}: {get(h, mpv, name, ctype, fmtid)}")
    print(f"{'RSS':20s}: {rss_mb():.0f} MB")
    mpv.mpv_terminate_destroy(h)


if __name__ == "__main__":
    main()
