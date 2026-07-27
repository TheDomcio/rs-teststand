//! Live-engine tests for the `WorkspaceFile` members.
//!
//! The workspace is created in memory by the test, so nothing on the station is
//! read or modified and the tests run on any installation.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.

#![cfg(feature = "live-engine")]

use rs_teststand::{Engine, Error};

/// Creates a workspace in memory for the test to inspect.
///
/// Built here rather than opened from an installation. Reading NI's shipped
/// workspace would mean depending on files this project does not own and
/// finding them through the registry, which is reaching outside the API for
/// something the API already provides. It would also skip on any station
/// without the examples installed, which looks green while testing nothing.
fn new_workspace(engine: &Engine) -> Result<rs_teststand::WorkspaceFile, Error> {
    engine.new_workspace_file()
}

#[test]
#[ignore = "requires a live engine"]
fn workspace_file_exposes_its_tree_and_source_control_state() -> Result<(), Error> {
    let engine = Engine::new()?;
    let workspace = new_workspace(&engine)?;

    // The tree is reachable. A workspace that has never been saved reports no
    // path, which is the correct answer rather than a missing one, so what is
    // asserted here is that the root is readable at all.
    let root = workspace.root_workspace_object()?;
    assert!(
        root.path().is_ok(),
        "the root workspace object should be readable"
    );

    // ProviderName distinguishes three states, and all three are legitimate on a
    // freshly opened example: no provider, the system default, or a named one.
    match workspace.provider_name()? {
        None => eprintln!("provider: none named"),
        Some(name) if name.is_empty() => eprintln!("provider: system default"),
        Some(name) => eprintln!("provider: {name}"),
    }

    // Reading the connection state must not fail, whatever its value. The engine
    // connects a workspace when it adopts it as the current one, which opening
    // read-only does not do, so `false` here is expected rather than a defect.
    let connected = workspace.is_connected_to_sc_provider()?;
    eprintln!("connected to source control provider: {connected}");

    Ok(())
}

/// `SaveWorkspaceAndProjectFiles` is deliberately **not** exercised.
///
/// It prompts the user when there are unsaved modifications, so calling it in an
/// automated suite would either block on the prompt or write to a file that
/// ships with the installation. This test records why the gap exists, so nobody
/// later "completes coverage" by adding a call that blocks CI.
#[test]
#[ignore = "requires a live engine"]
fn saving_a_workspace_is_interactive_and_stays_untested() -> Result<(), Error> {
    let engine = Engine::new()?;
    let workspace = new_workspace(&engine)?;
    // A workspace built in memory has nothing unsaved to prompt about, but that
    // is a property of this fixture rather than of the method. Assert only that
    // the workspace is usable; do not call the interactive member.
    assert!(workspace.root_workspace_object().is_ok());
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_workspace_is_a_property_object_file_underneath() -> Result<(), Error> {
    // WorkspaceFile.AsPropertyObjectFile was unreachable until PropertyObjectFile
    // existed. It is the route to a workspace's stored data and registered
    // types, and it is the last non-interactive member of the interface.
    let engine = Engine::new()?;
    let workspace = new_workspace(&engine)?;

    let as_file = workspace.as_property_object_file()?;
    assert!(
        as_file.data().is_ok(),
        "the file's data root should be readable"
    );
    assert!(
        as_file.type_usage_list().is_ok(),
        "a workspace file carries a type usage list like any other"
    );
    Ok(())
}
