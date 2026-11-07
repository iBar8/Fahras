#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Fahras look check on Linux: on a private Xvfb display and D-Bus, changes the
Explorer look and app icon from Settings, checks the open window, the saved
settings and a restart, in one language. Writes screenshots and results to
--evidence and exits non-zero if a check fails."""

import argparse
import ctypes
import ctypes.util
import hashlib
import os
import subprocess
import sys
import time
from pathlib import Path

from PIL import Image

parser = argparse.ArgumentParser()
parser.add_argument("--app", type=Path, required=True)
parser.add_argument("--evidence", type=Path, required=True)
parser.add_argument("--lang", required=True)
args = parser.parse_args()

EVIDENCE = args.evidence.resolve()
EVIDENCE.mkdir(parents=True, exist_ok=True)
HOME = EVIDENCE / f"home-{args.lang}"
FOLDER = EVIDENCE / "folder"
SETTINGS = HOME / ".config/strata/settings.toml"
RTL = args.lang == "ar"
WIDTH, HEIGHT = 1280, 800
results = []
process = None


def check(name, ok):
    results.append(f"{'PASS' if ok else 'FAIL'}  [linux-{args.lang}] {name}")
    print(results[-1], flush=True)


def xdotool(*arguments):
    return subprocess.run(["xdotool", *arguments], capture_output=True, text=True).stdout.strip()


def shot(name):
    path = EVIDENCE / f"linux-{args.lang}-{name}.png"
    subprocess.run(["import", "-window", "root", str(path)], check=True)
    return Image.open(path).convert("RGB")


x11 = ctypes.CDLL(ctypes.util.find_library("X11"))
x11.XOpenDisplay.restype = ctypes.c_void_p
x11.XInternAtom.restype = ctypes.c_ulong
x11.XInternAtom.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int]
x11.XGetWindowProperty.argtypes = [
    ctypes.c_void_p, ctypes.c_ulong, ctypes.c_ulong, ctypes.c_long, ctypes.c_long, ctypes.c_int,
    ctypes.c_ulong, ctypes.POINTER(ctypes.c_ulong), ctypes.POINTER(ctypes.c_int),
    ctypes.POINTER(ctypes.c_ulong), ctypes.POINTER(ctypes.c_ulong), ctypes.POINTER(ctypes.c_void_p),
]


def window_icon(window):
    """A hash of the window's _NET_WM_ICON, the icon X11 panels show."""
    display = x11.XOpenDisplay(None)
    atom = x11.XInternAtom(display, b"_NET_WM_ICON", 0)
    kind, fmt, count, after, data = (ctypes.c_ulong(), ctypes.c_int(), ctypes.c_ulong(), ctypes.c_ulong(), ctypes.c_void_p())
    x11.XGetWindowProperty(display, window, atom, 0, 1 << 24, 0, 0, ctypes.byref(kind), ctypes.byref(fmt),
                           ctypes.byref(count), ctypes.byref(after), ctypes.byref(data))
    if not data.value:
        return None
    return hashlib.sha256(ctypes.string_at(data, count.value * ctypes.sizeof(ctypes.c_long))).hexdigest()[:16]


def launch(label, settings):
    global process
    if settings is not None:
        SETTINGS.parent.mkdir(parents=True, exist_ok=True)
        SETTINGS.write_text(settings + "\n")
    for name in ("Documents", "Reports"):
        (FOLDER / name).mkdir(parents=True, exist_ok=True)
    for name in ("notes.txt", "تقرير.txt"):
        (FOLDER / name).write_text("sample\n")
    runtime = HOME / "run"
    runtime.mkdir(parents=True, exist_ok=True)
    runtime.chmod(0o700)
    environment = {
        **os.environ, "HOME": str(HOME), "XDG_CONFIG_HOME": str(HOME / ".config"),
        "XDG_DATA_HOME": str(HOME / ".local/share"), "XDG_CACHE_HOME": str(HOME / ".cache"),
        "XDG_STATE_HOME": str(HOME / ".local/state"), "XDG_RUNTIME_DIR": str(runtime),
        "GDK_BACKEND": "x11", "NO_AT_BRIDGE": "1", "GTK_A11Y": "none",
    }
    for name in ("WAYLAND_DISPLAY", "XDG_SESSION_TYPE"):
        environment.pop(name, None)
    log = open(EVIDENCE / f"linux-{args.lang}-{label}.log", "w")
    process = subprocess.Popen([str(args.app), str(FOLDER)], env=environment, stdout=log, stderr=subprocess.STDOUT)
    for _ in range(120):
        window = xdotool("search", "--pid", str(process.pid), "--onlyvisible").split("\n")[0]
        if window:
            break
        time.sleep(0.5)
    check(f"{label}: window opens", bool(window))
    xdotool("windowsize", window, str(WIDTH), str(HEIGHT))
    xdotool("windowmove", window, "0", "0")
    time.sleep(4)
    return int(window) if window else 0


