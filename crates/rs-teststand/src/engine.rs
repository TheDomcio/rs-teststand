//! The `Engine`, root of the object model and factory for everything else.

use rs_teststand_sys::{Dispatch, Value, create_dispatch};

#[path = "engine_startup.rs"]
mod startup;

use rs_teststand_sys::DialogInfo;

use crate::dispids::engine as dispid;
use crate::error::Error;

/// ProgID of the version-independent engine coclass. Resolves to the active
/// installation's `teapi.dll` via the registry.
const ENGINE_PROG_ID: &str = "TestStand.Engine";

/// The TestStand™ Engine.
///
/// Constructing an `Engine` creates the underlying COM object; dropping it
/// releases it. It is the entry point of the API:
///
/// ```no_run
/// use rs_teststand::Engine;
///
/// let engine = Engine::new()?;
/// println!("TestStand {}", engine.version_string()?);
/// # Ok::<(), rs_teststand::Error>(())
/// ```
#[derive(Debug)]
pub struct Engine {
    dispatch: Box<dyn Dispatch>,
    /// Dialogs closed while this engine was being created. See
    /// [`startup_dialogs`](Engine::startup_dialogs).
    startup_dialogs: Vec<DialogInfo>,
}

impl Engine {
    /// Creates the engine (STA COM apartment plus the `TestStand.Engine` object).
    ///
    /// # Errors
    /// [`Error::Com`] if COM cannot be initialized or the engine class
    /// cannot be created (e.g. no TestStand™ installation is registered).
    pub fn new() -> Result<Self, Error> {
        // Creating the engine can itself raise a dialog, before any option can
        // be set, see the `startup` module. The sweeper has to be running
        // before the call, because the call is what blocks.
        let sweeper = startup::Sweeper::start();
        let dispatch = create_dispatch(ENGINE_PROG_ID);
        let startup_dialogs = sweeper.stop();
        let engine = Self {
            dispatch: Box::new(dispatch?),
            startup_dialogs,
        };
        engine.suppress_modal_dialogs();
        engine.load_type_palettes();
        Ok(engine)
    }

    /// Dialogs that were closed while this engine was being created.
    ///
    /// A non-empty list is worth logging: it is the only record that something
    /// asked a question and was answered by closing the window.
    ///
    /// Empty means nothing *owned by this process* was found, which is not the
    /// same as no dialog having appeared. Detection cannot see another
    /// process's windows, and whether the engine's unreleased-files warning is
    /// raised in-process has not been established. Do not treat an empty list
    /// as proof that startup was clean.
    #[must_use]
    pub fn startup_dialogs(&self) -> &[DialogInfo] {
        &self.startup_dialogs
    }

    /// Loads the station's type palettes.
    ///
    /// Step types live in the palettes, so without this
    /// [`new_step`](Self::new_step) fails with `TS_Err_StepTypeNotFound` for
    /// every built-in type. The sequence editor does this as it starts; an
    /// engine created directly over COM does not, so the crate does it here.
    ///
    /// Conflicts are resolved by failing rather than prompting, and a failure
    /// is ignored for the same reason the dialog settings are: a station with
    /// no palettes configured is still a usable engine.
    fn load_type_palettes(&self) {
        let _ = self.load_type_palette_files_ex(crate::ConflictHandler::Error, 0);
    }

