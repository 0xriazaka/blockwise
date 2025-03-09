mod parser;

use parser::BlockParser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://eth.llamarpc.com";
    
    println!("Connecting to {}", rpc_url);
    let parser = BlockParser::new(rpc_url).await?;
    println!("✅ Connection successful!");
    
    println!("\nTesting get_block_number()...");
    match parser.get_block_number().await {
        Ok(block_number) => {
            println!("✅ Successfully retrieved latest block number!");
            println!("Current block number: {}", block_number);
        },
        Err(e) => {
            println!("❌ Failed to get block number: {}", e);
        }
    }
    
    Ok(())
}