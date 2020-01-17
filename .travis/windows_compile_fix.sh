#!/bin/sh
ls /mingw64/
cp -f /mingw64/x86_64-w64-mingw32/lib/{*.a,*.o} `rustc --print sysroot`/lib/rustlib/x86_64-pc-windows-gnu/lib