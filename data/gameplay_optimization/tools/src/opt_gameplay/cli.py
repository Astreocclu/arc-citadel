"""CLI interface for gameplay optimization tools.

Why Typer: Clean, type-safe CLI with automatic help generation.
Claude can call these commands directly from bash.
"""

import json
import sys
from pathlib import Path
from typing import Optional

import typer

from .evaluator import evaluate_simulation
from .learner import train_proposer
from .proposer import propose_fixes

app = typer.Typer(
    name="opt-gameplay",
    help="LLM-driven gameplay optimization tools for Arc Citadel",
)


@app.command()
def evaluate(
    focus: Path = typer.Option(..., "--focus", "-f", help="Path to focus config JSON"),
    sim_output: Path = typer.Option(..., "--sim-output", "-s", help="Path to simulation output"),
    output: Optional[Path] = typer.Option(None, "--output", "-o", help="Save report to file"),
):
    """Evaluate simulation output against focus expectations.

    Analyzes the simulation log and produces structured verdicts
    (HIT/PARTIAL/MISS) with reasoning for each expectation.

    Example:
        opt-gameplay evaluate -f focuses/action-selection.json -s /tmp/sim.txt
    """
    if not focus.exists():
        typer.echo(f"Error: Focus file not found: {focus}", err=True)
        raise typer.Exit(1)

    if not sim_output.exists():
        typer.echo(f"Error: Simulation output not found: {sim_output}", err=True)
        raise typer.Exit(1)

    typer.echo(f"Evaluating {sim_output.name} against {focus.name}...")

    try:
        report = evaluate_simulation(str(focus), str(sim_output))

        result_json = json.dumps(report, indent=2)

        if output:
            output.write_text(result_json)
            typer.echo(f"Report saved to {output}")
        else:
            typer.echo(result_json)

    except Exception as e:
        typer.echo(f"Error during evaluation: {e}", err=True)
        raise typer.Exit(1)


@app.command()
def propose(
    eval_result: Path = typer.Option(..., "--eval", "-e", help="Path to evaluation report JSON"),
    focus: Path = typer.Option(..., "--focus", "-f", help="Path to focus config JSON"),
    output: Optional[Path] = typer.Option(None, "--output", "-o", help="Save proposals to file"),
):
    """Generate fix proposals for failed expectations.

    Uses DSPy to reason about why expectations failed and
    propose concrete code changes to fix them.

    Example:
        opt-gameplay propose -e /tmp/eval.json -f focuses/action-selection.json
    """
    if not eval_result.exists():
        typer.echo(f"Error: Evaluation result not found: {eval_result}", err=True)
        raise typer.Exit(1)

    if not focus.exists():
        typer.echo(f"Error: Focus file not found: {focus}", err=True)
        raise typer.Exit(1)

    typer.echo("Generating fix proposals...")

    try:
        proposals = propose_fixes(str(eval_result), str(focus))

        result_json = json.dumps(proposals, indent=2)

        if output:
            output.write_text(result_json)
            typer.echo(f"Proposals saved to {output}")
        else:
            typer.echo(result_json)

    except Exception as e:
        typer.echo(f"Error generating proposals: {e}", err=True)
        raise typer.Exit(1)


@app.command()
def learn(
    changelog: Path = typer.Option(
        Path("data/gameplay_optimization/changelog.json"),
        "--changelog", "-c",
        help="Path to changelog JSON"
    ),
):
    """Train the proposer on successful fixes from changelog.

    Analyzes past optimization sessions to learn which types
    of proposals actually improved the simulation. Requires
    at least 3 successful fixes to train.

    Example:
        opt-gameplay learn -c changelog.json
    """
    if not changelog.exists():
        typer.echo(f"Error: Changelog not found: {changelog}", err=True)
        raise typer.Exit(1)

    typer.echo("Analyzing changelog for training examples...")

    try:
        result = train_proposer(str(changelog))

        if result["status"] == "success":
            typer.echo(f"Training complete! Used {result['examples_used']} examples.")
            typer.echo(f"Compiled proposer saved to: {result['saved_to']}")
        elif result["status"] == "skipped":
            typer.echo(f"Training skipped: {result['reason']}")
        else:
            typer.echo(f"Training failed: {result.get('error', 'Unknown error')}", err=True)
            raise typer.Exit(1)

    except Exception as e:
        typer.echo(f"Error during training: {e}", err=True)
        raise typer.Exit(1)


@app.command()
def diff(
    before: Path = typer.Option(..., "--before", "-b", help="Previous evaluation JSON"),
    after: Path = typer.Option(..., "--after", "-a", help="New evaluation JSON"),
):
    """Compare two evaluation reports to see improvement.

    Shows which expectations changed status and the overall
    hit rate delta. Useful for verifying a fix worked.

    Example:
        opt-gameplay diff -b /tmp/eval_v1.json -a /tmp/eval_v2.json
    """
    if not before.exists() or not after.exists():
        typer.echo("Error: Both evaluation files must exist", err=True)
        raise typer.Exit(1)

    with open(before) as f:
        before_data = json.load(f)
    with open(after) as f:
        after_data = json.load(f)

    # Compare hit rates
    before_rate = before_data.get("hit_rate", "?/?")
    after_rate = after_data.get("hit_rate", "?/?")

    typer.echo(f"Hit Rate: {before_rate} -> {after_rate}")
    typer.echo("")

    # Find changed verdicts
    before_verdicts = {v["expectation_id"]: v["verdict"] for v in before_data.get("verdicts", [])}
    after_verdicts = {v["expectation_id"]: v["verdict"] for v in after_data.get("verdicts", [])}

    changes = []
    for exp_id in set(before_verdicts.keys()) | set(after_verdicts.keys()):
        bv = before_verdicts.get(exp_id, "N/A")
        av = after_verdicts.get(exp_id, "N/A")
        if bv != av:
            changes.append(f"  {exp_id}: {bv} -> {av}")

    if changes:
        typer.echo("Changed verdicts:")
        for change in changes:
            typer.echo(change)
    else:
        typer.echo("No verdict changes")


if __name__ == "__main__":
    app()
