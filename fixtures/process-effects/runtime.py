#!/usr/bin/python3
"""Synthetic OS process mechanics only: no provider/authentication protocol."""
import os, pathlib, signal, sys, time
mode = sys.argv[1]
root = pathlib.Path(sys.argv[2])
if mode == 'finite':
    os.write(1, b'part'); time.sleep(.01); os.write(1, b'ial\n')
elif mode == 'nonzero':
    os.write(2, b'exact failure\n'); sys.exit(3)
elif mode == 'linger-zero':
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))
    os.close(1); os.close(2)
    time.sleep(30)
elif mode == 'saturate':
    for _ in range(128):
        os.write(1, b'o' * 8192); os.write(2, b'e' * 8192)
elif mode == 'stdin':
    os.write(2, b'e' * 131072)
    data = sys.stdin.buffer.read()
    os.write(1, str(len(data)).encode())
elif mode in ('descendant', 'escaped', 'parent-loss'):
    if mode != 'parent-loss':
        pid = os.fork()
        if pid: sys.exit(0)
    if mode == 'escaped': os.setsid()
    signal.signal(signal.SIGTERM, signal.SIG_IGN)
    (root / (mode + '.pid')).write_text(str(os.getpid()))
    while not (root / (mode + '.release')).exists(): time.sleep(.02)
