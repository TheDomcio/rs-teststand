//! Example: Create and manage TestStand workspace and project files programmatically via `WorkspaceFile` and `WorkspaceObject`.

use rs_teststand::Engine;

fn main() -> Result<(), rs_teststand::Error> {
    let engine = Engine::new()?;

    println!("Creating a new workspace file...");
    let ws_file = engine.new_workspace_file()?;
    let root_obj = ws_file.root_workspace_object()?;

    println!("Root workspace object:");
    println!("  Display Name: '{}'", root_obj.display_name()?);
    println!("  Object Type: {}", root_obj.object_type()?);
    println!("  Child Objects: {}", root_obj.num_contained_objects()?);

    println!("\nCreating project and folder structure...");
    let project = root_obj.new_folder("Project Alpha")?;
    println!("  Created project folder: '{}'", project.display_name()?);

    let seq_entry = project.new_file("MainTest.seq")?;
    println!(
        "  Created sequence file entry: '{}'",
        seq_entry.display_name()?
    );

    println!(
        "\nUpdated root child objects count: {}",
        root_obj.num_contained_objects()?
    );

    Ok(())
}
