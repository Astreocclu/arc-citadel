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


@generate.command("world")
@click.option("--seed-id", default=42, help="World seed ID")
@click.option("--radius", default=20, help="World radius in hexes")
@click.option("--output", default="world.json", help="Output file")
def generate_world(seed_id: int, radius: int, output: str):
    """Generate complete world with clusters, connectors, and fillers."""
    from worldgen.assembly.assembler import WorldAssembler
    from worldgen.schemas import WorldSeed, ClusterPlacement, ConnectorAssignment

    click.echo(f"Generating world (seed: {seed_id}, radius: {radius})")

    # Create sample seed
    seed = WorldSeed(
        seed_id=seed_id,
        name=f"World {seed_id}",
        world_radius=radius,
        clusters=[
            ClusterPlacement(template_id='settlement', instance_id='settlement_1'),
            ClusterPlacement(template_id='settlement', instance_id='settlement_2'),
        ],
        connectors=[
            ConnectorAssignment(
                collection_id='trade_route',
                instance_id='route_1',
                start_cluster='settlement_1',
                end_cluster='settlement_2',
            ),
        ],
    )

    assembler = WorldAssembler()
    world = assembler.assemble(seed)

    # Save to file
    output_path = Path(output)
    with open(output_path, 'w') as f:
        f.write(world.model_dump_json(indent=2))

    click.echo(f"Generated world with {world.total_hexes} hexes")
    click.echo(f"Generation time: {world.generation_time_ms}ms")
    click.echo(f"Saved to {output_path}")


@cli.command("test-connector")
@click.option("--start-q", default=0, help="Start Q coordinate")
@click.option("--start-r", default=0, help="Start R coordinate")
@click.option("--end-q", default=10, help="End Q coordinate")
@click.option("--end-r", default=5, help="End R coordinate")
def test_connector(start_q: int, start_r: int, end_q: int, end_r: int):
    """Test connector generation between two points."""
    from worldgen.connector_generator import ConnectorGenerator
    from worldgen.schemas.connector import ConnectorType

    gen = ConnectorGenerator()
    conn = gen.generate(
        ConnectorType.TRADE_ROUTE_MAJOR,
        start_pos=(start_q, start_r),
        end_pos=(end_q, end_r),
        placed_hexes=set(),
    )

    click.echo(f"Generated connector: {conn.id}")
    click.echo(f"  Length: {len(conn.hexes)} hexes")
    click.echo(f"  Anchor slots: {len(conn.minor_slots)}")
    click.echo(f"  Entry points: {len(conn.entry_points)}")


@cli.command("test-filler")
@click.option("--radius", default=3, help="World radius for test")
def test_filler(radius: int):
    """Test filler generation with a small world."""
    from worldgen.filler_generator import FillerGenerator
    from worldgen.schemas.world import HexMap, WorldHex
    from worldgen.schemas.base import HexCoord, Terrain, SpeciesFitness

    # Create hex map with center cluster
    hex_map = HexMap(seed_id=1, world_radius=radius, hexes={}, clusters={}, cluster_positions={})

    # Add fixed hexes at center
    for q in range(-1, 2):
        for r in range(-1, 2):
            if abs(q) + abs(r) <= 1:
                key = f'{q},{r}'
                hex_map.hexes[key] = WorldHex(
                    coord=HexCoord(q=q, r=r),
                    terrain=Terrain.PLAINS,
                    elevation=200.0,
                    moisture=0.5,
                    temperature=15.0,
                    species_fitness=SpeciesFitness(human=0.5, dwarf=0.5, elf=0.5),
                )

    initial_count = len(hex_map.hexes)
    click.echo(f"Initial hexes: {initial_count}")

    gen = FillerGenerator()
    hex_map = gen.generate_fillers(hex_map, world_radius=radius)

    final_count = len(hex_map.hexes)
    click.echo(f"Final hexes: {final_count}")
    click.echo(f"Fillers added: {final_count - initial_count}")


if __name__ == "__main__":
    cli()
