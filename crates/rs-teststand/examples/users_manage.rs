//! Example: create a user, check its credentials, and query privileges.
//!
//! ```text
//! cargo run --example users_manage
//! ```
//!
//! Everything happens in memory. The station's users file is not written and no
//! existing account is touched.
//!
//! Effective privileges normally come from the groups a user belongs to, which
//! are configured on the station — so this shows creating and interrogating a
//! user rather than granting rights.

use rs_teststand::{Engine, User, UserPrivilege};

/// Prints which of a selection of privileges a user holds.
fn report_privileges(user: &User, privileges: &[UserPrivilege]) -> Result<(), rs_teststand::Error> {
    for privilege in privileges {
        // has_privilege answers for the user *and* any group they belong to.
        println!(
            "    {:<20} {}",
            privilege.name(),
            user.has_privilege(*privilege)?
        );
    }
    Ok(())
}

fn main() -> Result<(), rs_teststand::Error> {
    let engine = Engine::new()?;

    // Passing no profile means the user inherits nothing.
    let user = engine.new_user(None)?;
    user.set_login_name("operator1")?;
    user.set_full_name("Test Operator")?;
    user.set_password("ts-secret")?;

    println!("User: {} ({})", user.login_name()?, user.full_name()?);

    // Check a credential without handling the stored value.
    println!(
        "  password 'ts-secret' valid: {}",
        user.validate_password("ts-secret")?
    );
    println!(
        "  password 'wrong' valid:     {}",
        user.validate_password("wrong")?
    );

    println!("  privilege checks:");
    report_privileges(
        &user,
        &[
            UserPrivilege::Operate,
            UserPrivilege::Execute,
            UserPrivilege::Develop,
            UserPrivilege::Debug,
            UserPrivilege::EditUsers,
        ],
    )?;

    // A privilege can also be named by its full path, which is how a leaf
    // inside a category is reached.
    println!(
        "  Debug.RunSelectedTests:  {}",
        user.has_privilege_named("Debug.RunSelectedTests")?
    );

    // The privilege tree is nested: categories hold the individual rights.
    let privileges = user.privileges()?;
    let count = privileges.get_num_sub_properties("")?;
    println!("  privilege tree ({count} categories):");
    for index in 0..count {
        let name = privileges.get_nth_sub_property_name("", index, 0)?;
        let members = privileges
            .get_property_object(&name, 0)?
            .get_num_sub_properties("")?;
        println!("    {name:<12} {members} member(s)");
    }

    // The station itself, read-only.
    println!("\nStation:");
    match engine.current_user()? {
        Some(current) => println!("  logged in as {}", current.login_name()?),
        None => println!("  nobody logged in (usual when login is not required)"),
    }
    println!(
        "  is 'operator1' a real account here? {}",
        engine.user_name_exists("operator1")?
    );

    Ok(())
}
