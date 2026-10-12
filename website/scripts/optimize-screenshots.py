"""Encode local screenshot delivery variants; leave capture originals untouched."""
from pathlib import Path
from io import BytesIO
import hashlib
import json
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
PUBLIC = ROOT / 'public'
LIMIT = 1_000_000
rows = []
variants = {}
for folder in ['desktop-showcase', 'apps', 'v05']:
    for source in sorted((PUBLIC / folder).rglob('*')):
        if source.suffix.lower() not in ['.png', '.jpg', '.jpeg'] or 'candidate' in source.parts or source.stat().st_size < 200_000:
            continue
        original = source.read_bytes()
        with Image.open(source) as capture:
            image = capture.convert('RGBA' if 'A' in capture.getbands() else 'RGB')
            for quality in [95, 90, 85, 80]:
                encoded = BytesIO()
                image.save(encoded, format='WEBP', quality=quality, method=6)
                data = encoded.getvalue()
                if len(data) < LIMIT:
                    break
            else:
                raise RuntimeError(f'{source}: budget requires a reviewed resize or lower quality')
            assert len(data) < len(original), f'{source}: encoding increases bytes'
            output = source.with_suffix('.webp')
            output.write_bytes(data)
            key = '/' + source.relative_to(PUBLIC).as_posix()
            variants[key] = '/' + output.relative_to(PUBLIC).as_posix()
            rows.append(dict(source=key, output=variants[key], width=image.width, height=image.height,
                             original_bytes=len(original), bytes=len(data), quality=quality,
                             source_sha256=hashlib.sha256(original).hexdigest(), sha256=hashlib.sha256(data).hexdigest()))
(ROOT / '.vitepress/theme/data/screenshot-variants.json').write_text(json.dumps(variants, indent=2) + '\n', encoding='utf-8')
report = dict(plan='PLAN-769', encoding='Lossy WebP, original dimensions, method=6; no crop/repaint/resize',
              budget_bytes=LIMIT, original_bytes=sum(r['original_bytes'] for r in rows),
              bytes=sum(r['bytes'] for r in rows), assets=rows)
(ROOT.parent / 'docs/reports/p769-image-catalog.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(json.dumps({k: v for k, v in report.items() if k != 'assets'}))
print(f'{len(rows)} variants; originals preserved')
