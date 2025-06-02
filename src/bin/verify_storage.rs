//! Binary to verify storage implementation

use ia::contexts::graph_management::verify_storage::verify_storage_operations;

fn main() {
    println!("Running Storage Verification...\n");
    verify_storage_operations();
    println!("\nStorage verification complete!");
}
