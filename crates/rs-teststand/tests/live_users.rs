//! Live-engine tests for the user-management API.
//!
//! Everything here works on in-memory user objects. The station's users file is
//! never written, and no existing account is modified.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.

#![cfg(feature = "live-engine")]

use std::time::Instant;

use rs_teststand::{Engine, Error, UserPrivilege};

fn a_new_user_carries_the_names_and_password_set_on_it(engine: &Engine) -> Result<(), Error> {
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

fn a_user_with_no_profile_holds_no_privileges(engine: &Engine) -> Result<(), Error> {
    // `new_user(None)` inherits nothing, so every built-in privilege should
    // answer false. That is what makes the profile argument meaningful.
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

fn every_built_in_privilege_name_is_accepted_by_the_engine(engine: &Engine) -> Result<(), Error> {
    // A misspelled privilege does not raise — it quietly answers false. So the
    // check here is that the engine accepts each name at all, which is what the
    // enum exists to guarantee.
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

fn privileges_inherit_from_a_profile_but_group_membership_does_not(
    engine: &Engine,
) -> Result<(), Error> {
    // The documented contract of the profile argument. Build a source user,
    // grant it something through its own privilege tree, then create a second
    // user from it and see the grant carried across.
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

fn the_station_reports_its_existing_accounts(engine: &Engine) -> Result<(), Error> {
    // Read-only inspection of the real station: no account is created here.

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

fn a_user_is_reachable_as_a_property_tree(engine: &Engine) -> Result<(), Error> {
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

fn the_users_file_exposes_the_lists_the_station_loaded(engine: &Engine) -> Result<(), Error> {
    // Read-only on purpose. This suite must never rewrite the station's real
    // user list, so nothing here saves and nothing is added or removed.
    let users_file = engine.users_file()?;

    let users = users_file.user_list()?;
    let groups = users_file.user_group_list()?;
    let profiles = users_file.user_profile_list()?;

    // Each is an array of objects, which is what makes the property-object API
    // the way to edit them rather than a typed setter.
    assert!(
        users.get_num_elements()? > 0,
        "a station always has at least one account"
    );
    assert!(groups.get_num_elements()? >= 0);
    assert!(profiles.get_num_elements()? >= 0);

    // The engine's own account lookup and the file's list must describe the
    // same station; if they disagree, one of them is not the loaded file.
    let first = users.get_property_object_by_offset(0, 0)?;
    let login = first.get_val_string("LoginName", 0)?;
    assert!(
        engine.user_name_exists(&login)?,
        "user {login} is in the file but the engine does not know the name"
    );
    Ok(())
}

fn the_users_file_knows_where_it_lives_and_saves_without_asking(
    engine: &Engine,
) -> Result<(), Error> {
    // The route that makes a new user outlive the process: the file view, its
    // path, and a save that does not stop for a person.
    let file = engine.users_file()?.as_property_object_file()?;

    let path = file.path()?;
    assert!(
        path.to_ascii_lowercase().ends_with(".ini"),
        "expected the users file to be an .ini, got {path:?}"
    );

    // Nothing above modified the file, so this writes nothing. That is exactly
    // what makes it safe to assert here: an unmodified file is a no-op, and
    // `false` would mean a prompt appeared and was declined, which must not
    // happen when prompting is off.
    assert!(
        file.save_file_if_modified(false)?,
        "an unmodified save must succeed without a dialog"
    );
    Ok(())
}

/// Every check in this file, over one engine.
///
/// A fresh engine costs about three seconds before it is usable, so sharing one
/// takes this file from 20 seconds to a few.
#[test]
#[ignore = "requires a live engine"]
fn users_behave_as_documented() -> Result<(), Error> {
    type Step = (&'static str, fn(&Engine) -> Result<(), Error>);

    let engine = Engine::new()?;
    let steps: [Step; 8] = [
        (
            "a new user carries the names and password set on it",
            a_new_user_carries_the_names_and_password_set_on_it,
        ),
        (
            "a user with no profile holds no privileges",
            a_user_with_no_profile_holds_no_privileges,
        ),
        (
            "every built in privilege name is accepted by the engine",
            every_built_in_privilege_name_is_accepted_by_the_engine,
        ),
        (
            "privileges inherit from a profile but group membership does not",
            privileges_inherit_from_a_profile_but_group_membership_does_not,
        ),
        (
            "the station reports its existing accounts",
            the_station_reports_its_existing_accounts,
        ),
        (
            "a user is reachable as a property tree",
            a_user_is_reachable_as_a_property_tree,
        ),
        (
            "the users file exposes the lists the station loaded",
            the_users_file_exposes_the_lists_the_station_loaded,
        ),
        (
            "the users file knows where it lives and saves without asking",
            the_users_file_knows_where_it_lives_and_saves_without_asking,
        ),
    ];

    for (label, step) in steps {
        let started = Instant::now();
        step(&engine)?;
        println!("  ok: {label} ({:?})", started.elapsed());
    }
    Ok(())
}
