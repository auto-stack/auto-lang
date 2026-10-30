import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import map_inventory


class IntakeAdmission(unittest.TestCase):
    def test_known_sources_admit_unknown_and_duplicate_inputs_reject(self):
        with tempfile.TemporaryDirectory(prefix="p890-inventory-") as directory:
            root = Path(directory)
            source = root / "crates/auto-lang-core/src/http_service_config.rs"
            source.parent.mkdir(parents=True)
            source.write_text("struct HttpServiceConfig;", encoding="utf-8")
            items = root / "items.json"
            items.write_text(json.dumps({"files": [{"path": source.as_posix(), "items": [
                {"kind": "struct", "symbol": "HttpServiceConfig", "attributes": []}
            ]}]}), encoding="utf-8")
            scripts = root / "scripts"
            scripts.mkdir()
            identity = "auto-lang-core\thttp_service_config::tests::valid"
            for tier in ("language", "tt", "tb", "taa", "ta", "t3"):
                (scripts / f"full-regression-{tier}-ids.txt").write_text(identity + "\n", encoding="utf-8")
            output = root / "map.json"
            argv = ["map_inventory", "--items", str(items), "--root", str(root), "--out", str(output)]
            with patch.object(sys, "argv", argv), contextlib.redirect_stdout(io.StringIO()):
                map_inventory.main()
            admitted = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(admitted["source_counts"], {"web-contract": 1})
            self.assertEqual(admitted["obligation_counts"], {"web": 6})
            self.assertEqual(admitted["executed"], 0)
            (scripts / "full-regression-tt-ids.txt").write_text((identity + "\n") * 2, encoding="utf-8")
            with patch.object(sys, "argv", argv), self.assertRaisesRegex(ValueError, "duplicate"):
                map_inventory.main()
            with self.assertRaisesRegex(ValueError, "no intake owner"):
                map_inventory.source_owner("other-repository/unknown.rs", "unclaimed")


if __name__ == "__main__":
    unittest.main()
