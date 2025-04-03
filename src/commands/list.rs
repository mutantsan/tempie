use crate::api::list_worklogs;

pub async fn list(from_date: &str, to_date: &str) {
    println!("Listing worklogs...");

    match list_worklogs(from_date, to_date).await {
        Ok(worklogs) => println!("Worklogs: {:?}", worklogs),
        Err(e) => eprintln!("❌ Failed to list worklogs: {}", e),
    }
}
