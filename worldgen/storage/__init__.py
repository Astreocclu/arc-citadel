"""Storage layer for worldgen assets."""

from .database import AssetDatabase

Database = AssetDatabase  # Alias for plan compatibility

__all__ = ["AssetDatabase", "Database"]