    /// Loads the type palette files (`Engine.LoadTypePaletteFilesEx`).
    ///
    /// Called during construction; exposed for a caller that reconfigures the
    /// palette list and needs to reload.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn load_type_palette_files_ex(
        &self,
        handler: crate::ConflictHandler,
        options: i32,
    ) -> Result<(), Error> {
        self.dispatch.call(
            dispid::LOAD_TYPE_PALETTE_FILES_EX,
            &[Value::I32(handler.bits()), Value::I32(options)],
        )?;
        Ok(())
    }

    /// Loads the type palette files (`Engine.LoadTypePaletteFiles`).
    ///
    /// The older form, without conflict handling. Kept because it is the member
    /// available on engines from TestStand 2016.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn load_type_palette_files(&self) -> Result<(), Error> {
        self.dispatch.call(dispid::LOAD_TYPE_PALETTE_FILES, &[])?;
        Ok(())
    }

    /// Unloads the type palette files (`Engine.UnloadTypePaletteFiles`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn unload_type_palette_files(&self) -> Result<(), Error> {
        self.dispatch.call(dispid::UNLOAD_TYPE_PALETTE_FILES, &[])?;
        Ok(())
    }

    /// Points the station's dialog-raising options at their non-interactive
    /// settings for this session.
    ///
    /// A modal dialog is fatal to an unattended host: no one is there to
    /// dismiss it, so the call blocks forever. That is unacceptable for CI,
    /// provisioning, or a long-lived service, so engine construction always
    /// applies these.
    ///
    /// Failures are deliberately ignored: an engine that cannot be configured
    /// is still usable, and construction must not fail over a hardening step.
    ///
    /// One gap is worth knowing about. Automatic login uses the operating
    /// system identity and skips password authentication, but only if that
    /// account is also a known engine user; if it is not, the engine falls back
    /// to asking, which is the one dialog this method cannot rule out. A
    /// station that runs headless should therefore have its service account
    /// present in the user file. Guard the first calls with a
    /// [`Watchdog`](crate::Watchdog) if that cannot be guaranteed.
    ///
    /// Tracing is left alone. Only the execution bits that halt and wait for a
    /// person are cleared, so a host keeps whatever tracing the station was
    /// configured for, see [`ExecutionMask`](crate::ExecutionMask).
    fn suppress_modal_dialogs(&self) {
        let Ok(options) = self.station_options() else {
            return;
        };

        // A run-time error must resolve itself rather than prompt.
        let _ = options.set_rte_option(crate::RunTimeErrorOption::Abort);

        // Every prompt the engine can raise while loading or editing files.
        let _ = options.set_prompt_to_find_files(false);
        let _ = options.set_type_version_auto_increment_prompt_opt(false);
        let _ = options.set_use_dialog_for_check_out(false);
        let _ = options.set_prompt_when_adding_files_to_sc(false);
        let _ = options.set_check_out_files_when_edited(false);

        // Logging in must never stop on a dialog. Privilege checking off and
        // login not required means no gate; auto-login uses the operating
        // system identity, which the engine accepts without asking for a
        // password. The residual risk is documented on this method.
        let _ = options.set_enable_user_privilege_checking(false);
        let _ = options.set_require_user_login(false);
        let _ = options.set_auto_login_system_user(true);

        // Debug features are a debugging aid with a cost, and two of them raise
        // a dialog at shutdown. Nothing here is useful to an unattended host.
        let _ = options.set_debug_options(crate::DebugOptions::NONE);

        // Read-modify-write: keep the station's tracing choices, drop only the
        // break bits, which suspend an execution until an operator acts.
        if let Ok(current) = options.execution_mask() {
            let running = current.difference(crate::ExecutionMask::BREAKS);
            let _ = options.set_execution_mask(running);
        }
    }

    /// The engine's major version number (`Engine.MajorVersion`): the two-digit
    /// major, so TestStand™ 2026 reports `26` and 2016 reports `16`.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn major_version(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(dispid::MAJOR_VERSION)?.as_i32()?)
    }

    /// The engine's minor version number (`Engine.MinorVersion`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn minor_version(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(dispid::MINOR_VERSION)?.as_i32()?)
    }

    /// The engine's revision version number (`Engine.RevisionVersion`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn revision_version(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(dispid::REVISION_VERSION)?.as_i32()?)
    }

    /// The engine's build version number (`Engine.BuildVersion`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn build_version(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(dispid::BUILD_VERSION)?.as_i32()?)
    }

    /// The engine's full version string (`Engine.VersionString`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn version_string(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(dispid::VERSION_STRING)?.into_string()?)
    }

    /// Returns `true` if the TestStand™ engine is running as a 64-bit process (`Engine.Is64Bit`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_64bit(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(dispid::IS_64BIT)?.as_bool()?)
    }

    /// The path to the TestStand™ root directory (`Engine.TestStandDirectory`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn teststand_directory(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(dispid::TESTSTAND_DIRECTORY)?
            .into_string()?)
    }

    /// The path to the TestStand™ `Bin` directory (`Engine.BinDirectory`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn bin_directory(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(dispid::BIN_DIRECTORY)?.into_string()?)
    }

    /// The path to the TestStand™ `Cfg` directory (`Engine.ConfigDirectory`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn config_directory(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(dispid::CONFIG_DIRECTORY)?.into_string()?)
    }

    /// Accesses the station's configuration settings (`Engine.StationOptions`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn station_options(&self) -> Result<crate::station::StationOptions, Error> {
        let dispatch = self.dispatch.get(dispid::STATION_OPTIONS)?.into_object()?;
        Ok(crate::station::StationOptions::new(dispatch))
    }

    /// Creates an empty sequence file (`Engine.NewSequenceFile`).
    ///
    /// The file exists only in memory until it is saved.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn new_sequence_file(&self) -> Result<crate::SequenceFile, Error> {
        Ok(crate::SequenceFile::new(
            self.dispatch
                .call(dispid::NEW_SEQUENCE_FILE, &[])?
                .into_object()?,
        ))
    }

    /// Starts a sequence running (`Engine.NewExecution`).
    ///
    /// The execution begins immediately.
    ///
    /// Pass `None` for `process_model` to run the sequence directly; supply one
    /// to run a process-model entry point instead. `execution_type_mask` is
    /// normally `0`.
    ///
    /// # Errors
    /// [`Error`] if the sequence cannot be started or the COM call fails.
    pub fn new_execution(
        &self,
        sequence_file: &crate::SequenceFile,
        sequence_name: &str,
        process_model: Option<&crate::SequenceFile>,
        break_at_first_step: bool,
        execution_type_mask: i32,
    ) -> Result<crate::Execution, Error> {
        let file = sequence_file
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live sequence file",
                actual: "a test fake with no COM identity",
            })?;
        // "No process model" is a null object reference, not a null variant.
        let model = process_model
            .and_then(crate::SequenceFile::duplicate_dispatch)
            .map_or(Value::NullObject, Value::Object);

        Ok(crate::Execution::new(
            self.dispatch
                .call(
                    dispid::NEW_EXECUTION,
                    &[
                        Value::Object(file),
                        Value::Str(sequence_name.to_owned()),
                        model,
                        Value::Bool(break_at_first_step),
                        Value::I32(execution_type_mask),
                    ],
                )?
                .into_object()?,
        ))
    }

    /// Posts a message on behalf of an execution (`Engine.PostUIMessage`).
    ///
    /// The counterpart to
    /// [`Thread::post_ui_message_ex`](crate::Thread::post_ui_message_ex), for
    /// code that is not itself running inside the sequence and therefore has no
    /// current thread to post from. Because there is no implied context, the
    /// execution and thread the message belongs to are given explicitly.
    ///
    /// `activex_data` carries structured data, read back by the host from
    /// [`UIMessage::activex_data`](crate::UIMessage::activex_data). Pass `None`
    /// to leave the slot empty.
    ///
    /// Pass `synchronous = true` in the ordinary case; see
    /// [`Thread::post_ui_message_ex`](crate::Thread::post_ui_message_ex) for why
    /// the blocking form is the safe default.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails, or if a wrapper has no COM identity.
    #[allow(
        clippy::too_many_arguments,
        reason = "mirrors Engine.PostUIMessage's parameter list and order, which                   is the point of a twin API: grouping them into a struct would                   make the Rust call unpredictable from the COM documentation"
    )]
    pub fn post_ui_message(
        &self,
        execution: &crate::Execution,
        thread: &crate::Thread,
        event_code: i32,
        numeric_data: f64,
        string_data: &str,
        activex_data: Option<&crate::PropertyObject>,
        synchronous: bool,
    ) -> Result<(), Error> {
        let missing = || Error::UnexpectedType {
            expected: "a live execution and thread",
            actual: "a test fake with no COM identity",
        };
        let execution_handle = execution.duplicate_dispatch().ok_or_else(missing)?;
        let thread_handle = thread.duplicate_dispatch().ok_or_else(missing)?;
        self.dispatch.call(
            dispid::POST_UI_MESSAGE,
            &[
                Value::Object(execution_handle),
                Value::Object(thread_handle),
                Value::I32(event_code),
                Value::F64(numeric_data),
                Value::Str(string_data.to_owned()),
                crate::execution::thread::object_argument(activex_data)?,
                Value::Bool(synchronous),
            ],
        )?;
        Ok(())
    }

    /// Logs a user in, or logs the current one out (`Engine.CurrentUser`).
    ///
    /// `Some(user)` makes that user current; `None` clears it, which the engine
    /// documents as logging out.
    ///
    /// **This does not check the password.** Setting the property is the act of
    /// logging in, not an authentication step: a host that cares must call
    /// [`User::validate_password`](crate::User::validate_password) first and
    /// refuse on `false`. Written this way because the engine draws the same
    /// line, and hiding a check inside a setter would make it unclear which one
    /// a caller had actually performed.
    ///
    /// A host built on the `ActiveX` UI controls should use their own login
    /// method instead, so the controls raise the event they expect; this is the
    /// headless path.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails, or if `user` has no COM identity.
    pub fn set_current_user(&self, user: Option<&crate::users::User>) -> Result<(), Error> {
        let value = match user {
            None => Value::NullObject,
            Some(user) => {
                user.duplicate_dispatch()
                    .map(Value::Object)
                    .ok_or(Error::UnexpectedType {
                        expected: "a live user",
                        actual: "a test fake with no COM identity",
                    })?
            }
        };
        self.dispatch.put(dispid::CURRENT_USER, value)?;
        Ok(())
    }

    /// Asks every execution to stop (`Engine.TerminateAll`).
    ///
    /// Termination, not abort: cleanup groups still run, so hardware is left in
    /// a safe state. Like [`Execution::terminate`](crate::Execution::terminate)
    /// it is a request, and returns before the runs have finished unwinding. A
    /// caller that needs them stopped must then wait for
    /// [`UIMessageCode::EndExecution`](crate::UIMessageCode::EndExecution).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn terminate_all(&self) -> Result<(), Error> {
        self.dispatch.call(dispid::TERMINATE_ALL, &[])?;
        Ok(())
    }

    /// Stops every execution without running cleanup (`Engine.AbortAll`).
    ///
    /// The blunt counterpart to [`terminate_all`](Self::terminate_all). Cleanup
    /// groups do **not** run, so anything a sequence would have switched off
    /// stays on. Prefer terminating unless the point is to stop now.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn abort_all(&self) -> Result<(), Error> {
        self.dispatch.call(dispid::ABORT_ALL, &[])?;
        Ok(())
    }

    /// The license the engine is currently using (`Engine.LicenseType`).
    ///
    /// **Using, not holding.** A freshly created engine has acquired nothing
    /// and reports [`LicenseType::NoLicense`](crate::LicenseType::NoLicense)
    /// even on a fully licensed station; the answer only becomes meaningful
    /// after something acquires. Use
    /// [`require_license`](Self::require_license) to ask whether the station
    /// can license this host.
    ///
    /// Reads state, so it acquires nothing and raises no dialog.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails, or [`Error::UnknownLicenseType`] if the
    /// engine reports a type this build does not name.
    pub fn license_type(&self) -> Result<crate::LicenseType, Error> {
        let raw = self.dispatch.get(dispid::LICENSE_TYPE)?.as_i32()?;
        crate::LicenseType::from_bits(raw).map_err(|bits| Error::UnknownLicenseType { bits })
    }

    /// Acquires a license, or fails if the station cannot grant one.
    ///
    /// The check a headless host should make before anything else, and the
    /// object it should keep alive while it runs.
    ///
    /// Acquiring is what makes a license real.
    /// [`license_type`](Self::license_type) reports the license the engine is
    /// *using*, and a freshly created engine is using none, measured on a
    /// station with a valid development system license, it reads `NoLicense`
    /// until something acquires. So reading before acquiring answers the wrong
    /// question, and this method acquires first.
    ///
    /// The request is [`ApplicationLicense::Unspecified`](crate::ApplicationLicense), which lets the engine
    /// grant whatever it has. Naming a kind can be refused even when the
    /// station is properly licensed: on a development system station,
    /// [`ApplicationLicense::OperatorInterface`](crate::ApplicationLicense) is turned down while
    /// unspecified succeeds. Ask for a specific kind through
    /// [`acquire_license`](Self::acquire_license) only when the host genuinely
    /// requires that one.
    ///
    /// The startup dialog is suppressed, so an unlicensed station returns an
    /// error rather than opening a window nobody will close.
    ///
    /// **Refusal is retried for a few seconds before it is believed.** The
    /// licensing subsystem is not ready the instant the engine object exists:
    /// measured on a properly licensed station, acquiring immediately after
    /// construction is refused, while the same call half a second later
    /// succeeds. A host that trusted the first answer would report an
    /// unlicensed station to its operator and stop. So a refusal is retried
    /// until it stops changing, which costs an unlicensed station a few seconds
    /// once, at startup.
    ///
    /// Success is the handle, not the type.
    /// [`HeldLicense::kind`](crate::HeldLicense::kind) reports what the engine
    /// says it is using and can still read
    /// [`NoLicense`](crate::LicenseType::NoLicense) after an unspecified
    /// request was granted, so treat it as information rather than as the
    /// verdict.
    ///
    /// # Errors
    /// [`Error::NoLicense`] if no license can be acquired, or [`Error`] if the
    /// COM call fails.
    pub fn require_license(&self) -> Result<crate::HeldLicense<'_>, Error> {
        /// Longest to keep asking before calling the station unlicensed.
        const PATIENCE: core::time::Duration = core::time::Duration::from_secs(3);
        /// Gap between attempts.
        const RETRY_INTERVAL: core::time::Duration = core::time::Duration::from_millis(100);

        let started = std::time::Instant::now();
        let handle = loop {
            match self.acquire_license(
                crate::ApplicationLicense::Unspecified,
                crate::AcquireLicenseOptions::SUPPRESS_STARTUP_DIALOG,
            ) {
                Ok(handle) => break handle,
                Err(Error::NoLicense) if started.elapsed() < PATIENCE => {
                    std::thread::sleep(RETRY_INTERVAL);
                }
                Err(other) => return Err(other),
            }
        };
        // The grant is the handle. `LicenseType` is informational and does not
        // always follow an unspecified request: measured on a licensed station,
        // acquiring unspecified returns a handle while the type still reads
        // `NoLicense`, and only a named request such as a sequence editor makes
        // it report `DevelopmentSystem`. So the type is recorded, not gated on.
        let kind = self.license_type()?;
        Ok(crate::HeldLicense::new(self, handle, kind))
    }

    /// A description of the current license (`Engine.GetLicenseDescription`).
    ///
    /// Free text meant for a person, so log it rather than branch on it; use
    /// [`license_type`](Self::license_type) for decisions.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_license_description(&self) -> Result<String, Error> {
        // The engine declares one reserved parameter, documented as always
        // zero.
        Ok(self
            .dispatch
            .call(dispid::GET_LICENSE_DESCRIPTION, &[Value::I32(0)])?
            .into_string()?)
    }

    /// The license this application requested (`Engine.ApplicationLicense`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails, or [`Error::UnknownLicenseType`] if the
    /// engine reports a value this build does not name.
    pub fn application_license(&self) -> Result<crate::ApplicationLicense, Error> {
        let raw = self.dispatch.get(dispid::APPLICATION_LICENSE)?.as_i32()?;
        crate::ApplicationLicense::from_bits(raw).map_err(|bits| Error::UnknownLicenseType { bits })
    }

    /// Acquires a license and returns its handle (`Engine.AcquireLicense`).
    ///
    /// Release it with [`release_license`](Self::release_license); the license
    /// is held until every handle for it is released.
    ///
    /// **Pass [`AcquireLicenseOptions::SUPPRESS_STARTUP_DIALOG`](crate::AcquireLicenseOptions) on any station
    /// without a person at it.** Without it, an engine that cannot acquire the
    /// license opens a window offering to evaluate, activate or buy, and waits.
    /// A headless host stops there until something kills it. With it, the same
    /// situation returns an error this method propagates.
    ///
    /// Prefer
    /// [`ApplicationLicense::Unspecified`](crate::ApplicationLicense),
    /// which lets the engine grant whatever it has. Naming a kind is a
    /// constraint, not a preference, and a smaller request is not a safer one:
    /// on a station licensed for a development system,
    /// [`OperatorInterface`](crate::ApplicationLicense::OperatorInterface) is
    /// refused while unspecified succeeds. Name a kind only when the host truly
    /// requires it.
    ///
    /// Most callers want [`require_license`](Self::require_license) instead,
    /// which acquires and hands back a guard that releases on drop.
    ///
    /// # Errors
    /// [`Error::NoLicense`] if the license was not granted, or [`Error`] if the
    /// COM call fails.
    ///
    /// A handle of zero is treated as refusal. The reference says this member
    /// returns an error when it cannot acquire the license; measured against an
    /// unlicensed station it succeeds and hands back zero instead. A caller
    /// that trusted the documented behavior would carry on unlicensed, so the
    /// zero is turned into the error the caller was promised.
    pub fn acquire_license(
        &self,
        license: crate::ApplicationLicense,
        options: crate::AcquireLicenseOptions,
    ) -> Result<i32, Error> {
        let handle = self
            .dispatch
            .call(
                dispid::ACQUIRE_LICENSE,
                &[Value::I32(license.bits()), Value::I32(options.bits())],
            )?
            .as_i32()?;
        if handle == 0 {
            return Err(Error::NoLicense);
        }
        Ok(handle)
    }

    /// Releases a license handle (`Engine.ReleaseLicense`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn release_license(&self, handle: i32) -> Result<(), Error> {
        // Second parameter is reserved and documented as zero.
        self.dispatch.call(
            dispid::RELEASE_LICENSE,
            &[Value::I32(handle), Value::I32(0)],
        )?;
        Ok(())
    }

    /// Whether the station licenses an add-on feature
    /// (`Engine.HasAddonLicense`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn has_addon_license(&self, feature_name: &str) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                dispid::HAS_ADDON_LICENSE,
                &[Value::Str(feature_name.to_owned())],
            )?
            .as_bool()?)
    }

    /// Releases every code module the engine has loaded
    /// (`Engine.UnloadAllModules`).
    ///
    /// Loading a sequence file loads its modules, and they stay loaded until
    /// that file is closed. That is what makes the second run fast, and also
    /// what holds a DLL open against the build that wants to replace it.
    /// Unloading here frees them all at once, without closing anything.
    ///
    /// Call it between runs, not during one: a module in use by a live
    /// execution is not a candidate, and the next run reloads whatever it needs.
    ///
    /// **State inside a module does not survive.** Anything a module kept in a
    /// static or a global is gone once it is unloaded, and the reload starts
    /// from nothing. A station whose modules carry state between steps that way
    /// should keep that state in the engine instead, or not call this.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn unload_all_modules(&self) -> Result<(), Error> {
        self.dispatch.call(dispid::UNLOAD_ALL_MODULES, &[])?;
        Ok(())
    }

    /// Whether breakpoints stop an execution (`Engine.BreakpointsEnabled`).
    ///
    /// The master switch. With it off, breakpoints stay set but nothing stops
    /// on them, which is how a station runs unattended without anyone having to
    /// strip a sequence file of the breakpoints someone left in it.
    ///
    /// Distinct from the station option of the same name, which is the setting
    /// written to disk. This is the engine's live state.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn breakpoints_enabled(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(dispid::BREAKPOINTS_ENABLED)?.as_bool()?)
    }

    /// Turns breakpoints on or off (`Engine.BreakpointsEnabled`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_breakpoints_enabled(&self, enabled: bool) -> Result<(), Error> {
        self.dispatch
            .put(dispid::BREAKPOINTS_ENABLED, Value::Bool(enabled))?;
        Ok(())
    }

    /// Whether breakpoints survive the file they are set in
    /// (`Engine.PersistBreakpoints`).
    ///
    /// On, the engine remembers them across a close and reopen. A host that
    /// sets breakpoints on behalf of a remote panel usually wants this off, so
    /// that a debugging session leaves nothing behind on the station.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn persist_breakpoints(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(dispid::PERSIST_BREAKPOINTS)?.as_bool()?)
    }

    /// Chooses whether breakpoints are remembered (`Engine.PersistBreakpoints`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_persist_breakpoints(&self, persist: bool) -> Result<(), Error> {
        self.dispatch
            .put(dispid::PERSIST_BREAKPOINTS, Value::Bool(persist))?;
        Ok(())
    }

    /// Runs a .NET garbage collection now
    /// (`Engine.DoDotNetGarbageCollection`).
    ///
    /// Only relevant to a station whose steps call .NET code. Collection is
    /// otherwise periodic, on the
    /// [interval](Self::dot_net_garbage_collection_interval); this forces one,
    /// which is worth doing between runs on a long-lived host rather than
    /// during a measurement, since collection pauses the runtime.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn do_dot_net_garbage_collection(&self) -> Result<(), Error> {
        // The engine declares one reserved parameter, optional and defaulting
        // to zero. Supplying it keeps the call correct if the default ever
        // stops being applied.
        self.dispatch
            .call(dispid::DO_DOT_NET_GARBAGE_COLLECTION, &[Value::I32(0)])?;
        Ok(())
    }

    /// How often the engine collects .NET garbage, in milliseconds
    /// (`Engine.DotNetGarbageCollectionInterval`).
    ///
    /// Zero or less means automatic collection is off. A host built on this
    /// crate will normally read `-1`, and that is correct rather than broken:
    /// the three-second default belongs to applications built on the UI
    /// control, and a headless host does not create one. Nothing collects on a
    /// timer unless this is set to a positive interval, so a long-lived host
    /// that runs .NET steps should either set one or call
    /// [`do_dot_net_garbage_collection`](Self::do_dot_net_garbage_collection)
    /// between runs.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn dot_net_garbage_collection_interval(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(dispid::DOT_NET_GARBAGE_COLLECTION_INTERVAL)?
            .as_i32()?)
    }

    /// Sets the .NET collection interval, in milliseconds
    /// (`Engine.DotNetGarbageCollectionInterval`).
    ///
    /// Zero or less switches automatic collection off.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_dot_net_garbage_collection_interval(&self, milliseconds: i32) -> Result<(), Error> {
        self.dispatch.put(
            dispid::DOT_NET_GARBAGE_COLLECTION_INTERVAL,
            Value::I32(milliseconds),
        )?;
        Ok(())
    }

    /// The .NET runtime version the engine loaded (`Engine.DotNetCLRVersion`).
    ///
    /// Empty on a station where nothing has pulled the runtime in yet, so treat
    /// an empty string as "not loaded" rather than as an error.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn dot_net_clr_version(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(dispid::DOT_NET_CLR_VERSION)?
            .into_string()?)
    }

    /// The station's user list, as a file (`Engine.UsersFile`).
    ///
    /// The users the engine loaded at startup, and the only route to writing
    /// them back. [`new_user`](Self::new_user) builds a user in memory; without
    /// saving through this file the station is unchanged once the process
    /// exits.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn users_file(&self) -> Result<crate::UsersFile, Error> {
        Ok(crate::UsersFile::new(
            self.dispatch.get(dispid::USERS_FILE)?.into_object()?,
        ))
    }

    /// Whether the host polls for messages (`Engine.UIMessagePollingEnabled`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn ui_message_polling_enabled(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(dispid::UI_MESSAGE_POLLING_ENABLED)?
            .as_bool()?)
    }

    /// Turns message polling on or off (`Engine.UIMessagePollingEnabled`).
    ///
    /// Off by default. A headless host must turn it on before anything appears
    /// in the queue, without it the queue stays empty however much a sequence
    /// posts.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_ui_message_polling_enabled(&self, enabled: bool) -> Result<(), Error> {
        self.dispatch
            .put(dispid::UI_MESSAGE_POLLING_ENABLED, Value::Bool(enabled))?;
        Ok(())
    }

    /// Whether the message queue is empty (`Engine.IsUIMessageQueueEmpty`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_ui_message_queue_empty(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(dispid::IS_UI_MESSAGE_QUEUE_EMPTY)?
            .as_bool()?)
    }

    /// Takes the next message from the queue (`Engine.GetUIMessage`).
    ///
    /// Check [`is_ui_message_queue_empty`](Self::is_ui_message_queue_empty)
    /// first. The message must be acknowledged once handled, see
    /// [`UIMessage::acknowledge`](crate::UIMessage::acknowledge).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_ui_message(&self) -> Result<crate::UIMessage, Error> {
        Ok(crate::UIMessage::new(
            self.dispatch
                .call(dispid::GET_UI_MESSAGE, &[])?
                .into_object()?,
        ))
    }

    /// Creates a step (`Engine.NewStep`).
    ///
    /// `adapter_key_name` selects the code-module adapter, see
    /// [`AdapterKeyName`](crate::AdapterKeyName). `step_type_name` names the
    /// step type, for example `NumericLimitTest` or `Action`.
    ///
    /// An empty key does **not** mean "no code module". It means the step type
    /// chooses, falling back to the station's `DefaultAdapter` when the type
    /// designates none, so an empty key on an `Action` yields whatever adapter
    /// the station happens to default to. Pass
    /// [`AdapterKeyName::NoneAdapter`](crate::AdapterKeyName::NoneAdapter) to
    /// actually mean no code module.
    ///
    /// The step is not part of any sequence until it is inserted.
    ///
    /// # Errors
    /// [`Error`] if the step type is unknown or the COM call fails.
    pub fn new_step(
        &self,
        adapter_key_name: &str,
        step_type_name: &str,
    ) -> Result<crate::Step, Error> {
        Ok(crate::Step::new(
            self.dispatch
                .call(
                    dispid::NEW_STEP,
                    &[
                        Value::Str(adapter_key_name.to_owned()),
                        Value::Str(step_type_name.to_owned()),
                    ],
                )?
                .into_object()?,
        ))
    }

    /// Creates a sequence (`Engine.NewSequence`).
    ///
    /// The sequence is not part of any file until it is inserted.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn new_sequence(&self) -> Result<crate::Sequence, Error> {
        Ok(crate::Sequence::new(
            self.dispatch
                .call(dispid::NEW_SEQUENCE, &[])?
                .into_object()?,
        ))
    }

    /// Creates a user account object (`Engine.NewUser`).
    ///
    /// Pass an existing user as `profile` to inherit its privileges; the new
    /// user does **not** join any group the profile belongs to. Pass `None` for
    /// a user with no privileges.
    ///
    /// The result exists only in memory, nothing is written to the station's
    /// users file by creating one.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn new_user(
        &self,
        profile: Option<&crate::users::User>,
    ) -> Result<crate::users::User, Error> {
        // The engine reads the profile's privileges, so it needs a real
        // handle; a null means "no privileges to inherit".
        // The profile is required, and "no profile" is a null object
        // reference, a VT_DISPATCH holding nothing. VT_NULL and VT_EMPTY are
        // both refused here, and omitting the argument reports it as missing.
        let argument = profile
            .and_then(crate::users::User::duplicate_dispatch)
            .map_or(Value::NullObject, Value::Object);
        Ok(crate::users::User::new(
            self.dispatch
                .call(dispid::NEW_USER, &[argument])?
                .into_object()?,
        ))
    }

    /// Finds a user by login name (`Engine.GetUser`).
    ///
    /// Returns `None` when no user has that name, rather than erroring.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_user(&self, login_name: &str) -> Result<Option<crate::users::User>, Error> {
        match self
            .dispatch
            .call(dispid::GET_USER, &[Value::Str(login_name.to_owned())])?
        {
            Value::Object(dispatch) => Ok(Some(crate::users::User::new(dispatch))),
            Value::Null | Value::Empty => Ok(None),
            other => Err(Error::UnexpectedType {
                expected: "Object or Null",
                actual: other.kind(),
            }),
        }
    }

    /// Whether a login name is already taken (`Engine.UserNameExists`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn user_name_exists(&self, login_name: &str) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                dispid::USER_NAME_EXISTS,
                &[Value::Str(login_name.to_owned())],
            )?
            .as_bool()?)
    }

    /// The user currently logged in (`Engine.CurrentUser`).
    ///
    /// Returns `None` when nobody is logged in, which is the normal state on a
    /// station that does not require a login.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn current_user(&self) -> Result<Option<crate::users::User>, Error> {
        match self.dispatch.get(dispid::CURRENT_USER)? {
            Value::Object(dispatch) => Ok(Some(crate::users::User::new(dispatch))),
            Value::Null | Value::Empty => Ok(None),
            other => Err(Error::UnexpectedType {
                expected: "Object or Null",
                actual: other.kind(),
            }),
        }
    }

    /// Whether the logged-in user holds a privilege
    /// (`Engine.CurrentUserHasPrivilege`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn current_user_has_privilege(
        &self,
        privilege: crate::users::UserPrivilege,
    ) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                dispid::CURRENT_USER_HAS_PRIVILEGE,
                &[Value::Str(privilege.name().to_owned())],
            )?
            .as_bool()?)
    }

    /// Creates a standalone `PropertyObject` (`Engine.NewPropertyObject`).
    ///
    /// The object belongs to no sequence file or station; it is useful as the
    /// root of a tree you build in memory. Pass a type name only when
    /// `value_type` is `NamedType`.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn new_property_object(
        &self,
        value_type: crate::PropValType,
        as_array: bool,
        type_name: &str,
        options: i32,
    ) -> Result<crate::property::PropertyObject, Error> {
        Ok(crate::property::PropertyObject::new(
            self.dispatch
                .call(
                    dispid::NEW_PROPERTY_OBJECT,
                    &[
                        Value::I32(value_type as i32),
                        Value::Bool(as_array),
                        Value::Str(type_name.to_owned()),
                        Value::I32(options),
                    ],
                )?
                .into_object()?,
        ))
    }

    /// Shuts the engine down and leaves this thread's COM apartment.
    ///
    /// For a host that owns the engine on a **spawned** thread. Such a thread
    /// really does detach when it ends, so the apartment it initialized has to
    /// be closed or the COM runtime is left believing a live thread still owns
    /// one. The process's main thread does not need this: it is ending anyway.
    ///
    /// Consuming `self` is what makes the ordering safe, the engine is
    /// released before the apartment closes, and no caller can hold a reference
    /// across the boundary.
    ///
    /// # Errors
    /// [`Error`] if a COM call during shutdown fails. The apartment is closed
    /// either way.
    pub fn close(self, timeout: std::time::Duration) -> Result<bool, Error> {
        let confirmed = self.shutdown(timeout);
        rs_teststand_sys::close_apartment(self.dispatch);
        confirmed
    }

    /// Closes files, terminates executions, and waits for the engine to say it
    /// is done (`Engine.ShutDown`).
    ///
    /// `ShutDown` is **asynchronous**. It returns as soon as the request is
    /// accepted, having only *started* terminating executions and closing
    /// files; the engine reports completion later by posting
    /// [`UIMessageCode::ShutDownComplete`](crate::UIMessageCode::ShutDownComplete)
    /// to its message queue. So a caller that simply calls it and drops the
    /// engine tears down COM underneath work that is still running.
    ///
    /// This does the whole protocol: enables message polling, asks the engine
    /// to shut down, then pumps and drains until the engine confirms or
    /// `timeout` elapses.
    ///
    /// Returns `true` when the engine confirmed. `false` means the timeout came
    /// first, or the engine posted
    /// [`ShutDownCanceled`](crate::UIMessageCode::ShutDownCanceled), which a
    /// sequence can cause, for instance by refusing to terminate. Either way the
    /// wait is **bounded**: an unattended host must not be able to hang here.
    ///
    /// Shutting down twice is harmless; the second call simply finds nothing to
    /// do and returns once the engine answers.
    ///
    /// # Errors
    /// [`Error`] if a COM call fails.
    pub fn shutdown(&self, timeout: std::time::Duration) -> Result<bool, Error> {
        // Without polling the completion message goes to an event sink that a
        // headless caller does not have, and the wait could never end.
        self.set_ui_message_polling_enabled(true)?;
        self.dispatch
            .call(dispid::SHUT_DOWN, &[Value::Bool(true)])?;

        let started = std::time::Instant::now();
        while started.elapsed() < timeout {
            if crate::pump_thread_messages() {
                return Ok(false);
            }
            while !self.is_ui_message_queue_empty()? {
                let message = self.get_ui_message()?;
                let code = crate::UIMessageCode::from_bits(message.event()?);
                // Acknowledge before deciding: an unacknowledged synchronous
                // message would hold up the very shutdown being waited on.
                message.acknowledge()?;
                match code {
                    Ok(crate::UIMessageCode::ShutDownComplete) => return Ok(true),
                    Ok(crate::UIMessageCode::ShutDownCanceled) => return Ok(false),
                    _ => {}
                }
            }
        }
        Ok(false)
    }

    /// The station's templates file (`Engine.GetTemplatesFile`).
    ///
    /// Holds the variable, step and sequence prototypes the editor offers when
    /// inserting. It is a station-wide file, so it is empty until someone adds
    /// templates to it, an empty one is the normal state, not a failure.
    ///
    /// A template is an ordinary [`PropertyObject`](crate::PropertyObject), not
    /// a type of its own, so a program is free to keep its own prototypes in a
    /// container it builds itself rather than in this file.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_templates_file(
        &self,
        options: crate::GetTemplatesFileOptions,
    ) -> Result<crate::property::PropertyObjectFile, Error> {
        Ok(crate::property::PropertyObjectFile::new(
            self.dispatch
                .call(dispid::GET_TEMPLATES_FILE, &[Value::I32(options.bits())])?
                .into_object()?,
        ))
    }

    /// Opens a sequence file, or returns the already-loaded one
    /// (`Engine.GetSequenceFileEx`).
    ///
    /// The engine caches the file and counts load references, so every
    /// successful call must be paired with
    /// [`release_sequence_file_ex`](Self::release_sequence_file_ex).
    ///
    /// Both option arguments matter on an unattended host:
    /// [`crate::sequence::GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK`] suppresses a load
    /// callback that could raise a dialog, and [`crate::sequence::ConflictHandler::Error`]
    /// fails the load instead of prompting.
    ///
    /// # Errors
    /// [`Error`] if the file cannot be opened or the COM call fails.
    pub fn get_sequence_file_ex(
        &self,
        path: &str,
        options: crate::sequence::GetSeqFileOptions,
        handler: crate::sequence::ConflictHandler,
    ) -> Result<crate::sequence::SequenceFile, Error> {
        let dispatch = self
            .dispatch
            .call(
                dispid::GET_SEQUENCE_FILE_EX,
                &[
                    Value::Str(path.to_owned()),
                    Value::I32(options.bits()),
                    Value::I32(handler.bits()),
                ],
            )?
            .into_object()?;
        Ok(crate::sequence::SequenceFile::new(dispatch))
    }

    /// Drops one load reference on a sequence file
    /// (`Engine.ReleaseSequenceFileEx`).
    ///
    /// Returns `true` when that was the last reference and the engine has
    /// discarded the file. `false` means something else still holds it open,
    /// so the file stays loaded, which is why only the `true` case also
    /// releases the wrapper's own COM reference.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn release_sequence_file_ex(
        &self,
        sequence_file: crate::sequence::SequenceFile,
        options: i32,
    ) -> Result<bool, Error> {
        let released = self
            .dispatch
            .call(
                dispid::RELEASE_SEQUENCE_FILE_EX,
                &[
                    Value::Object(sequence_file.into_dispatch()),
                    Value::I32(options),
                ],
            )?
            .as_bool()?;
        Ok(released)
    }

    /// Accesses the collection of search directories (`Engine.SearchDirectories`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn search_directories(&self) -> Result<crate::station::SearchDirectories, Error> {
        let dispatch = self
            .dispatch
            .get(dispid::SEARCH_DIRECTORIES)?
            .into_object()?;
        Ok(crate::station::SearchDirectories::new(dispatch))
    }

    /// Accesses the station global variables container (`Engine.Globals`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn globals(&self) -> Result<crate::property::PropertyObject, Error> {
        let dispatch = self.dispatch.get(dispid::GLOBALS)?.into_object()?;
        Ok(crate::property::PropertyObject::new(dispatch))
    }

    /// Creates a new workspace file object (`Engine.NewWorkspaceFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn new_workspace_file(&self) -> Result<crate::workspace::WorkspaceFile, Error> {
        let dispatch = self
            .dispatch
            .call(dispid::NEW_WORKSPACE_FILE, &[])?
            .into_object()?;
        Ok(crate::workspace::WorkspaceFile::new(dispatch))
    }

    /// Opens an existing workspace file (`Engine.OpenWorkspaceFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn open_workspace_file(
        &self,
        path: &str,
        read_only: bool,
        options: i32,
    ) -> Result<crate::workspace::WorkspaceFile, Error> {
        let dispatch = self
            .dispatch
            .call(
                dispid::OPEN_WORKSPACE_FILE,
                &[
                    Value::Str(path.to_string()),
                    Value::Bool(read_only),
                    Value::I32(options),
                ],
            )?
            .into_object()?;
        Ok(crate::workspace::WorkspaceFile::new(dispatch))
    }

    /// Flushes modified station globals and configuration to disk (`Engine.CommitGlobalsToDisk`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn commit_globals_to_disk(&self, prompt_on_save_conflicts: bool) -> Result<(), Error> {
        self.dispatch.call(
            dispid::COMMIT_GLOBALS_TO_DISK,
            &[Value::Bool(prompt_on_save_conflicts)],
        )?;
        Ok(())
    }

    /// Builds an engine over a caller-supplied dispatch handle. Test-only seam
    /// for exercising wrapper logic against a fake, with no live COM.
    #[cfg(test)]
    pub(crate) fn from_dispatch(dispatch: Box<dyn Dispatch>) -> Self {
        Self {
            dispatch,
            // Nothing was created, so nothing could have asked anything.
            startup_dialogs: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Value};

    use super::{Engine, dispid};
    use crate::error::Error;

    /// A scripted response for one dispatch id, so the fake needs no COM and no
    /// `Clone` on `Value`.
    #[derive(Debug, Clone)]
    enum Scripted {
        Bool(bool),
        I32(i32),
        Str(&'static str),
        Fail(i32),
    }

    #[derive(Debug)]
    struct FakeDispatch {
        responses: HashMap<i32, Scripted>,
        /// Every `put` and `call` in order, so a test can assert what a wrapper
        /// sent rather than only what it read back.
        written: Written,
    }

    /// Shared with the test, because `Engine` takes the dispatch by value.
    type Written = std::rc::Rc<core::cell::RefCell<Vec<(i32, Value)>>>;

    impl FakeDispatch {
        fn new(entries: impl IntoIterator<Item = (i32, Scripted)>, written: Written) -> Self {
            Self {
                responses: entries.into_iter().collect(),
                written,
            }
        }
    }

    impl rs_teststand_sys::Dispatch for FakeDispatch {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.responses.get(&dispid) {
                Some(Scripted::Bool(value)) => Ok(Value::Bool(*value)),
                Some(Scripted::I32(value)) => Ok(Value::I32(*value)),
                Some(Scripted::Str(value)) => Ok(Value::Str((*value).to_owned())),
                Some(Scripted::Fail(code)) => Err(ComError::hresult(*code, "fake")),
                None => Err(ComError::hresult(0, "fake: unscripted dispid")),
            }
        }

        fn put(&self, dispid: i32, value: Value) -> Result<(), ComError> {
            self.written.borrow_mut().push((dispid, value));
            Ok(())
        }

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            let first = match args.first() {
                Some(Value::I32(value)) => Value::I32(*value),
                _ => Value::Empty,
            };
            self.written.borrow_mut().push((dispid, first));
            Ok(Value::Empty)
        }
    }

    fn engine_with(entries: impl IntoIterator<Item = (i32, Scripted)>) -> Engine {
        engine_recording(entries).0
    }

    /// An engine plus the log of everything it writes.
    fn engine_recording(entries: impl IntoIterator<Item = (i32, Scripted)>) -> (Engine, Written) {
        let written: Written = std::rc::Rc::default();
        let dispatch = FakeDispatch::new(entries, std::rc::Rc::clone(&written));
        (Engine::from_dispatch(Box::new(dispatch)), written)
    }

    /// What a wrapper sent, reduced to the shapes these members use.
    ///
    /// `Value` carries COM payloads that have no meaningful equality, so it does
    /// not implement `PartialEq`. Comparing the handful of scalar cases here is
    /// enough and keeps that out of the public type.
    #[derive(Debug, PartialEq, Eq)]
    enum Sent {
        Empty,
        Bool(bool),
        I32(i32),
        Other,
    }

    impl From<&Value> for Sent {
        fn from(value: &Value) -> Self {
            match *value {
                Value::Empty => Self::Empty,
                Value::Bool(flag) => Self::Bool(flag),
                Value::I32(number) => Self::I32(number),
                _ => Self::Other,
            }
        }
    }

    /// Whether the log holds exactly this one entry.
    fn wrote(written: &Written, dispid: i32, expected: &Sent) -> bool {
        let log = written.borrow();
        matches!(log.as_slice(), [(id, sent)] if *id == dispid && Sent::from(sent) == *expected)
    }

    #[test]
    fn major_version_reads_i4_property() -> Result<(), Error> {
        let engine = engine_with([(dispid::MAJOR_VERSION, Scripted::I32(26))]);
        assert_eq!(engine.major_version()?, 26);
        Ok(())
    }

    #[test]
    fn version_string_reads_bstr_property() -> Result<(), Error> {
        let engine = engine_with([(dispid::VERSION_STRING, Scripted::Str("26.0.0.123"))]);
        assert_eq!(engine.version_string()?, "26.0.0.123");
        Ok(())
    }

    #[test]
    fn is_64bit_reads_bool_property() -> Result<(), Error> {
        let engine = engine_with([(dispid::IS_64BIT, Scripted::Bool(true))]);
        assert!(engine.is_64bit()?);
        Ok(())
    }

    #[test]
    fn directories_read_bstr_properties() -> Result<(), Error> {
        let engine = engine_with([
            (dispid::TESTSTAND_DIRECTORY, Scripted::Str("T:\\TestStand")),
            (dispid::BIN_DIRECTORY, Scripted::Str("T:\\TestStand\\Bin")),
            (
                dispid::CONFIG_DIRECTORY,
                Scripted::Str("T:\\TestStand\\Cfg"),
            ),
        ]);
        assert_eq!(engine.teststand_directory()?, "T:\\TestStand");
        assert_eq!(engine.bin_directory()?, "T:\\TestStand\\Bin");
        assert_eq!(engine.config_directory()?, "T:\\TestStand\\Cfg");
        Ok(())
    }

    #[test]
    fn unload_all_modules_calls_the_method_with_no_arguments() -> Result<(), Error> {
        let (engine, written) = engine_recording([]);
        engine.unload_all_modules()?;
        assert!(
            wrote(&written, dispid::UNLOAD_ALL_MODULES, &Sent::Empty),
            "expected one argument-free call, got {written:?}",
        );
        Ok(())
    }

    #[test]
    fn breakpoints_enabled_round_trips() -> Result<(), Error> {
        let engine = engine_with([(dispid::BREAKPOINTS_ENABLED, Scripted::Bool(true))]);
        assert!(engine.breakpoints_enabled()?);

        let (engine, written) = engine_recording([]);
        engine.set_breakpoints_enabled(false)?;
        assert!(
            wrote(&written, dispid::BREAKPOINTS_ENABLED, &Sent::Bool(false)),
            "expected the flag to be written as a bool, got {written:?}",
        );
        Ok(())
    }

    #[test]
    fn persist_breakpoints_round_trips() -> Result<(), Error> {
        let engine = engine_with([(dispid::PERSIST_BREAKPOINTS, Scripted::Bool(false))]);
        assert!(!engine.persist_breakpoints()?);

        let (engine, written) = engine_recording([]);
        engine.set_persist_breakpoints(true)?;
        assert!(
            wrote(&written, dispid::PERSIST_BREAKPOINTS, &Sent::Bool(true)),
            "expected the flag to be written as a bool, got {written:?}",
        );
        Ok(())
    }

    #[test]
    fn dot_net_collection_passes_the_reserved_argument() -> Result<(), Error> {
        // The engine declares the parameter optional with a zero default. Send
        // it explicitly so the call stays correct if the default is dropped.
        let (engine, written) = engine_recording([]);
        engine.do_dot_net_garbage_collection()?;
        assert!(
            wrote(
                &written,
                dispid::DO_DOT_NET_GARBAGE_COLLECTION,
                &Sent::I32(0)
            ),
            "expected the reserved argument to be sent as zero, got {written:?}",
        );
        Ok(())
    }

    #[test]
    fn dot_net_collection_interval_round_trips() -> Result<(), Error> {
        let engine = engine_with([(
            dispid::DOT_NET_GARBAGE_COLLECTION_INTERVAL,
            Scripted::I32(30_000),
        )]);
        assert_eq!(engine.dot_net_garbage_collection_interval()?, 30_000);

        let (engine, written) = engine_recording([]);
        engine.set_dot_net_garbage_collection_interval(5_000)?;
        assert!(
            wrote(
                &written,
                dispid::DOT_NET_GARBAGE_COLLECTION_INTERVAL,
                &Sent::I32(5_000)
            ),
            "expected the interval to be written as an i4, got {written:?}",
        );
        Ok(())
    }

    #[test]
    fn dot_net_clr_version_is_empty_when_the_runtime_is_not_loaded() -> Result<(), Error> {
        // Documented behavior: empty means "not loaded", not "failed".
        let engine = engine_with([(dispid::DOT_NET_CLR_VERSION, Scripted::Str(""))]);
        assert_eq!(engine.dot_net_clr_version()?, "");
        Ok(())
    }

    #[test]
    fn com_failure_propagates_as_typed_error() {
        // 0x8004_2001 stands in for an engine HRESULT; the exact code must survive.
        let engine = engine_with([(dispid::MAJOR_VERSION, Scripted::Fail(-2_147_209_215))]);
        let result = engine.major_version();
        assert!(
            matches!(result, Err(Error::Com { hresult, .. }) if hresult == -2_147_209_215),
            "expected Com error carrying the HRESULT, got {result:?}",
        );
    }

    #[test]
    fn wrong_variant_type_is_reported_not_coerced() {
        // Property answers with a string where the wrapper wants an i32.
        let engine = engine_with([(dispid::MAJOR_VERSION, Scripted::Str("not a number"))]);
        let result = engine.major_version();
        assert!(
            matches!(
                result,
                Err(Error::UnexpectedType {
                    expected: "I32",
                    ..
                })
            ),
            "expected a type-mismatch error, got {result:?}",
        );
    }
}
