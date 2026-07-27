//! The file that holds the station's users and groups.

use rs_teststand_sys::Dispatch;

use crate::Error;
use crate::dispids::users_file;
use crate::property::{PropertyObject, PropertyObjectFile};

/// The station's user list, as a file (`UsersFile`).
///
/// Reached from [`Engine::users_file`](crate::Engine::users_file). Everything a
/// station knows about who may log in and what they may do lives here, and this
/// is the only route to **persisting** it: [`User`](crate::User) objects created
/// through [`Engine::new_user`](crate::Engine::new_user) exist in memory until
/// this file is saved.
///
/// The lists are [`PropertyObject`] arrays rather than typed collections,
/// because that is what the engine returns and how it expects them to be
/// edited. Add or remove an element with the property-object API and the change
/// lands in the list itself; see [`user_list`](Self::user_list).
///
/// # Saving
///
/// Go through [`as_property_object_file`](Self::as_property_object_file) and
/// call [`PropertyObjectFile::save_file_if_modified`], passing `false` so the
/// engine writes the file instead of asking a person first.
#[derive(Debug)]
pub struct UsersFile {
    dispatch: Box<dyn Dispatch>,
}

impl UsersFile {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The users, as an array of `User` objects (`UsersFile.UserList`).
    ///
    /// Editing happens through this array, not through a setter: use the
    /// property-object API to append or remove an element and the station's
    /// user list changes with it.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn user_list(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch.get(users_file::USER_LIST)?.into_object()?,
        ))
    }

    /// The user groups (`UsersFile.UserGroupList`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn user_group_list(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch
                .get(users_file::USER_GROUP_LIST)?
                .into_object()?,
        ))
    }

    /// The user profiles (`UsersFile.UserProfileList`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn user_profile_list(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch
                .get(users_file::USER_PROFILE_LIST)?
                .into_object()?,
        ))
    }

    /// Re-reads the file from disk (`UsersFile.ReloadFromDisk`).
    ///
    /// **Everything read from this file beforehand is stale afterwards.** The
    /// engine documents that references to the user list and to individual
    /// users are out of date once this returns, so drop what you are holding
    /// and read it again rather than reusing it. Consuming `&self` cannot
    /// express that, since the file object itself stays valid; only what came
    /// out of it does not.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn reload_from_disk(&self) -> Result<(), Error> {
        self.dispatch.call(users_file::RELOAD_FROM_DISK, &[])?;
        Ok(())
    }

    /// The file view of this object (`UsersFile.AsPropertyObjectFile`).
    ///
    /// Where the path lives, and where saving happens.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object_file(&self) -> Result<PropertyObjectFile, Error> {
        Ok(PropertyObjectFile::new(
            self.dispatch
                .call(users_file::AS_PROPERTY_OBJECT_FILE, &[])?
                .into_object()?,
        ))
    }
}
