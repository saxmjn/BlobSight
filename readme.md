# BlobSight

Real-time Ethereum L2 and blob transaction monitor with proto-danksharding support.

## Overview

BlobSight is a high-performance Ethereum monitoring tool designed to track Layer 2 (L2) transactions and upcoming blob data in real-time. Built with Rust, it offers robust handling of Ethereum WebSocket connections, efficient transaction filtering, and rapid data storage using RocksDB.

## Features

- Real-time monitoring of Ethereum for L2-related transactions
- Support for Optimism, Arbitrum, and zkSync
- Ready for EIP-4844 (proto-danksharding) blob transactions
- Concurrent transaction processing
- Efficient data storage with RocksDB
- RESTful API for querying transaction data and statistics

## Prerequisites

- Rust 1.54 or higher
- An Ethereum node with WebSocket support (e.g., Infura, Alchemy)

## Setup
1. Install Cargo 
