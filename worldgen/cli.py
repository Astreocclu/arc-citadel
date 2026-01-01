"""CLI interface for worldgen pipeline."""

from pathlib import Path
from typing import Optional

import click

from worldgen import config
from worldgen.storage import Database


@click.group()
def cli():
    """Arc Citadel World Generation Pipeline"""
    pass


@cli.command()
@click.option("--output", default="output", help="Output directory")
def init(output: str):
    """Initialize output directory structure and database."""
    output_path = Path(output)

    dirs = [
        output_path / "libraries",
        output_path / "seeds",
        output_path / "worlds",
        output_path / "logs",
    ]

    for d in dirs:
        d.mkdir(parents=True, exist_ok=True)
        click.echo(f"Created {d}")

    # Initialize database
    db_path = output_path / "libraries" / "assets.db"
    db = Database(db_path)
    db.init()
    db.close()
    click.echo(f"Initialized database at {db_path}")

    click.echo("Initialized output directories")


@cli.command()
@click.option("--db", default=None, help="Database path")
def stats(db: Optional[str]):
    """Show library statistics."""
    if db:
        db_path = Path(db)
    else:
        db_path = config.DATABASE_PATH

    if not db_path.exists():
        click.echo(f"Database not found at {db_path}")
        click.echo("Run 'worldgen init' first")
        return

    database = Database(db_path)
    stats_data = database.get_stats()
    database.close()

    click.echo("Asset Library Statistics:")
    click.echo(f"  Components: {stats_data.get('components', 0)}")
    click.echo(f"  Connectors: {stats_data.get('connectors', 0)}")
    click.echo(f"  Minor anchors: {stats_data.get('minors', 0)}")


@cli.group()
def generate():
    """Generate asset libraries."""
    pass


@generate.command("components")
@click.option("--target-score", default=9.0, help="Minimum quality score")
@click.option("--count", default=100, help="Components per category")
@click.option("--category", default=None, help="Specific category to generate")
@click.option("--db", default=None, help="Database path")
def generate_components(target_score: float, count: int, category: Optional[str], db: Optional[str]):
    """Generate component library using DeepSeek."""
    click.echo(f"Generating components (target score: {target_score})")
    click.echo("Note: Requires DEEPSEEK_API_KEY environment variable")

    if db:
        db_path = Path(db)
    else:
        db_path = config.DATABASE_PATH

    if not db_path.exists():
        click.echo(f"Database not found at {db_path}")
        click.echo("Run 'worldgen init' first")
        return

    # TODO: Implement actual generation
    click.echo("Component generation not yet implemented")
    if category:
        click.echo(f"Would generate {count} components for category: {category}")
    else:
        click.echo(f"Would generate {count} components per category")


if __name__ == "__main__":
    cli()
