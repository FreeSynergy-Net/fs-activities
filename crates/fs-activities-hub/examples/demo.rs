//! History over the hub: connect, register, observe, print the timeline.
//!
//! Needs a running hub (`fs-registry`). Run with `cargo run -p fs-activities-hub --example demo`; without a
//! hub it says so and ends (programs work alone, ADR 0019).

use std::sync::Arc;

use fs_activities_core::{EntryFilter, HistoryRepo, Observer};
use fs_activities_hub::{HubSource, manifest};
use fs_objects::TursoStore;
use fs_registry_client::{HubStatus, connect, offer_install};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = match connect(manifest()).await? {
        HubStatus::Connected(s) => s,
        HubStatus::Absent => {
            println!("no hub here. {}", offer_install());
            return Ok(());
        }
    };
    let store = Arc::new(TursoStore::open_in_memory().await?);
    let repo = HistoryRepo::new(store.clone());
    let observer = Observer::new(repo.clone(), Arc::new(HubSource::new(session)));
    println!("recorded: {:?}", observer.catch_up().await?);
    for e in repo.timeline(&EntryFilter::default(), None).await?.items {
        println!("{} {} {} {:?}", e.at, e.program, e.kind, e.name);
    }
    Ok(())
}