def stop():
    process.terminate()
    process.wait(15)
    time.sleep(1)


def x(value):
    return WIDTH - value if RTL else value


def click(left, top, settle=1.2):
    xdotool("mousemove", str(left), str(top), "click", "1")
    time.sleep(settle)


def key(*keys, settle=0.6):
    xdotool("key", *keys)
    time.sleep(settle)


def sidebar(image):
    return "#%02x%02x%02x" % image.getpixel((x(40), int(HEIGHT * 0.7)))


def settings_top():
    """The Settings panel's top edge: the first white row down the middle.
    Explorer's Windows 11 look keeps a tab row above it, moving it down."""
    image = shot("settings-top")
    for top in range(0, 300):
        if min(image.getpixel((WIDTH // 2, top))) >= 250:
            return top
    return None


SETTINGS_BASE = []


def choose(window, query, down, name):
    """Settings search shows the one matching row; its choice is the first in
    the page, mirrored for right to left."""
    xdotool("windowactivate", "--sync", str(window))
    key("ctrl+comma", settle=2.5)
    top = settings_top()
    if not SETTINGS_BASE and top is not None:
        SETTINGS_BASE.append(top)
    shift = top - SETTINGS_BASE[0] if top is not None and SETTINGS_BASE else 0
    print(f"      settings panel at {top} (shift {shift})")
    click(x(170), 108 + shift, 0.6)
    key("ctrl+a")
    xdotool("type", "--delay", "60", query)
    time.sleep(1.5)
    shot(f"{name}-search")
    click(x(640), 241 + shift)
    shot(f"{name}-menu")
    if down:
        key(*(["Down"] * down))
    key("Return", settle=1.5)
    shot(f"{name}-chosen")
    key("Escape", settle=1.5)


def saved(line):
    return line in SETTINGS.read_text()


icons = {}
for icon in ("manila", "forest", "ink", "clay"):
    window = launch(f"icon-{icon}", f'language = "{args.lang}"\napp_icon = "{icon}"')
    icons[icon] = window_icon(window)
    stop()
check("each app icon gives the window its own icon", None not in icons.values() and len(set(icons.values())) == 4)

window = launch("live", f'language = "{args.lang}"\nmode = "theme"\ntheme = "github"\napp_icon = "manila"')
check("starts on Manila", window_icon(window) == icons["manila"])
default = sidebar(shot("01-start"))
for skin, down, expected in (("windows11", 2, "#ffffff"), ("windows10", 1, "#ffffff"), ("off", 0, default)):
    choose(window, "skin", down, f"02-skin-{skin}")
    color = sidebar(shot(f"03-skinned-{skin}"))
    check(f"live switch to {skin}: sidebar {color}, expected {expected}", color == expected)
    stored = saved(f'explorer_skin = "{skin}"') or (skin == "off" and "explorer_skin" not in SETTINGS.read_text())
    check(f"{skin} is saved", stored)
choose(window, "skin", 2, "04-skin-windows11")
choose(window, "manila", 1, "05-icon-forest")
check("the open window shows Forest at once", window_icon(window) == icons["forest"])
check("Forest is saved", saved('app_icon = "forest"'))
shot("06-live-done")
stop()

window = launch("restart", None)
check("Forest after a restart", window_icon(window) == icons["forest"])
check("Windows 11 skin after a restart", sidebar(shot("07-restarted")) == "#ffffff")
stop()

(EVIDENCE / f"linux-look-{args.lang}.txt").write_text("\n".join(results) + "\n")
failures = sum(line.startswith("FAIL") for line in results)
print(f"Failures: {failures}")
sys.exit(failures)
