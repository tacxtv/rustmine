use crate::launch::{LaunchMetadata, Memory};

mod launch;

#[tokio::main]
async fn main() {
    let path = std::env::current_dir().unwrap();

    launch::launch_minecraft(Some(LaunchMetadata {
        path,
        version: "1.20.1".to_owned(),
        instance_name: None,
        loader: None,
        java: None,
        screen: None,
        memory: Memory {
            min: Some("2G".to_owned()),
            max: Some("4G".to_owned()),
        },
    })).await;
}
