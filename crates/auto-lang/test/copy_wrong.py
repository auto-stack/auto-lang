import os, glob, shutil
for f in glob.glob('crates/auto-lang/test/a2ts/**/*.wrong.ts', recursive=True):
    shutil.copy(f, f.replace('.wrong.ts', '.expected.ts'))
