//! Integration tests for the Metis MCP Server
//!
//! This module organizes tests by functionality area to keep them manageable.
//!
//! ## Test Organization:
//! - `tools_basic`: Basic tool availability and initialization tests
//! - `document_lifecycle`: Document creation, validation, and phase transitions
//! - `update_operations`: Content updates, exit criteria, and relationship updates
//! - `search_and_query`: Document listing, searching, and querying
//! - `obsidian_integration`: Obsidian vault opening and configuration
//! - `error_handling`: Error cases, edge cases, and constraint handling
//! - `common`: Shared utilities and helper functions

mod common;
mod document_lifecycle;
mod error_handling;
mod obsidian_integration;
mod search_and_query;
mod tools_basic;
mod update_operations;

// Re-export the common test utilities for easy access
pub use common::*;
