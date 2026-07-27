//! A station user.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::user;
use crate::property::PropertyObject;
use crate::users::UserPrivilege;

/// A user account (`User`).
///
/// Obtained from [`Engine::new_user`](crate::Engine::new_user),
/// [`Engine::get_user`](crate::Engine::get_user), or
/// [`Engine::current_user`](crate::Engine::current_user).
///
/// A user object built in memory is not part of the station until it is written
/// to the users file; creating and configuring one here changes nothing on disk.
///
/// Effective privileges usually come from the groups a user belongs to rather
/// than from the user directly, which is why
/// [`has_privilege`](Self::has_privilege) answers for the user *and* their
/// groups, while [`privileges`](Self::privileges) exposes only what is set on
/// the user itself.
#[derive(Debug)]
pub struct User {
    dispatch: Box<dyn Dispatch>,
}

impl User {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The login name (`User.LoginName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn login_name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(user::LOGIN_NAME)?.into_string()?)
    }

    /// Sets the login name (`User.LoginName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_login_name(&self, name: &str) -> Result<(), Error> {
        self.dispatch
            .put(user::LOGIN_NAME, Value::Str(name.to_owned()))?;
        Ok(())
    }

    /// The display name (`User.FullName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn full_name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(user::FULL_NAME)?.into_string()?)
    }

    /// Sets the display name (`User.FullName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_full_name(&self, name: &str) -> Result<(), Error> {
        self.dispatch
            .put(user::FULL_NAME, Value::Str(name.to_owned()))?;
        Ok(())
    }

    /// Sets the password (`User.Password`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_password(&self, password: &str) -> Result<(), Error> {
        self.dispatch
            .put(user::PASSWORD, Value::Str(password.to_owned()))?;
        Ok(())
    }

    /// Reads the stored password field (`User.Password`).
    ///
    /// Prefer [`validate_password`](Self::validate_password) for checking a
    /// credential: it compares without the caller handling the stored value.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn password(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(user::PASSWORD)?.into_string()?)
    }

    /// Whether `password` matches this user's (`User.ValidatePassword`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn validate_password(&self, password: &str) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(user::VALIDATE_PASSWORD, &[Value::Str(password.to_owned())])?
            .as_bool()?)
    }

    /// Whether the user, or any group they belong to, holds a privilege
    /// (`User.HasPrivilege`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn has_privilege(&self, privilege: UserPrivilege) -> Result<bool, Error> {
        self.has_privilege_named(privilege.name())
    }

    /// Whether the user holds a privilege named by string
    /// (`User.HasPrivilege`).
    ///
    /// Takes either a base name or a full path such as
    /// `Debug.RunSelectedTests`. Prefer
    /// [`has_privilege`](Self::has_privilege) for the built-in set; this exists
    /// for custom privileges, which no enum can enumerate.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn has_privilege_named(&self, privilege: &str) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(user::HAS_PRIVILEGE, &[Value::Str(privilege.to_owned())])?
            .as_bool()?)
    }

    /// The privilege settings held on the user itself (`User.Privileges`).
    ///
    /// This is not the answer to "can this user do X" — group membership is not
    /// reflected here. Use [`has_privilege`](Self::has_privilege) for that.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn privileges(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch.get(user::PRIVILEGES)?.into_object()?,
        ))
    }

    /// The member list of a user *group* (`User.Members`).
    ///
    /// Only meaningful when this object represents a group rather than a person.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn members(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch.get(user::MEMBERS)?.into_object()?,
        ))
    }

    /// The user as a plain property tree (`User.AsPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch
                .call(user::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// An owned handle to the same user, for passing it back to the engine.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }
}
