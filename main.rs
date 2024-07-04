extern crate web3;
extern crate serde_json;
extern crate plotters;
#[macro_use]
extern crate structopt;

use std::collections::{HashMap};
use web3::types::{BlockNumber, Bytes, H160, H256, U256, U64, H64, Transaction};
use web3::transports::Http;
use web3::Web3;
use structopt::StructOpt;
use std::convert::TryInto;

#[derive(StructOpt)]
struct Cli {
    /// Start block number
    #[structopt(short, long)]
    start_block: u64,

    /// End block number
    #[structopt(short, long)]
    end_block: u64,

    /// RPC URL
    #[structopt(short, long)]
    rpc_url: String,
}

#[derive(Debug, Clone)]
struct Block {
    pub number: U256,
    pub hash: H256,
    pub parent_hash: H256,
    pub nonce: Option<H64>,
    pub logs_bloom: Option<Bytes>,
    pub transactions_root: H256,
    pub state_root: H256,
    pub receipts_root: H256,
    // pub author: H160,
    pub difficulty: U256,
    pub total_difficulty: U256,
    pub extra_data: Bytes,
    pub size: Option<U256>,
    pub gas_limit: U256,
    pub gas_used: U256,
    pub timestamp: U256,
    pub base_fee_per_gas: Option<U256>,
    pub transactions: Vec<Transaction>,
    pub uncles: Vec<H256>,
}

#[derive(Debug, Clone)]
struct TransactionMeta {
    pub transaction: Transaction,
    pub blob_count: u64,
    pub blob_size: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let cli = Cli::from_args();

    // Connect to an Ethereum node
    let transport = Http::new(&cli.rpc_url)?;
    let web3 = Web3::new(transport);

    // Get the block data
    let blocks = get_blocks(&web3, cli.start_block, cli.end_block).await?;

    // Print the distribution of transactions for blobs
    // print_blob_tx_distribution(&blocks);

    Ok(())
}

async fn get_blocks(
    web3: &Web3<Http>,
    start_block: u64,
    end_block: u64,
) -> Result<Vec<Block>, Box<dyn std::error::Error>> {
    let mut blocks = Vec::new();
    for block_num in start_block..=end_block {
        let block = get_block(web3, block_num).await?;
        blocks.push(block);
    }
    Ok(blocks)
}

async fn get_block(web3: &Web3<Http>, block_num: u64) -> Result<Block, Box<dyn std::error::Error>> {
    use web3::types::BlockId;
    let block = web3.eth().block(BlockId::Number(BlockNumber::Number(block_num.into()))).await?;
    if let Some(block) = block {
        let transactions = get_transactions(web3, &block).await?;
        let block_number = U256::from(block.number.unwrap().as_u64());
        let block_data = Block {
            number: block_number,
            hash: block.hash.unwrap(),
            parent_hash: block.parent_hash,
            nonce: block.nonce,
            logs_bloom: block.logs_bloom.map(|b| Bytes::from(b.0)),
            transactions_root: block.transactions_root,
            state_root: block.state_root,
            receipts_root: block.receipts_root,
            // author: block.author.unwrap_or_else(H160::zero),
            difficulty: block.difficulty,
            total_difficulty: block.total_difficulty.unwrap(),
            extra_data: block.extra_data,
            size: block.size.map(U256::from),
            gas_limit: block.gas_limit,
            gas_used: block.gas_used,
            timestamp: block.timestamp,
            base_fee_per_gas: block.base_fee_per_gas,
            transactions,
            uncles: block.uncles,
        };
        let block_number_u64 = block_data.number.as_u64();
        let gas_used_u64 = block_data.gas_used.as_u64();
        let block_size_kb = block_data.size.map_or(0.0, |s| (s.as_u64() as f64) / 1024.0);
        let num_transactions = block_data.transactions.len();
        println!(
            "Processed block number: {}, Gas used: {}, Block size: {:.2} KB, Number of transactions: {}",
            block_number_u64, gas_used_u64, block_size_kb, num_transactions
        );
        Ok(block_data)
    } else {
        Err("Block not found".into())
    }
}

async fn get_transactions(
    web3: &Web3<Http>,
    block: &web3::types::Block<H256>,
) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
    let mut transactions = Vec::new();
    for tx_hash in &block.transactions {
        let tx = web3.eth().transaction(web3::types::TransactionId::Hash(*tx_hash)).await?;
        if let Some(tx) = tx {
            // let (blob_count, blob_size) = get_blob_properties(&tx.input.0)?;
            transactions.push(tx);
        }
    }
    Ok(transactions)
}

fn get_blob_properties(input: &[u8]) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let blob_count = U256::from(&input[0..32]);
    let blob_size = U256::from(&input[32..64]);
    let blob_count_u64: u64 = blob_count.try_into().map_err(|_| "Integer overflow when casting blob count to u64")?;
    let blob_size_u64: u64 = blob_size.try_into().map_err(|_| "Integer overflow when casting blob size to u64")?;
    Ok((blob_count_u64, blob_size_u64))
}

fn is_blob_transaction(tx: &Transaction) -> bool {
    if let Some(tx_type) = tx.transaction_type {
        tx_type == U64::from(3) // Assuming 3 is the type for blob transactions
    } else {
        false
    }
}

// fn print_blob_tx_distribution(blocks: &[Block]) {
//     let mut blob_tx_counts: HashMap<(u64, u64), u64> = HashMap::new();

//     for block in blocks {
//         for tx_meta in &block.transactions {
//             if is_blob_transaction(&tx_meta.transaction) {
//                 let key = (tx_meta.blob_count, tx_meta.blob_size);
//                 *blob_tx_counts.entry(key).or_insert(0) += 1;
//             }
//         }
//     }

//     println!("Distribution of BlobTransactions:");
//     for ((blob_count, blob_size), count) in blob_tx_counts {
//         println!("{} blobs of size {}: {}", blob_count, blob_size, count);
//     }
// }
