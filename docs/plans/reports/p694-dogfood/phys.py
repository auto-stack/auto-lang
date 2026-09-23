"""物理输入驱动（PLAN-694 T-03 实机走查）——SendInput 真鼠标/键盘。

桌面窗全屏 1920x1280；截图缩放 2000/1920≈1.0417（截图坐标÷1.0417=实坐标）。
"""
import ctypes
import time

user32 = ctypes.windll.user32

INPUT_MOUSE = 0
INPUT_KEYBOARD = 1
MOUSEEVENTF_MOVE = 0x0001
MOUSEEVENTF_ABSOLUTE = 0x8000
MOUSEEVENTF_LEFTDOWN = 0x0002
MOUSEEVENTF_LEFTUP = 0x0004
KEYEVENTF_UNICODE = 0x0004
KEYEVENTF_KEYUP = 0x0002

VK_RETURN = 0x0D


class _INPUT(ctypes.Structure):
    pass


_MOUSEINPUT = ctypes.Structure
_KEYBDINPUT = ctypes.Structure


class MOUSEINPUT(ctypes.Structure):
    _fields_ = [("dx", ctypes.c_long), ("dy", ctypes.c_long),
                ("mouseData", ctypes.c_ulong), ("dwFlags", ctypes.c_ulong),
                ("time", ctypes.c_ulong), ("dwExtraInfo", ctypes.c_size_t)]


class KEYBDINPUT(ctypes.Structure):
    _fields_ = [("wVk", ctypes.c_ushort), ("wScan", ctypes.c_ushort),
                ("dwFlags", ctypes.c_ulong), ("time", ctypes.c_ulong),
                ("dwExtraInfo", ctypes.c_size_t)]


class UNION(ctypes.Union):
    _fields_ = [("mi", MOUSEINPUT), ("ki", KEYBDINPUT)]


_INPUT._fields_ = [("type", ctypes.c_ulong), ("u", UNION)]


def _mouse(flags, dx=0, dy=0):
    inp = _INPUT()
    inp.type = INPUT_MOUSE
    inp.u.mi = MOUSEINPUT(dx, dy, 0, flags, 0, 0)
    user32.SendInput(1, ctypes.byref(inp), ctypes.sizeof(inp))


def _key(wvk=0, scan=0, flags=0):
    inp = _INPUT()
    inp.type = INPUT_KEYBOARD
    inp.u.ki = KEYBDINPUT(wvk, scan, flags, 0, 0)
    user32.SendInput(1, ctypes.byref(inp), ctypes.sizeof(inp))


def mouse_move(x, y):
    ax = int(x * 65535 / 1919)
    ay = int(y * 65535 / 1279)
    _mouse(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, ax, ay)
    time.sleep(0.08)


def click(x, y):
    mouse_move(x, y)
    time.sleep(0.12)
    _mouse(MOUSEEVENTF_LEFTDOWN)
    time.sleep(0.06)
    _mouse(MOUSEEVENTF_LEFTUP)
    time.sleep(0.2)


def type_text(text):
    for ch in text:
        code = ord(ch)
        _key(scan=code, flags=KEYEVENTF_UNICODE)
        time.sleep(0.03)
        _key(scan=code, flags=KEYEVENTF_UNICODE | KEYEVENTF_KEYUP)
        time.sleep(0.09)


def press_return():
    _key(wvk=VK_RETURN)
    time.sleep(0.05)
    _key(wvk=VK_RETURN, flags=KEYEVENTF_KEYUP)
    time.sleep(0.15)
