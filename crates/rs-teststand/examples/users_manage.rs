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
//! are configured on the station, so this shows creating and interrogating a
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

    // Where accounts actually live. Everything above this point exists only in
    // memory: `new_user` builds a User, it does not enrol one. A user becomes
    // part of the station when it is inserted into this file's list and the
    // file is written.
    let users_file = engine.users_file()?;
    let file = users_file.as_property_object_file()?;
    println!(
        "
Users file:"
    );
    println!("  path            {}", file.path()?);
    println!(
        "  users           {}",
        users_file.user_list()?.get_num_elements()?
    );
    println!(
        "  groups          {}",
        users_file.user_group_list()?.get_num_elements()?
    );
    println!(
        "  profiles        {}",
        users_file.user_profile_list()?.get_num_elements()?
    );

    // Deliberately not run: it would edit the station this example is reading.
    // The two calls a host makes to enrol the user built above are
    //
    //     users_file.user_list()?.set_num_elements(count + 1, 0)?;   // then fill
    //     file.save_file_if_modified(false)?;                        // then write
    //
    // `false` matters: with `true` the engine puts a save dialog on screen, and
    // a host with no operator would wait for an answer that never comes.
    println!(
        "
  (this example does not write; enrolling a user would alter the station)"
    );

    Ok(())
}
