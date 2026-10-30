"""Explicit cross-repository test ownership; no arbitrary parent traversal."""
import os
from pathlib import Path

WEB_RECIPES = frozenset(("web-unit", "web-http", "web-generation"))


def owned_path(root, path):
    root, path = Path(root).resolve(), Path(path).resolve()
    return path.is_relative_to(root) or path.is_relative_to(root.parent / "auto-web")


def relative_path(root, path):
    if not owned_path(root, path):
        raise ValueError("test source outside authorized language/Web repositories")
    return Path(os.path.relpath(Path(path).resolve(), Path(root).resolve())).as_posix()


def workspace_root(root, recipe):
    root = Path(root).resolve()
    workspace = (root / recipe["workspace"]).resolve()
    owner = recipe.get("repository", "language")
    if owner not in ("language", "web"):
        raise ValueError("unknown recipe repository owner")
    authorized = root if owner == "language" else root.parent / "auto-web"
    if not workspace.is_relative_to(authorized):
        raise ValueError("recipe escapes its declared repository")
    return workspace
