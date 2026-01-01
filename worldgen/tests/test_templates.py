"""Tests for template loading."""

from pathlib import Path
import tempfile

import pytest

from worldgen.templates.template_loader import TemplateLoader
from worldgen.schemas import ClusterTemplate, Species


class TestTemplateLoader:
    def test_load_single_template(self):
        loader = TemplateLoader()
        template = loader.load_template("dwarf/hold_major")
        assert template is not None
        assert template.id == "dwarf_hold_major"
        assert template.species == Species.DWARF

    def test_load_all_templates(self):
        loader = TemplateLoader()
        templates = loader.load_all()
        assert len(templates) >= 1
        assert "dwarf_hold_major" in templates
