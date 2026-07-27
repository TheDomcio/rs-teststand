//! Dispatch identifiers (DISPIDs) for TestStand™ COM members.
//!
//! All constants are sourced from TestStand™ type library ground truth.

#![allow(dead_code)]

/// Internal macro to declare DISPID constant groups without repetitive boilerplate.
macro_rules! define_dispid_group {
    (
        $(#[$mod_doc:meta])*
        $mod_name:ident {
            $(
                $(#[$doc:meta])*
                $name:ident = $val:expr
            );* $(;)?
        }
    ) => {
        $(#[$mod_doc])*
        // `pub(crate)`: these tables are internal ground-truth constants, not
        // public API. The wrappers reference them; consumers never see a DISPID.
        pub(crate) mod $mod_name {
            $(
                $(#[$doc])*
                pub(crate) const $name: i32 = $val;
            )*
        }
    };
}

define_dispid_group! {
    /// Members of the `Engine` object (`IEngine`).
    engine {
        SHUT_DOWN = 0x0001;
        MAJOR_VERSION = 0x0135;
        MINOR_VERSION = 0x0136;
        REVISION_VERSION = 0x0137;
        VERSION_STRING = 0x0138;
        CONFIG_DIRECTORY = 0x0140;
        BIN_DIRECTORY = 0x0141;
        TESTSTAND_DIRECTORY = 0x0142;
        COMMIT_GLOBALS_TO_DISK = 0x011e;
        NEW_PROPERTY_OBJECT = 0x00aa;
        NEW_SEQUENCE_FILE = 0x000b;
        NEW_SEQUENCE = 0x001e;
        NEW_EXECUTION = 0x006e;
        GET_UI_MESSAGE = 0x00bf;
        UI_MESSAGE_POLLING_ENABLED = 0x017c;
        IS_UI_MESSAGE_QUEUE_EMPTY = 0x017d;
        POST_UI_MESSAGE = 0x0238;
        LOAD_TYPE_PALETTE_FILES = 0x01e2;
        UNLOAD_TYPE_PALETTE_FILES = 0x01e3;
        LOAD_TYPE_PALETTE_FILES_EX = 0x0232;
        NEW_STEP = 0x0032;
        NEW_USER = 0x0046;
        USER_NAME_EXISTS = 0x0047;
        GET_USER = 0x0048;
        CURRENT_USER_HAS_PRIVILEGE = 0x0049;
        CURRENT_USER = 0x012d;
        GET_SEQUENCE_FILE_EX = 0x01ab;
        RELEASE_SEQUENCE_FILE_EX = 0x01c5;
        SEARCH_DIRECTORIES = 0x0231;
        STATION_OPTIONS = 0x023b;
        GLOBALS = 0x012c;
        NEW_WORKSPACE_FILE = 0x0235;
        OPEN_WORKSPACE_FILE = 0x0236;
        CURRENT_WORKSPACE_FILE = 0x01c6;
        GET_TEMPLATES_FILE = 0x026e;
        BUILD_VERSION = 0x02d0;
        IS_64BIT = 0x0324;
    }
}

define_dispid_group! {
    /// Members of the `StationOptions` object (`IStationOptions`).
    station_options {
        AUTO_LOGIN_SYSTEM_USER = 0x0003;
        RTE_OPTION = 0x0005;
        TRACING_ENABLED = 0x0006;
        BREAKPOINTS_ENABLED = 0x0007;
        DISABLE_RESULTS = 0x0008;
        ALWAYS_GOTO_CLEANUP_ON_FAILURE = 0x0009;
        INTERACTIVE_BRANCH_MODE = 0x000b;
        SET_TIME_LIMIT_ENABLED = 0x002c;
        SET_TIME_LIMIT = 0x002d;
        SET_TIME_LIMIT_ACTION = 0x002e;
        SHOW_HIDDEN_PROPERTIES = 0x000f;
        PROMPT_TO_FIND_FILES = 0x0010;
        STATION_ID = 0x0019;
        USE_STATION_MODEL = 0x001b;
        ALLOW_OTHER_MODELS = 0x001c;
        LANGUAGE = 0x001e;
        USE_LOCALIZED_DECIMAL_POINT = 0x001f;
        ALLOW_SEQUENCE_CALLS_FROM_REMOTE_MACHINE = 0x0022;
        ALLOW_ALL_USERS_ACCESS_FROM_REMOTE_MACHINE = 0x0023;
        SHOW_ENGINE_TRAY_ICON_ON_REMOTE_STATIONS = 0x0024;
        CHECK_OUT_FILES_WHEN_EDITED = 0x0025;
        PROMPT_WHEN_ADDING_FILES_TO_SC = 0x0026;
        USE_DIALOG_FOR_CHECK_OUT = 0x0027;
        CHECK_OUT_ONLY_SELECTED_FILES = 0x0028;
        UI_MESSAGE_DELAY = 0x002a;
        UI_MESSAGE_MIN_DELAY = 0x002b;
        INTERACTIVE_EXE_PROPAGATE_STATUS = 0x002f;
        BREAK_ON_STEP_FAILURE = 0x0030;
        BREAK_ON_SEQUENCE_FAILURE = 0x0031;
        DEBUG_OPTIONS = 0x0032;
        DEFAULT_FILE_WRITING_FORMAT = 0x003c;
        ALLOW_AUTOMATIC_TYPE_CONFLICT_RESOLUTION = 0x003d;
        FILE_MODIFICATION_INDICATOR_POLICY = 0x003e;
        PRELOAD_PROGRESS_DELAY = 0x0040;
        AUTO_CREATE_VARIABLE_LOCATION = 0x0048;
        ALLOW_CANCELLING_PRELOAD_EXPRESSION = 0x0041;
        DEFAULT_CPU_AFFINITY_FOR_THREADS = 0x003f;
        DEFAULT_CPU_AFFINITY_FOR_THREADS_EX = 0x0042;
        ENABLE_USER_PRIVILEGE_CHECKING = 0x0002;
        EXECUTION_MASK = 0x000a;
        LOGIN_ON_START = 0x0043;
        RECOGNIZE_MB_CHARS = 0x0020;
        RELOAD_DOCS_WHEN_OPENING_WORKSPACE = 0x0017;
        RELOAD_WORKSPACE_AT_STARTUP = 0x0018;
        REQUIRE_USER_LOGIN = 0x0004;
        STATION_MODEL_SEQUENCE_FILE_PATH = 0x001d;
        SYSTEM_DEFAULT_SOURCE_CODE_CONTROL_PROVIDER = 0x0029;
        TYPE_VERSION_AUTO_INCREMENT_PROMPT_OPT = 0x0016;
        USER_FILE_PATH = 0x0001;
    }
}

define_dispid_group! {
    /// Members of the `SearchDirectories` collection (`ISearchDirectories`).
    search_directories {
        ITEM = 0x0000;
        COUNT = 0x0002;
        INSERT = 0x0003;
        REMOVE = 0x0004;
        MOVE_SEARCH_DIRECTORY = 0x0005;
        RELOAD = 0x0006;
    }
}

define_dispid_group! {
    /// Members of a `SearchDirectory` object (`ISearchDirectory`).
    search_directory {
        TYPE = 0x0001;
        PATH = 0x0002;
        SEARCH_SUBDIRECTORIES = 0x0003;
        FILE_EXTENSION_RESTRICTIONS = 0x0004;
        EXCLUDE_FILE_EXTENSION = 0x0005;
        DISABLED = 0x0006;
        EXCLUDE_HIDDEN_SUBDIRECTORIES = 0x0007;
    }
}

define_dispid_group! {
    /// Members of the `PropertyObject` object (`IPropertyObject`).
    property_object {
        GET_VAL_NUMBER = 0x01f5;
        SET_VAL_NUMBER = 0x01f6;
        GET_VAL_BOOLEAN = 0x01f7;
        SET_VAL_BOOLEAN = 0x01f8;
        GET_VAL_STRING = 0x01f9;
        SET_VAL_STRING = 0x01fa;
        GET_PROPERTY_OBJECT = 0x01fb;
        SET_PROPERTY_OBJECT = 0x01fc;
        GET_TYPE = 0x0214;
        NEW_SUB_PROPERTY = 0x0226;
        DELETE_SUB_PROPERTY = 0x0227;
        EXISTS = 0x023a;
        CLONE = 0x023b;
    }
}

define_dispid_group! {
    /// Members of the `WorkspaceFile` object (`IWorkspaceFile`).
    workspace_file {
        ROOT_WORKSPACE_OBJECT = 0x0001;
        AS_PROPERTY_OBJECT_FILE = 0x0004;
        IS_CONNECTED_TO_SC_PROVIDER = 0x0002;
        PROVIDER_NAME = 0x0003;
        FIND_WORKSPACE_OBJECT = 0x0006;
        SAVE_WORKSPACE_AND_PROJECT_FILES = 0x000a;
    }
}

define_dispid_group! {
    /// Members of the `WorkspaceObject` object (`IWorkspaceObject`).
    workspace_object {
        NEW_FOLDER = 0x0001;
        NEW_FILE = 0x0002;
        INSERT_OBJECT = 0x0003;
        PATH = 0x0004;
        DISPLAY_NAME = 0x0005;
        IS_CODE_MODULE = 0x0006;
        CODE_MODULE_SEQ_FILE_PATH = 0x0007;
        REMOVE_OBJECT = 0x0009;
        OBJECT_TYPE = 0x000b;
        SOURCE_CONTROL_STATUS = 0x000d;
        AS_PROPERTY_OBJECT = 0x000f;
        GET_ABSOLUTE_PATH = 0x0010;
        PROJECT_FILE = 0x0011;
        FILE_EXISTS = 0x0014;
        GET_PARENT_CONTAINER = 0x0017;
        NUM_CONTAINED_OBJECTS = 0x0018;
        GET_CONTAINED_OBJECT = 0x0019;
    }
}

/// `PropertyObjectFile` dispinterface members.
pub(crate) mod property_object_file {
    /// `TypeUsageList`, read-only.
    pub(crate) const TYPE_USAGE_LIST: i32 = 0x65;
    /// `IncChangeCount`, no params.
    pub(crate) const INC_CHANGE_COUNT: i32 = 0x66;
    /// `Path`, read/write.
    pub(crate) const PATH: i32 = 0x6c;
    /// `Data`, read-only — the file's root property object.
    pub(crate) const DATA: i32 = 0x6e;
}

/// `TypeUsageList` dispinterface members.
pub(crate) mod type_usage_list {
    /// `NumTypes`, read-only.
    pub(crate) const NUM_TYPES: i32 = 0x1;
    /// `GetTypeDefinition`, 1 param.
    pub(crate) const GET_TYPE_DEFINITION: i32 = 0x2;
    /// `GetTypeIndex`, 1 param.
    pub(crate) const GET_TYPE_INDEX: i32 = 0x3;
    /// `InsertType`, 3 params.
    pub(crate) const INSERT_TYPE: i32 = 0x4;
    /// `RemoveType`, 1 param.
    pub(crate) const REMOVE_TYPE: i32 = 0x5;
    /// `ChangeCount`, read-only.
    pub(crate) const CHANGE_COUNT: i32 = 0x9;
}

/// `User` dispinterface members.
pub(crate) mod user {
    /// `ValidatePassword`, 1 param.
    pub(crate) const VALIDATE_PASSWORD: i32 = 0x1;
    /// `HasPrivilege`, 1 param.
    pub(crate) const HAS_PRIVILEGE: i32 = 0x2;
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x3;
    /// `LoginName`, read/write.
    pub(crate) const LOGIN_NAME: i32 = 0x14;
    /// `Password`, read/write.
    pub(crate) const PASSWORD: i32 = 0x15;
    /// `FullName`, read/write.
    pub(crate) const FULL_NAME: i32 = 0x16;
    /// `Privileges`, read-only.
    pub(crate) const PRIVILEGES: i32 = 0x18;
    /// `Members`, read-only.
    pub(crate) const MEMBERS: i32 = 0x19;
}

/// `ArrayDimensions` dispinterface members.
pub(crate) mod array_dimensions {
    /// `SetBoundsByStrings`, 2 params.
    pub(crate) const SET_BOUNDS_BY_STRINGS: i32 = 0x6;
    /// `LowerBoundsString`, read-only.
    pub(crate) const LOWER_BOUNDS_STRING: i32 = 0x4;
    /// `UpperBoundsString`, read-only.
    pub(crate) const UPPER_BOUNDS_STRING: i32 = 0x5;
    /// `NumDimensions`, read-only.
    pub(crate) const NUM_DIMENSIONS: i32 = 0x14;
}

/// `PropertyObjectType` dispinterface members.
pub(crate) mod property_object_type {
    /// `ValueType`, read-only.
    pub(crate) const VALUE_TYPE: i32 = 0x1;
    /// `IsObject`, read-only.
    pub(crate) const IS_OBJECT: i32 = 0x2;
    /// `Representation`, read/write.
    pub(crate) const REPRESENTATION: i32 = 0xb;
    /// `ArrayDimensions`, read-only.
    pub(crate) const ARRAY_DIMENSIONS: i32 = 0x6;
    /// `DisplayString`, read-only.
    pub(crate) const DISPLAY_STRING: i32 = 0x5;
}

/// `PropertyObject` introspection members.
pub(crate) mod property_object_introspect {
    /// `GetType`, 5 params — three are `[out]` byref, so it is not bindable
    /// through the current in-only dispatch seam. Recorded for completeness.
    #[allow(dead_code, reason = "needs byref out-param support in the sys crate")]
    pub(crate) const GET_TYPE: i32 = 0x214;
    /// `GetNumSubProperties`, 1 param.
    pub(crate) const GET_NUM_SUB_PROPERTIES: i32 = 0x228;
    /// `GetNthSubPropertyName`, 3 params.
    pub(crate) const GET_NTH_SUB_PROPERTY_NAME: i32 = 0x229;
    /// `GetNthSubProperty`, 3 params.
    pub(crate) const GET_NTH_SUB_PROPERTY: i32 = 0x282;
    /// `GetTypeDisplayString`, 2 params.
    pub(crate) const GET_TYPE_DISPLAY_STRING: i32 = 0x249;
    /// `GetTypeFlags`, 2 params.
    pub(crate) const GET_TYPE_FLAGS: i32 = 0x26c;
    /// `Type`, read-only — yields a `PropertyObjectType`.
    pub(crate) const TYPE: i32 = 0x284;
    /// `GetValInteger64`, 2 params.
    pub(crate) const GET_VAL_INTEGER64: i32 = 0x2d0;
    /// `SetValInteger64`, 3 params.
    pub(crate) const SET_VAL_INTEGER64: i32 = 0x2d1;
    /// `GetValUnsignedInteger64`, 2 params.
    pub(crate) const GET_VAL_UNSIGNED_INTEGER64: i32 = 0x2d4;
    /// `SetValUnsignedInteger64`, 3 params.
    pub(crate) const SET_VAL_UNSIGNED_INTEGER64: i32 = 0x2d5;
    /// `Evaluate`, 1 param — obsolete, kept for older engines.
    pub(crate) const EVALUATE: i32 = 0x23c;
    /// `EvaluateEx`, 2 params.
    pub(crate) const EVALUATE_EX: i32 = 0x28a;
    /// `Name`, read/write.
    pub(crate) const NAME: i32 = 0x24e;
    /// `TypeVersion`, read/write.
    pub(crate) const TYPE_VERSION: i32 = 0x26b;
    /// `Attributes`, read-only.
    pub(crate) const ATTRIBUTES: i32 = 0x286;
    /// `Enumerators`, read-only.
    pub(crate) const ENUMERATORS: i32 = 0x32a;
    /// `UpdateEnumerators`, 1 param.
    pub(crate) const UPDATE_ENUMERATORS: i32 = 0x32b;
    /// `GetValueDisplayName`, 2 params.
    pub(crate) const GET_VALUE_DISPLAY_NAME: i32 = 0x32d;
    /// `NumericFormat`, read/write.
    pub(crate) const NUMERIC_FORMAT: i32 = 0x276;
    /// `GetFormattedValue`, 5 params.
    pub(crate) const GET_FORMATTED_VALUE: i32 = 0x24a;
    /// `GetPropertyObjectByOffset`, 2 params.
    pub(crate) const GET_PROPERTY_OBJECT_BY_OFFSET: i32 = 0x254;
    /// `SetNumElements`, 2 params.
    pub(crate) const SET_NUM_ELEMENTS: i32 = 0x24c;
    /// `GetNumElements`, no params.
    pub(crate) const GET_NUM_ELEMENTS: i32 = 0x24b;
}

/// `Execution` dispinterface members.
pub(crate) mod execution {
    /// `Terminate`, no params.
    pub(crate) const TERMINATE: i32 = 0x8;
    /// `WaitForEnd`, obsolete; superseded by `WaitForEndEx`.
    pub(crate) const WAIT_FOR_END: i32 = 0xc;
    /// `WaitForEndEx`, 4 params (last two optional).
    pub(crate) const WAIT_FOR_END_EX: i32 = 0x14;
    /// `Id`, read-only.
    pub(crate) const ID: i32 = 0x7b;
    /// `Break`, no params — suspends the execution.
    pub(crate) const BREAK: i32 = 0x1;
    /// `Resume`, no params.
    pub(crate) const RESUME: i32 = 0x2;
    /// `Abort`, no params — stops without running cleanup.
    pub(crate) const ABORT: i32 = 0x7;
    /// `CancelTermination`, no params.
    pub(crate) const CANCEL_TERMINATION: i32 = 0xa;
    /// `GetThread`, 1 param.
    pub(crate) const GET_THREAD: i32 = 0xd;
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1d;
    /// `GetSequenceFile`, no params.
    pub(crate) const GET_SEQUENCE_FILE: i32 = 0xe;
    /// `StepOver`, no params.
    pub(crate) const STEP_OVER: i32 = 0x3;
    /// `StepInto`, no params.
    pub(crate) const STEP_INTO: i32 = 0x4;
    /// `StepOut`, no params.
    pub(crate) const STEP_OUT: i32 = 0x5;
    /// `GetFileGlobals`, no params.
    pub(crate) const GET_FILE_GLOBALS: i32 = 0x84;
    /// `ResultStatus`, read-only string.
    pub(crate) const RESULT_STATUS: i32 = 0x78;
    /// `DisplayName`, read-only string.
    pub(crate) const DISPLAY_NAME: i32 = 0x7c;
    /// `NumThreads`, read-only.
    pub(crate) const NUM_THREADS: i32 = 0x32;
    /// `ForegroundThread`, read-only.
    pub(crate) const FOREGROUND_THREAD: i32 = 0x85;
    /// `SequenceFilePath`, read-only string.
    pub(crate) const SEQUENCE_FILE_PATH: i32 = 0x7e;
    /// `SecondsExecuting`, read-only double.
    pub(crate) const SECONDS_EXECUTING: i32 = 0x81;
    /// `SecondsSuspended`, read-only double.
    pub(crate) const SECONDS_SUSPENDED: i32 = 0x82;
    /// `ErrorObject`, read-only.
    pub(crate) const ERROR_OBJECT: i32 = 0x79;
    /// `ResultObject`, read-only.
    pub(crate) const RESULT_OBJECT: i32 = 0x7d;
    /// `GetStates`, 2 `[out]` byref params — not bindable through the current
    /// in-only dispatch seam. Recorded so nobody re-derives it.
    #[allow(dead_code, reason = "needs byref out-param support in the sys crate")]
    pub(crate) const GET_STATES: i32 = 0x13;
}

/// `Thread` dispinterface members.
pub(crate) mod thread {
    /// `GetSequenceContext`, 2 params — the second is an optional `[out]`
    /// byref frame id, which is omitted.
    pub(crate) const GET_SEQUENCE_CONTEXT: i32 = 0x5;
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1d;
    /// `PostUIMessageEx`, 5 params; the fourth is `VT_UNKNOWN`.
    pub(crate) const POST_UI_MESSAGE_EX: i32 = 0x29;
    /// `WaitForEnd`, 2 params.
    pub(crate) const WAIT_FOR_END: i32 = 0x9;
    /// `Id`, read-only.
    pub(crate) const ID: i32 = 0x20;
    /// `DisplayName`, read-only string.
    pub(crate) const DISPLAY_NAME: i32 = 0x1e;
    /// `CallStackSize`, read-only.
    pub(crate) const CALL_STACK_SIZE: i32 = 0x21;
    /// `UniqueThreadId`, read-only string.
    pub(crate) const UNIQUE_THREAD_ID: i32 = 0x33;
    /// `Execution`, read-only.
    pub(crate) const EXECUTION: i32 = 0x1f;
    /// `ExternallySuspended`, read-only — true once a `Break` has taken effect.
    pub(crate) const EXTERNALLY_SUSPENDED: i32 = 0x28;
}

/// `SequenceContext` dispinterface members.
pub(crate) mod sequence_context {
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1d;
    /// `Locals`, read-only.
    pub(crate) const LOCALS: i32 = 0x36;
    /// `Parameters`, read-only.
    pub(crate) const PARAMETERS: i32 = 0x37;
    /// `FileGlobals`, read-only — the run's own copy, not the file's defaults.
    pub(crate) const FILE_GLOBALS: i32 = 0x38;
    /// `StationGlobals`, read-only — outlives the execution.
    pub(crate) const STATION_GLOBALS: i32 = 0x39;
    /// `Execution`, read-only.
    pub(crate) const EXECUTION: i32 = 0x31;
    /// `Thread`, read-only.
    pub(crate) const THREAD: i32 = 0x30;
    /// `SequenceFile`, read-only — outlives the execution.
    pub(crate) const SEQUENCE_FILE: i32 = 0x22;
    /// `Sequence`, read-only.
    pub(crate) const SEQUENCE: i32 = 0x21;
    /// `Step`, read-only — present only while a step executes.
    pub(crate) const STEP: i32 = 0x1e;
    /// `StepIndex`, read-only.
    pub(crate) const STEP_INDEX: i32 = 0x23;
    /// `NumStepsExecuted`, read-only.
    pub(crate) const NUM_STEPS_EXECUTED: i32 = 0x57;
    /// `CallStackDepth`, read-only.
    pub(crate) const CALL_STACK_DEPTH: i32 = 0x3a;
    /// `SequenceFailed`, read-only.
    pub(crate) const SEQUENCE_FAILED: i32 = 0x41;
    /// `ErrorReported`, read-only.
    pub(crate) const ERROR_REPORTED: i32 = 0x50;
    /// `RunTimeErrorMessage`, read-only.
    pub(crate) const RUN_TIME_ERROR_MESSAGE: i32 = 0x2a;
    /// `Root`, read-only — the context's own property tree root, which is what
    /// an expression means by `ThisContext`.
    pub(crate) const ROOT: i32 = 0x2e;
    /// `GetMultipleValues`, `[out]` byref — not bindable through the in-only
    /// dispatch seam. Recorded so nobody re-derives it.
    #[allow(dead_code, reason = "needs byref out-param support in the sys crate")]
    pub(crate) const GET_MULTIPLE_VALUES: i32 = 0x58;
}

/// `UIMessage` dispinterface members.
pub(crate) mod ui_message {
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1d;
    /// `Event`, read-only.
    pub(crate) const EVENT: i32 = 0x1e;
    /// `IsSynchronous`, read-only.
    pub(crate) const IS_SYNCHRONOUS: i32 = 0x1f;
    /// `Execution`, read-only.
    pub(crate) const EXECUTION: i32 = 0x20;
    /// `Thread`, read-only.
    pub(crate) const THREAD: i32 = 0x21;
    /// `NumericData`, read-only.
    pub(crate) const NUMERIC_DATA: i32 = 0x22;
    /// `StringData`, read-only.
    pub(crate) const STRING_DATA: i32 = 0x23;
    /// `ActiveXData`, read-only. Declared `VT_UNKNOWN`, not `VT_DISPATCH`.
    pub(crate) const ACTIVE_X_DATA: i32 = 0x25;
    /// `Acknowledge`, no params.
    pub(crate) const ACKNOWLEDGE: i32 = 0x3c;
}

/// `Step` dispinterface members.
pub(crate) mod step {
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1;
    /// `AdapterKeyName`, read/write — a string, not a number.
    pub(crate) const ADAPTER_KEY_NAME: i32 = 0x15;
    /// `RunMode`, read/write — a string, not a number.
    pub(crate) const RUN_MODE: i32 = 0x17;
    /// `Precondition`, read/write.
    pub(crate) const PRECONDITION: i32 = 0x1a;
    /// `PostExpression`, read/write.
    pub(crate) const POST_EXPRESSION: i32 = 0x53;
    /// `CreateNewUniqueStepId`, no params.
    pub(crate) const CREATE_NEW_UNIQUE_STEP_ID: i32 = 0x9c;
    /// `StepType`, read-only.
    pub(crate) const STEP_TYPE: i32 = 0x22;
    /// `RecordResult`, read/write.
    pub(crate) const RECORD_RESULT: i32 = 0x54;
    /// `Name`, read/write.
    pub(crate) const NAME: i32 = 0x57;
    /// `ResultRecordingOption`, read/write.
    pub(crate) const RESULT_RECORDING_OPTION: i32 = 0xd2;
}

/// `Sequence` dispinterface members.
pub(crate) mod sequence {
    /// `Parameters`, read-only.
    pub(crate) const PARAMETERS: i32 = 0x32;
    /// `Locals`, read-only.
    pub(crate) const LOCALS: i32 = 0x33;
    /// `Name`, read/write.
    pub(crate) const NAME: i32 = 0x39;
    /// `GetNumSteps`, 1 param.
    pub(crate) const GET_NUM_STEPS: i32 = 0x1;
    /// `GetStep`, 2 params.
    pub(crate) const GET_STEP: i32 = 0x2;
    /// `InsertStep`, 3 params.
    pub(crate) const INSERT_STEP: i32 = 0x4;
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0xb;
    /// `CreateNewUniqueStepIds`, no params.
    pub(crate) const CREATE_NEW_UNIQUE_STEP_IDS: i32 = 0x46;
}

/// `SequenceFile` dispinterface members.
pub(crate) mod sequence_file {
    /// `GetSequence`, 1 parameter.
    pub(crate) const GET_SEQUENCE: i32 = 0x1;
    /// `GetSequenceByName`, 1 parameter.
    pub(crate) const GET_SEQUENCE_BY_NAME: i32 = 0x2;
    /// `AsPropertyObjectFile`, no params.
    pub(crate) const AS_PROPERTY_OBJECT_FILE: i32 = 0xe;
    /// `Save`, 1 parameter.
    pub(crate) const SAVE: i32 = 0x7;
    /// `NumSequences`, read-only.
    pub(crate) const NUM_SEQUENCES: i32 = 0x32;
    /// `Path`, read-only.
    pub(crate) const PATH: i32 = 0x34;
    /// `InsertSequence`, 1 param.
    pub(crate) const INSERT_SEQUENCE: i32 = 0x4;
    /// `FileGlobalsDefaultValues`, read-only.
    pub(crate) const FILE_GLOBALS_DEFAULT_VALUES: i32 = 0x33;
    /// `GetSequenceIndex`, 1 parameter.
    pub(crate) const GET_SEQUENCE_INDEX: i32 = 0xc;
}
