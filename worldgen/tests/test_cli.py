"""Tests for CLI interface."""

from click.testing import CliRunner
from pathlib import Path

from worldgen.cli import cli


class TestCLI:
    def test_cli_exists(self):
        """Test that CLI help works."""
        runner = CliRunner()
        result = runner.invoke(cli, ["--help"])
        assert result.exit_code == 0
        assert "Arc Citadel World Generation Pipeline" in result.output

    def test_init_command(self):
        """Test init command creates directories and database."""
        runner = CliRunner()
        with runner.isolated_filesystem():
            result = runner.invoke(cli, ["init", "--output", "test_output"])
            assert result.exit_code == 0
            assert "Initialized" in result.output

            # Verify directories were created
            assert Path("test_output/libraries").exists()
            assert Path("test_output/seeds").exists()
            assert Path("test_output/worlds").exists()
            assert Path("test_output/logs").exists()

            # Verify database was created
            assert Path("test_output/libraries/assets.db").exists()

    def test_init_command_default_output(self):
        """Test init command with default output directory."""
        runner = CliRunner()
        with runner.isolated_filesystem():
            result = runner.invoke(cli, ["init"])
            assert result.exit_code == 0
            assert "Initialized" in result.output
            assert Path("output/libraries/assets.db").exists()

    def test_stats_command(self):
        """Test stats command shows library statistics."""
        runner = CliRunner()
        with runner.isolated_filesystem():
            # Init first
            runner.invoke(cli, ["init", "--output", "test_output"])

            # Then get stats
            result = runner.invoke(cli, ["stats", "--db", "test_output/libraries/assets.db"])
            assert result.exit_code == 0
            assert "Asset Library Statistics:" in result.output
            assert "Components:" in result.output
            assert "Connectors:" in result.output
            assert "Minor anchors:" in result.output

    def test_stats_command_no_db(self):
        """Test stats command when database doesn't exist."""
        runner = CliRunner()
        with runner.isolated_filesystem():
            result = runner.invoke(cli, ["stats", "--db", "nonexistent.db"])
            assert result.exit_code == 0
            assert "Database not found" in result.output
            assert "Run 'worldgen init' first" in result.output

    def test_generate_command_group(self):
        """Test generate command group exists."""
        runner = CliRunner()
        result = runner.invoke(cli, ["generate", "--help"])
        assert result.exit_code == 0
        assert "Generate asset libraries" in result.output

    def test_generate_components_command(self):
        """Test generate components command exists."""
        runner = CliRunner()
        with runner.isolated_filesystem():
            # Init first
            runner.invoke(cli, ["init", "--output", "test_output"])

            result = runner.invoke(cli, [
                "generate", "components",
                "--db", "test_output/libraries/assets.db",
                "--count", "10",
                "--category", "dwarf_hold_forge"
            ])
            assert result.exit_code == 0
            assert "Component generation not yet implemented" in result.output
            assert "dwarf_hold_forge" in result.output

    def test_generate_components_no_db(self):
        """Test generate components when database doesn't exist."""
        runner = CliRunner()
        with runner.isolated_filesystem():
            result = runner.invoke(cli, [
                "generate", "components",
                "--db", "nonexistent.db"
            ])
            assert result.exit_code == 0
            assert "Database not found" in result.output
