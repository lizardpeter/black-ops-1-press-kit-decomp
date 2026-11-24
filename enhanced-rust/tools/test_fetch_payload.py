import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from fetch_payload import validate_manifest, target_for, verify_file

class PayloadManifestTests(unittest.TestCase):
    def test_inventory_count_and_size(self):
        path = Path(__file__).with_name("payload-manifest.json")
        doc = json.loads(path.read_text(encoding="utf-8"))
        self.assertEqual(len(doc["files"]), 44)
        self.assertEqual(validate_manifest(doc), 409146676)

    def test_rejects_traversal_and_absolute_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for rel in ("../escape.mp4", "/absolute.mp4", "a//b", "a/./b", "a\\b"):
                with self.subTest(path=rel):
                    with self.assertRaises(ValueError):
                        target_for(root, rel)

    def test_verifies_file_size_and_md5(self):
        data = b"original payload fixture"
        item = {"size": len(data), "etag": hashlib.md5(data, usedforsecurity=False).hexdigest()}
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "fixture.bin"
            path.write_bytes(data)
            self.assertTrue(verify_file(path, item))
            path.write_bytes(data + b"x")
            self.assertFalse(verify_file(path, item))

if __name__ == "__main__":
    unittest.main()
