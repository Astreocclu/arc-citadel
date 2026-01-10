"""Pydantic schemas for worldgen."""

from .base import (
    Terrain,
    ResourceType,
    Abundance,
    FeatureType,
    Species,
    Resource,
    Feature,
    SpeciesFitness,
    HexCoord,
    generate_stable_id,
)
from .component import Component, ComponentCategory, ConnectionPoint
from .template import (
    SlotCount,
    ClusterSlot,
    InternalConnection,
    ExternalPort,
    ClusterTemplate,
    AssembledCluster,
)
from .connector import (
    ConnectorType,
    EntryPoint,
    MinorAnchorSlot,
    ElasticSegment,
    ConnectorHex,
    ConnectorCollection,
)
from .minor import MinorCategory, MinorAnchor
from .seed import ClusterPlacement, ConnectorAssignment, LayoutHint, WorldSeed
from .world import WorldHex, HexMap, AssembledWorld
from .tagged import EdgeType, FoundingContext, TaggedHex, HexCluster, ScaleValidation

__all__ = [
    # base
    "Terrain",
    "ResourceType",
    "Abundance",
    "FeatureType",
    "Species",
    "Resource",
    "Feature",
    "SpeciesFitness",
    "HexCoord",
    "generate_stable_id",
    # component
    "Component",
    "ComponentCategory",
    "ConnectionPoint",
    # template
    "SlotCount",
    "ClusterSlot",
    "InternalConnection",
    "ExternalPort",
    "ClusterTemplate",
    "AssembledCluster",
    # connector
    "ConnectorType",
    "EntryPoint",
    "MinorAnchorSlot",
    "ElasticSegment",
    "ConnectorHex",
    "ConnectorCollection",
    # minor
    "MinorCategory",
    "MinorAnchor",
    # seed
    "ClusterPlacement",
    "ConnectorAssignment",
    "LayoutHint",
    "WorldSeed",
    # world
    "WorldHex",
    "HexMap",
    "AssembledWorld",
    # tagged (100m scale composition)
    "EdgeType",
    "FoundingContext",
    "TaggedHex",
    "HexCluster",
    "ScaleValidation",
]
