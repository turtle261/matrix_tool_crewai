"""
Matrix Tool for CrewAI.

This module provides tools for interacting with Matrix rooms,
allowing CrewAI agents to join rooms, watch for messages,
redact inappropriate content, and ban users.
"""

from .matrix_tool import MatrixTool

__all__ = ["MatrixTool"] 