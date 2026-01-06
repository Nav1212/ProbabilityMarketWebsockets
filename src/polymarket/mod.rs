//! Polymarket module - Client implementation for Polymarket CLOB API

pub mod auth;
pub mod client;
pub mod messages;
pub mod rest;
pub mod websocket;

pub use client::PolymarketClient;
