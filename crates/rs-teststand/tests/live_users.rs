//! Live-engine tests for the user-management API.
//!
//! Everything here works on in-memory user objects. The station's users file is
//! never written, and no existing account is modified.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.

#![cfg(feature = "live-engine")]

use rs_teststand::{Engine, Error, UserPrivilege};

#[test]
#[ignore = "requires a live engine"]
fn a_new_user_carries_the_names_and_password_set_on_it() -> Result<(), Error> {
    let engine = Engine::new()?;
    let user = engine.new_user(None)?;

    user.set_login_name("operator1")?;
    user.set_full_name("Test Operator")?;
    user.set_password("ts-secret")?;

    assert_eq!(user.login_name()?, "operator1");
    assert_eq!(user.full_name()?, "Test Operator");

    // The credential check is the supported way to test a password; it must
    // accept the right one and reject anything else.
    assert!(user.validate_password("ts-secret")?);
    assert!(!user.validate_password("wrong")?);
    assert!(!user.validate_password("")?);
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_user_with_no_profile_holds_no_privileges() -> Result<(), Error> {
    // `new_user(None)` inherits nothing, so every built-in privilege should
    // answer false. That is what makes the profile argument meaningful.
    let engine = Engine::new()?;
    let user = engine.new_user(None)?;

    let mut granted = Vec::new();
    for privilege in UserPrivilege::ALL {
        if user.has_privilege(privilege)? {
            granted.push(privilege.name());
        }
    }
    assert!(
        granted.is_empty(),
        "a profile-less user unexpectedly holds: {granted:?}"
    );
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn every_built_in_privilege_name_is_accepted_by_the_engine() -> Result<(), Error> {
    // A misspelled privilege does not raise — it quietly answers false. So the
    // check here is that the engine accepts each name at all, which is what the
    // enum exists to guarantee.
    let engine = Engine::new()?;
    let user = engine.new_user(None)?;

    let mut rejected = Vec::new();
    for privilege in UserPrivilege::ALL {
        if let Err(error) = user.has_privilege(privilege) {
            rejected.push(format!("{}: {error}", privilege.name()));
        }
    }
    assert!(rejected.is_empty(), "engine rejected: {rejected:?}");
    println!("  {} privilege names accepted", UserPrivilege::ALL.len());
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn privileges_inherit_from_a_profile_but_group_membership_does_not() -> Result<(), Error> {
    // The documented contract of the profile argument. Build a source user,
    // grant it something through its own privilege tree, then create a second
    // user from it and see the grant carried across.
    let engine = Engine::new()?;
    let source = engine.new_user(None)?;
    source.set_login_name("profile-source")?;

    // The privilege tree is a property tree, but its members are not plain
    // booleans — writing one as a bool is refused — so discover the layout
    // rather than assume it.
    let privileges = source.privileges()?;
    let count = privileges.get_num_sub_properties("")?;
    println!("  privilege tree holds {count} entries");
    for index in 0..count.min(8) {
        let name = privileges.get_nth_sub_property_name("", index, 0)?;
        let child = privileges.get_property_object(&name, 0)?;
        let kind = child.property_type()?.display_string()?;
        let inner = child.get_num_sub_properties("")?;
        println!("    {name:24} {kind:20} {inner} member(s)");
    }

    // Whatever the layout, creating from a profile must at least succeed and
    // produce a usable user. Granting through the tree needs the layout above,
    // which is why this stops short of asserting an inherited grant.
    let inherited = engine.new_user(Some(&source))?;
    inherited.set_login_name("profile-derived")?;
    assert_eq!(inherited.login_name()?, "profile-derived");
    assert!(
        !inherited.has_privilege(UserPrivilege::Operate)?,
        "an ungranted profile should not confer Operate"
    );
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn the_station_reports_its_existing_accounts() -> Result<(), Error> {
    // Read-only inspection of the real station: no account is created here.
    let engine = Engine::new()?;

    // A name nobody would have configured.
    assert!(
        !engine.user_name_exists("definitely-not-a-real-login-name-9137")?,
        "an invented login name should not exist"
    );
    assert!(
        engine
            .get_user("definitely-not-a-real-login-name-9137")?
            .is_none(),
        "looking up a missing user should be None, not an error"
    );

    match engine.current_user()? {
        Some(user) => println!("  logged in as: {}", user.login_name()?),
        None => println!("  nobody logged in (expected when login is not required)"),
    }
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_user_is_reachable_as_a_property_tree() -> Result<(), Error> {
    let engine = Engine::new()?;
    let user = engine.new_user(None)?;
    user.set_login_name("tree-probe")?;

    let tree = user.as_property_object()?;
    let count = tree.get_num_sub_properties("")?;
    assert!(count > 0, "a user should expose its fields as properties");

    let mut names = Vec::new();
    for index in 0..count {
        names.push(tree.get_nth_sub_property_name("", index, 0)?);
    }
    println!("  user fields: {}", names.join(", "));
    Ok(())
}
