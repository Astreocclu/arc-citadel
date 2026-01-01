"""SQLite database for asset storage."""

import json
import sqlite3
from pathlib import Path
from typing import Optional

from worldgen.schemas import (
    Component,
    ComponentCategory,
    ConnectorCollection,
    ConnectorType,
    MinorAnchor,
    MinorCategory,
)


class AssetDatabase:
    """SQLite database for worldgen assets.

    Stores Pydantic models as JSON in the data column.
    Uses raw sqlite3 (no ORM).
    """

    def __init__(self, db_path: Path):
        self.db_path = db_path
        self._conn: Optional[sqlite3.Connection] = None

    @property
    def conn(self) -> sqlite3.Connection:
        if self._conn is None:
            self.db_path.parent.mkdir(parents=True, exist_ok=True)
            self._conn = sqlite3.connect(self.db_path)
            self._conn.row_factory = sqlite3.Row
        return self._conn

    def init(self) -> None:
        """Initialize database schema."""
        self.conn.executescript("""
            CREATE TABLE IF NOT EXISTS components (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                species TEXT NOT NULL,
                tags TEXT NOT NULL,
                data TEXT NOT NULL,
                quality_score REAL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS connectors (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL,
                tags TEXT NOT NULL,
                data TEXT NOT NULL,
                quality_score REAL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS minors (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                tags TEXT NOT NULL,
                data TEXT NOT NULL,
                quality_score REAL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_comp_category ON components(category);
            CREATE INDEX IF NOT EXISTS idx_comp_species ON components(species);
            CREATE INDEX IF NOT EXISTS idx_conn_type ON connectors(type);
            CREATE INDEX IF NOT EXISTS idx_minor_category ON minors(category);
        """)
        self.conn.commit()

    def list_tables(self) -> list[str]:
        """List all tables in the database."""
        cursor = self.conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table'"
        )
        return [row["name"] for row in cursor.fetchall()]

    # =========================================================================
    # Component methods
    # =========================================================================

    def save_component(self, component: Component) -> None:
        """Save a component to the database (insert or replace)."""
        self.conn.execute(
            """
            INSERT OR REPLACE INTO components (id, category, species, tags, data, quality_score)
            VALUES (?, ?, ?, ?, ?, ?)
            """,
            (
                component.id,
                component.category.value,
                component.species.value,
                json.dumps(component.tags),
                component.model_dump_json(),
                component.quality_score,
            ),
        )
        self.conn.commit()

    def get_component(self, component_id: str) -> Optional[Component]:
        """Get a component by ID."""
        cursor = self.conn.execute(
            "SELECT data FROM components WHERE id = ?", (component_id,)
        )
        row = cursor.fetchone()
        if row:
            return Component.model_validate_json(row["data"])
        return None

    def list_components(
        self,
        category: Optional[ComponentCategory] = None,
        species: Optional[str] = None,
        min_quality: Optional[float] = None,
        limit: int = 100,
    ) -> list[Component]:
        """List components with optional filters."""
        query = "SELECT data FROM components WHERE 1=1"
        params: list = []

        if category:
            query += " AND category = ?"
            params.append(category.value)
        if species:
            query += " AND species = ?"
            params.append(species)
        if min_quality:
            query += " AND quality_score >= ?"
            params.append(min_quality)

        query += " LIMIT ?"
        params.append(limit)

        cursor = self.conn.execute(query, params)
        return [Component.model_validate_json(row["data"]) for row in cursor.fetchall()]

    def delete_component(self, component_id: str) -> bool:
        """Delete a component by ID. Returns True if deleted."""
        cursor = self.conn.execute(
            "DELETE FROM components WHERE id = ?", (component_id,)
        )
        self.conn.commit()
        return cursor.rowcount > 0

    # =========================================================================
    # Connector methods
    # =========================================================================

    def save_connector(self, connector: ConnectorCollection) -> None:
        """Save a connector to the database (insert or replace)."""
        self.conn.execute(
            """
            INSERT OR REPLACE INTO connectors (id, type, tags, data, quality_score)
            VALUES (?, ?, ?, ?, ?)
            """,
            (
                connector.id,
                connector.type.value,
                json.dumps(connector.tags),
                connector.model_dump_json(),
                connector.quality_score,
            ),
        )
        self.conn.commit()

    def get_connector(self, connector_id: str) -> Optional[ConnectorCollection]:
        """Get a connector by ID."""
        cursor = self.conn.execute(
            "SELECT data FROM connectors WHERE id = ?", (connector_id,)
        )
        row = cursor.fetchone()
        if row:
            return ConnectorCollection.model_validate_json(row["data"])
        return None

    def list_connectors(
        self,
        connector_type: Optional[ConnectorType] = None,
        min_quality: Optional[float] = None,
        limit: int = 100,
    ) -> list[ConnectorCollection]:
        """List connectors with optional filters."""
        query = "SELECT data FROM connectors WHERE 1=1"
        params: list = []

        if connector_type:
            query += " AND type = ?"
            params.append(connector_type.value)
        if min_quality:
            query += " AND quality_score >= ?"
            params.append(min_quality)

        query += " LIMIT ?"
        params.append(limit)

        cursor = self.conn.execute(query, params)
        return [ConnectorCollection.model_validate_json(row["data"]) for row in cursor.fetchall()]

    def delete_connector(self, connector_id: str) -> bool:
        """Delete a connector by ID. Returns True if deleted."""
        cursor = self.conn.execute(
            "DELETE FROM connectors WHERE id = ?", (connector_id,)
        )
        self.conn.commit()
        return cursor.rowcount > 0

    # =========================================================================
    # Minor anchor methods
    # =========================================================================

    def save_minor(self, minor: MinorAnchor) -> None:
        """Save a minor anchor to the database (insert or replace)."""
        self.conn.execute(
            """
            INSERT OR REPLACE INTO minors (id, category, tags, data, quality_score)
            VALUES (?, ?, ?, ?, ?)
            """,
            (
                minor.id,
                minor.category.value,
                json.dumps(minor.tags),
                minor.model_dump_json(),
                minor.quality_score,
            ),
        )
        self.conn.commit()

    def get_minor(self, minor_id: str) -> Optional[MinorAnchor]:
        """Get a minor anchor by ID."""
        cursor = self.conn.execute(
            "SELECT data FROM minors WHERE id = ?", (minor_id,)
        )
        row = cursor.fetchone()
        if row:
            return MinorAnchor.model_validate_json(row["data"])
        return None

    def list_minors(
        self,
        category: Optional[MinorCategory] = None,
        min_quality: Optional[float] = None,
        limit: int = 100,
    ) -> list[MinorAnchor]:
        """List minor anchors with optional filters."""
        query = "SELECT data FROM minors WHERE 1=1"
        params: list = []

        if category:
            query += " AND category = ?"
            params.append(category.value)
        if min_quality:
            query += " AND quality_score >= ?"
            params.append(min_quality)

        query += " LIMIT ?"
        params.append(limit)

        cursor = self.conn.execute(query, params)
        return [MinorAnchor.model_validate_json(row["data"]) for row in cursor.fetchall()]

    def delete_minor(self, minor_id: str) -> bool:
        """Delete a minor anchor by ID. Returns True if deleted."""
        cursor = self.conn.execute(
            "DELETE FROM minors WHERE id = ?", (minor_id,)
        )
        self.conn.commit()
        return cursor.rowcount > 0

    # =========================================================================
    # Statistics and utility methods
    # =========================================================================

    def get_stats(self) -> dict:
        """Get database statistics."""
        stats = {}
        for table in ["components", "connectors", "minors"]:
            cursor = self.conn.execute(f"SELECT COUNT(*) as count FROM {table}")
            stats[table] = cursor.fetchone()["count"]
        return stats

    def close(self) -> None:
        """Close database connection."""
        if self._conn:
            self._conn.close()
            self._conn = None
