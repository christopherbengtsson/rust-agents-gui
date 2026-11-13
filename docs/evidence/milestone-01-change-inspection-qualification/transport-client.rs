//! Opt-in production-client OS qualification against disposable supervisor state.
//! Not part of the unit suite. Admits one selected preview, then observes it.
use std::time::{Duration, Instant};
use workspace_core::{connection::SupervisorClient, workspaces::*};
fn main() {
    let mut args = std::env::args().skip(1);
    let client = SupervisorClient::new(args.next().unwrap().into());
    let workspace_id = WorkspaceId(args.next().unwrap().parse().unwrap());
    let WorkspaceReply::Changes { observation } = client.workspace_request(WorkspaceRequest::Changes {
        workspace_id, refresh: false, comparison_id: None, after: None,
    }).unwrap() else { panic!("wrong reply") };
    let page = observation.page.unwrap();
    let file = page.files.iter().find(|f| f.path == "limited.txt").unwrap();
    let target = PreviewTarget { workspace_id, comparison_id: page.comparison_id, file_id: file.id };
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut load = true;
    loop {
        let reply = client.workspace_request(WorkspaceRequest::ChangePreview { target: target.clone(), load }).unwrap();
        load = false;
        let WorkspaceReply::ChangePreview { observation } = reply else { panic!("wrong preview reply") };
        if observation.phase == InspectionPhase::Ready {
            let preview = observation.preview.expect("preview exists");
            println!("rows={} limited={:?}", preview.rows.len(), preview.limited);
            assert_eq!(preview.rows.len(), 200);
            assert!(preview.limited.is_some());
            break;
        }
        assert!(matches!(observation.phase, InspectionPhase::Queued | InspectionPhase::Running), "{observation:?}");
        assert!(Instant::now() < deadline, "preview did not settle");
        std::thread::sleep(Duration::from_millis(10));
    }
}
