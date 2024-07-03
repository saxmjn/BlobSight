use std::collections::HashMap;
use std::time::Duration;
use ethers::prelude::*;
use rocksdb::{Options, DB};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Deserialize, Serialize, Debug)]
struct BlobTransaction {
    chain: String,
    tx_hash: H256,
    from: Address,
    to: Address,
    value: U256,
    gas_used: U256,
    gas_price: U256,
    timestamp: u64,
    blob_count: usize,
}

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(short, long)]
    start_block: u64,
    #[structopt(short, long)]
    end_block: u64,
    #[structopt(short, long)]
    rpc_url: String,
}

fn is_blob_transaction(tx: &Transaction) -> bool {
    tx.transaction_type == Some(TransactionType::Eip2930(1))
}

fn count_blobs_in_transaction(tx: &Transaction) -> usize {
    if is_blob_transaction(tx) {
        // The blob count is encoded in the input data after the prefix
        let blob_count_hex = &tx.input[2..];
        usize::from_str_radix(blob_count_hex, 16).unwrap_or(0)
    } else {
        0
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::from_args();

    // Open the RocksDB database
    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DB::open(&opts, "blob_transactions")?;

    // Create a provider from the RPC URL
    let provider = Provider::<Http>::try_from(opts.rpc_url)?;

    // Iterate through the specified block range
    for block_num in opts.start_block..=opts.end_block {
        let block = provider.get_block(block_num).await?;
        for tx in block.transactions {
            // Check if the transaction is a blob transaction
            if is_blob_transaction(&tx) {
                let blob_count = count_blobs_in_transaction(&tx);
                let blob_tx = BlobTransaction {
                    chain: "mainnet".to_string(),
                    tx_hash: tx.hash,
                    from: tx.from.unwrap(),
                    to: tx.to.unwrap(),
                    value: tx.value,
                    gas_used: tx.gas_used.unwrap(),
                    gas_price: tx.gas_price.unwrap(),
                    timestamp: block.timestamp,
                    blob_count,
                };

                // Store the blob transaction in the RocksDB database
                let key = format!("mainnet:{tx_hash:?}", tx_hash = blob_tx.tx_hash);
                db.put(key.as_bytes(), &serde_json::to_vec(&blob_tx)?)?;
            }
        }
    }

    Ok(())
}