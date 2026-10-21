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
        NEW_PROPERTY_OBJECT_FILE = 0x019f;
        NUM_ADAPTERS = 0x0131;
        GET_ADAPTER = 0x005a;
        GET_ADAPTER_BY_KEY_NAME = 0x005b;
        GET_TYPE_PALETTE_FILE_LIST = 0x01e4;
        SET_TYPE_PALETTE_FILE_LIST = 0x01e5;
        NEW_SEQUENCE_FILE = 0x000b;
        NEW_SEQUENCE = 0x001e;
        NEW_EXECUTION = 0x006e;
        GET_EXECUTION = 0x046b;
        NEW_EXPRESSION = 0x023a;
        LOCALIZE_EXPRESSION = 0x0191;
        DELOCALIZE_EXPRESSION = 0x0192;
        CHECK_EXPR_SYNTAX = 0x0118;
        CHECK_EXPRESSION = 0x01fe;
        GET_UI_MESSAGE = 0x00bf;
        UI_MESSAGE_POLLING_ENABLED = 0x017c;
        IS_UI_MESSAGE_QUEUE_EMPTY = 0x017d;
        POST_UI_MESSAGE = 0x0238;
        USERS_FILE = 0x01a1;
        TERMINATE_ALL = 0x0072;
        ABORT_ALL = 0x0070;
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
        GET_STATION_MODEL_SEQUENCE_FILE = 0x0120;
        SEARCH_DIRECTORIES = 0x0231;
        STATION_OPTIONS = 0x023b;
        GLOBALS = 0x012c;
        NEW_WORKSPACE_FILE = 0x0235;
        OPEN_WORKSPACE_FILE = 0x0236;
        CURRENT_WORKSPACE_FILE = 0x01c6;
        GET_TEMPLATES_FILE = 0x026e;
        BUILD_VERSION = 0x02d0;
        IS_64BIT = 0x0324;
        UNLOAD_ALL_MODULES = 0x011f;
        BREAKPOINTS_ENABLED = 0x0169;
        PERSIST_BREAKPOINTS = 0x0246;
        WATCH_EXPRESSIONS_ENABLED = 0x0242;
        PERSIST_WATCH_EXPRESSIONS = 0x0247;
        GET_WATCH_EXPRESSIONS_CHANGE_COUNT = 0x0226;
        GET_WATCH_EXPRESSIONS = 0x0227;
        SEARCH_FILES = 0x0225;
        GET_SYNC_MANAGER = 0x0203;
        GET_OUTPUT_MESSAGES = 0x025b;
        NEW_OUTPUT_MESSAGE = 0x025c;
        OUTPUT_MESSAGES_ENABLED = 0x025d;
        NEW_OUTPUT_MESSAGES = 0x025e;
        NEW_UNDO_ITEM_CREATOR = 0x025a;
        NEW_INTERACTIVE_ARGS = 0x0082;
        NEW_UNDO_STACK = 0x025f;
        UNDO_LIMIT = 0x027a;
        FIND_FILE = 0x0104;
        FIND_PATH = 0x0105;
        NEW_STEP_TYPE = 0x01a6;
        SERIALIZE_OBJECTS = 0x01d0;
        NEW_RESULT_LOG = 0x02bd;
        NEW_RESULT_LOGGER = 0x02bc;
        UNSERIALIZE_OBJECTS = 0x01d1;
        GET_TYPE_DEFINITION = 0x01dc;
        GET_TYPE_NAMES = 0x01dd;
        DIFF_SEQUENCE_FILES = 0x0258;
        NEW_CSV_FILE_INPUT_RECORD_STREAM = 0x041d;
        NEW_CSV_FILE_OUTPUT_RECORD_STREAM = 0x041e;
        DOT_NET_GARBAGE_COLLECTION_INTERVAL = 0x02be;
        DO_DOT_NET_GARBAGE_COLLECTION = 0x02bf;
        DOT_NET_CLR_VERSION = 0x02c3;
        LICENSE_TYPE = 0x0245;
        GET_LICENSE_DESCRIPTION = 0x0248;
        ACQUIRE_LICENSE = 0x0277;
        RELEASE_LICENSE = 0x0278;
        APPLICATION_LICENSE = 0x0279;
        HAS_ADDON_LICENSE = 0x0325;
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
        GET_VAL_NUMBER_BY_OFFSET = 0x0263;
        SET_VAL_NUMBER_BY_OFFSET = 0x0264;
        GET_VAL_BOOLEAN_BY_OFFSET = 0x0250;
        SET_VAL_BOOLEAN_BY_OFFSET = 0x0251;
        GET_VAL_STRING_BY_OFFSET = 0x0252;
        SET_VAL_STRING_BY_OFFSET = 0x0253;
        SET_PROPERTY_OBJECT_BY_OFFSET = 0x0255;
        GET_VAL_IDISPATCH_BY_OFFSET = 0x0258;
        SET_VAL_IDISPATCH_BY_OFFSET = 0x0259;
        GET_VAL_INTERFACE_BY_OFFSET = 0x025a;
        SET_VAL_INTERFACE_BY_OFFSET = 0x025b;
        GET_VAL_INTEGER64_BY_OFFSET = 0x02d2;
        SET_VAL_INTEGER64_BY_OFFSET = 0x02d3;
        GET_VAL_UNSIGNED_INTEGER64_BY_OFFSET = 0x02d6;
        SET_VAL_UNSIGNED_INTEGER64_BY_OFFSET = 0x02d7;
        UNSERIALIZE_EX = 0x026a;
        SERIALIZE = 0x023f;
        SET_VAL_NUMBER = 0x01f6;
        GET_VAL_BOOLEAN = 0x01f7;
        SET_VAL_BOOLEAN = 0x01f8;
        GET_VAL_STRING = 0x01f9;
        SET_VAL_STRING = 0x01fa;
        GET_PROPERTY_OBJECT = 0x01fb;
        SET_PROPERTY_OBJECT = 0x01fc;
        GET_TYPE = 0x0214;
        GET_XML = 0x027a;
        SET_XML = 0x027f;
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

/// `Adapter` dispinterface members.
pub(crate) mod adapter {
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x2;
    /// `DisplayName`, read-only string.
    pub(crate) const DISPLAY_NAME: i32 = 0x16;
    /// `IconName`, read-only string.
    pub(crate) const ICON_NAME: i32 = 0x2c;
    /// `IsConfigurable`, read-only boolean.
    pub(crate) const IS_CONFIGURABLE: i32 = 0x14;
    /// `KeyName`, read-only string.
    pub(crate) const KEY_NAME: i32 = 0x15;
    /// `Hidden`, read/write boolean.
    pub(crate) const HIDDEN: i32 = 0x2d;
    /// `ShowArgsInStepDescription`, read/write boolean.
    pub(crate) const SHOW_ARGS_IN_STEP_DESCRIPTION: i32 = 0x2e;
    /// `SmallIconIndex`, read-only integer.
    pub(crate) const SMALL_ICON_INDEX: i32 = 0x2a;
    /// `LargeIconIndex`, read-only integer.
    pub(crate) const LARGE_ICON_INDEX: i32 = 0x2b;
}

/// `PropertyObjectFile` dispinterface members.
pub(crate) mod property_object_file {
    /// `ReadFile`, one optional conflict-handler parameter.
    pub(crate) const READ_FILE: i32 = 0x68;
    /// `FileType`, read-only enum.
    pub(crate) const FILE_TYPE: i32 = 0x78;
    /// `TypeUsageList`, read-only.
    pub(crate) const TYPE_USAGE_LIST: i32 = 0x65;
    /// `IncChangeCount`, no params.
    pub(crate) const INC_CHANGE_COUNT: i32 = 0x66;
    /// `IsModified`, read-only.
    pub(crate) const IS_MODIFIED: i32 = 0x67;
    /// `ChangeCount`, read/write.
    pub(crate) const CHANGE_COUNT: i32 = 0x6b;
    /// `Path`, read/write.
    pub(crate) const PATH: i32 = 0x6c;
    /// `Data`, read-only, the file's root property object.
    pub(crate) const DATA: i32 = 0x6e;
    /// `SaveFileIfModified`, one `VT_BOOL` param, returns `VT_BOOL`.
    pub(crate) const SAVE_FILE_IF_MODIFIED: i32 = 0x7a;
    /// `IsDiskFileModified`, read-only.
    pub(crate) const IS_DISK_FILE_MODIFIED: i32 = 0x77;
    /// `IsDiskFileReadOnly`, read-only.
    pub(crate) const IS_DISK_FILE_READ_ONLY: i32 = 0x79;
    /// `Version`, read/write.
    pub(crate) const VERSION: i32 = 0x7e;
    /// `WriteFile`, one format parameter.
    pub(crate) const WRITE_FILE: i32 = 0x69;
}

/// `UsersFile` dispinterface members.
pub(crate) mod users_file {
    /// `UserList`, read-only here, an array of `User` objects.
    pub(crate) const USER_LIST: i32 = 0x1;
    /// `UserProfileList`, read-only here.
    pub(crate) const USER_PROFILE_LIST: i32 = 0x2;
    /// `ReloadFromDisk`, no params.
    pub(crate) const RELOAD_FROM_DISK: i32 = 0x3;
    /// `AsPropertyObjectFile`, no params.
    pub(crate) const AS_PROPERTY_OBJECT_FILE: i32 = 0x4;
    /// `UserGroupList`, read-only here.
    pub(crate) const USER_GROUP_LIST: i32 = 0x5;
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
    /// `GetIsTypeAttachedToFile`, 1 param, returns `VT_BOOL`.
    pub(crate) const GET_IS_TYPE_ATTACHED_TO_FILE: i32 = 0x7;
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
    /// `GetType`, 5 params, three are `[out]` byref, so it is not bindable
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
    /// `IsTypeDefinition`, read-only boolean.
    pub(crate) const IS_TYPE_DEFINITION: i32 = 0x25f;
    /// `GetFlags`, 2 params.
    pub(crate) const GET_FLAGS: i32 = 0x212;
    /// `SetFlags`, 3 params.
    pub(crate) const SET_FLAGS: i32 = 0x213;
    /// `GetInstanceDefaultFlags`, 2 params.
    pub(crate) const GET_INSTANCE_DEFAULT_FLAGS: i32 = 0x25d;
    /// `SetInstanceDefaultFlags`, 3 params.
    pub(crate) const SET_INSTANCE_DEFAULT_FLAGS: i32 = 0x25e;
    /// `GetInstanceOverrideFlags`, 2 params.
    pub(crate) const GET_INSTANCE_OVERRIDE_FLAGS: i32 = 0x26e;
    /// `SetInstanceOverrideFlags`, 3 params.
    pub(crate) const SET_INSTANCE_OVERRIDE_FLAGS: i32 = 0x26f;
    /// `Comment`, read/write.
    pub(crate) const COMMENT: i32 = 0x24f;
    /// `Type`, read-only, yields a `PropertyObjectType`.
    pub(crate) const TYPE: i32 = 0x284;
    /// `GetValInteger64`, 2 params.
    pub(crate) const GET_VAL_INTEGER64: i32 = 0x2d0;
    /// `SetValInteger64`, 3 params.
    pub(crate) const SET_VAL_INTEGER64: i32 = 0x2d1;
    /// `GetValUnsignedInteger64`, 2 params.
    pub(crate) const GET_VAL_UNSIGNED_INTEGER64: i32 = 0x2d4;
    /// `SetValUnsignedInteger64`, 3 params.
    pub(crate) const SET_VAL_UNSIGNED_INTEGER64: i32 = 0x2d5;
    /// `Evaluate`, 1 param, obsolete, kept for older engines.
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
    /// `Search`, 9 parameters.
    pub(crate) const SEARCH: i32 = 0x27b;
}

/// Members of `ResultLogger`.
pub(crate) mod result_logger {
    /// `Open`, path and two boolean flags.
    pub(crate) const OPEN: i32 = 0x32;
    /// `Close`, no parameters.
    pub(crate) const CLOSE: i32 = 0x33;
    /// `Path`, read/write string.
    pub(crate) const PATH: i32 = 0x3f;
    /// `ContinueInNewFile`, one path.
    pub(crate) const CONTINUE_IN_NEW_FILE: i32 = 0x40;
    /// `Flush`, no parameters.
    pub(crate) const FLUSH: i32 = 0x41;
    /// `DeferrableRename`, one path.
    pub(crate) const DEFERRABLE_RENAME: i32 = 0x42;
    /// `GetFilesWritten`, one clear flag.
    pub(crate) const GET_FILES_WRITTEN: i32 = 0x43;
    /// `RecordFilesWritten`, read/write boolean.
    pub(crate) const RECORD_FILES_WRITTEN: i32 = 0x44;
    /// `LogBegin`, thread, parent, model type, data, path, socket.
    pub(crate) const LOG_BEGIN: i32 = 0x34;
    /// `LogPreBatch`, model entry-point parameters.
    pub(crate) const LOG_PRE_BATCH: i32 = 0x35;
    /// `LogBatchStart`, model entry-point parameters.
    pub(crate) const LOG_BATCH_START: i32 = 0x36;
    /// `LogPreUUT`, model entry-point parameters.
    pub(crate) const LOG_PRE_UUT: i32 = 0x37;
    /// `LogUUTStart`, model entry-point parameters.
    pub(crate) const LOG_UUT_START: i32 = 0x38;
    /// `LogOnTheFlyStepResults`, thread, UUT, steps, results, callbacks, parents.
    pub(crate) const LOG_ON_THE_FLY_STEP_RESULTS: i32 = 0x39;
    /// `LogUUTDone`, thread, data, UUT, status, result.
    pub(crate) const LOG_UUT_DONE: i32 = 0x3a;
    /// `LogPostUUT`, thread, data, UUT, status.
    pub(crate) const LOG_POST_UUT: i32 = 0x3b;
    /// `LogBatchDone`, thread, data, UUT, status.
    pub(crate) const LOG_BATCH_DONE: i32 = 0x3c;
    /// `LogPostBatch`, thread, data, UUT, status.
    pub(crate) const LOG_POST_BATCH: i32 = 0x3d;
    /// `LogEnd`, thread and model data.
    pub(crate) const LOG_END: i32 = 0x3e;
}

/// Members of `ExecutionOutputRecordStreams`.
pub(crate) mod execution_output_record_streams {
    /// `NewStream`, one name.
    pub(crate) const NEW_STREAM: i32 = 0x1;
    /// `GetStream`, one name.
    pub(crate) const GET_STREAM: i32 = 0x2;
    /// `CloseAndRemoveStream`, one name.
    pub(crate) const CLOSE_AND_REMOVE_STREAM: i32 = 0x3;
    /// `CloseAll`, no parameters.
    pub(crate) const CLOSE_ALL: i32 = 0x4;
    /// `RemoveAll`, no parameters.
    pub(crate) const REMOVE_ALL: i32 = 0x5;
    /// `BeginNextUUTOrBatch`, no parameters.
    pub(crate) const BEGIN_NEXT_UUT_OR_BATCH: i32 = 0x6;
}

/// Members of `ExecutionOutputRecordStream`.
pub(crate) mod execution_output_record_stream {
    /// `Name`, read-only string.
    pub(crate) const NAME: i32 = 0x1;
    /// `DefineFields`, an object and an optional mapping string.
    pub(crate) const DEFINE_FIELDS: i32 = 0x5;
    /// `Insert`, one output record stream.
    pub(crate) const INSERT: i32 = 0x2;
    /// `Remove`, one output record stream.
    pub(crate) const REMOVE: i32 = 0x3;
    /// `WriteRecord`, an object and an optional mapping string.
    pub(crate) const WRITE_RECORD: i32 = 0x6;
    /// `WriteRecordFrom`, a context object and a string array.
    pub(crate) const WRITE_RECORD_FROM: i32 = 0x7;
    /// `Flush`, no parameters.
    pub(crate) const FLUSH: i32 = 0x8;
    /// `Close`, no parameters.
    pub(crate) const CLOSE: i32 = 0x9;
    /// `NumRecordsToDisplayInReportView`, read/write integer.
    pub(crate) const NUM_RECORDS_TO_DISPLAY: i32 = 0xa;
    /// `SetActiveReport`, no parameters.
    pub(crate) const SET_ACTIVE_REPORT: i32 = 0xb;
    /// `ClearRecordsFromReportView`, no parameters.
    pub(crate) const CLEAR_RECORDS_FROM_REPORT_VIEW: i32 = 0xc;
    /// `CloseAtNextUUTOrBatch`, read/write boolean.
    pub(crate) const CLOSE_AT_NEXT_UUT_OR_BATCH: i32 = 0xd;
    /// `Format`, read/write string.
    pub(crate) const FORMAT: i32 = 0xe;
}

/// Members of `ResultLog`.
pub(crate) mod result_log {
    /// `Open`, one path.
    pub(crate) const OPEN: i32 = 0x32;
    /// `Close`, no parameters.
    pub(crate) const CLOSE: i32 = 0x33;
    /// `ReadNextRecord`, four outputs.
    pub(crate) const READ_NEXT_RECORD: i32 = 0x34;
    /// `SetReportPaths`, one property object.
    pub(crate) const SET_REPORT_PATHS: i32 = 0x35;
    /// `GetReportPaths`, no parameters.
    pub(crate) const GET_REPORT_PATHS: i32 = 0x36;
    /// `OnTheFly`, read-only boolean.
    pub(crate) const ON_THE_FLY: i32 = 0x37;
    /// `Path`, read-only string.
    pub(crate) const PATH: i32 = 0x38;
    /// `ClosedWhenWritten`, read-only boolean.
    pub(crate) const CLOSED_WHEN_WRITTEN: i32 = 0x39;
    /// `UniqueId`, read-only string.
    pub(crate) const UNIQUE_ID: i32 = 0x3a;
    /// `SecondsAtStartIn1970UniversalCoordinatedTime`, read-only double.
    pub(crate) const SECONDS_AT_START: i32 = 0x3b;
    /// `SimulateOnTheFly`, read/write boolean.
    pub(crate) const SIMULATE_ON_THE_FLY: i32 = 0x3c;
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
    /// `Break`, no params, suspends the execution.
    pub(crate) const BREAK: i32 = 0x1;
    /// `Resume`, no params.
    pub(crate) const RESUME: i32 = 0x2;
    /// `Abort`, no params, stops without running cleanup.
    pub(crate) const ABORT: i32 = 0x7;
    /// `CancelTermination`, no params.
    pub(crate) const CANCEL_TERMINATION: i32 = 0xa;
    /// `GetThread`, 1 param.
    pub(crate) const GET_THREAD: i32 = 0xd;
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1d;
    /// `OutputRecordStreams`, read-only collection. Data Streams API, 2017+.
    pub(crate) const OUTPUT_RECORD_STREAMS: i32 = 0xdc;
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
    /// `ThreadIds`, read-only, returns `VT_ARRAY | VT_I4`.
    pub(crate) const THREAD_IDS: i32 = 0xce;
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
    /// `Report`, read-only.
    pub(crate) const REPORT: i32 = 0x7a;
    /// `Reports`, read-only collection.
    pub(crate) const REPORTS: i32 = 0xbe;
    /// `GetStates`, 2 `[out]` byref params.
    pub(crate) const GET_STATES: i32 = 0x13;
}

/// `Thread` dispinterface members.
pub(crate) mod thread {
    /// `GetSequenceContext`, 2 params, the second is an optional `[out]`
    /// byref frame id, which is omitted.
    pub(crate) const GET_SEQUENCE_CONTEXT: i32 = 0x5;
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1d;
    /// `PostUIMessageEx`, 5 params; the fourth is `VT_UNKNOWN`.
    pub(crate) const POST_UI_MESSAGE_EX: i32 = 0x29;
    /// Arms a one-shot stop after the next step finishes.
    pub(crate) const SET_STEP_OVER: i32 = 0x2;
    /// Arms a stop at the first step inside what the next step calls.
    pub(crate) const SET_STEP_INTO: i32 = 0x3;
    /// Arms a stop once the current sequence returns.
    pub(crate) const SET_STEP_OUT: i32 = 0x4;
    /// Clears a stop armed by one of the step members.
    pub(crate) const CLEAR_TEMPORARY_BREAKPOINT: i32 = 0x1;
    /// Starts a suspended thread running.
    pub(crate) const RESUME: i32 = 0x2c;
    /// Clears the run-time error recorded against the current step.
    pub(crate) const CLEAR_CURRENT_RTE: i32 = 0x6;
    /// Sends accumulated results to the post-results callbacks now.
    pub(crate) const FLUSH_POST_RESULTS: i32 = 0x3d;
    /// Whether the run will step into the current step's code module.
    pub(crate) const WILL_STEP_INTO_MODULE: i32 = 0x3e;
    /// How the thread answers a terminate request.
    pub(crate) const TERMINATION_OPTION: i32 = 0x3c;
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
    /// `ExternallySuspended`, read-only, true once a `Break` has taken effect.
    pub(crate) const EXTERNALLY_SUSPENDED: i32 = 0x28;
}

/// `SequenceContext` dispinterface members.
pub(crate) mod sequence_context {
    /// `PreviousStep`, read-only.
    pub(crate) const PREVIOUS_STEP: i32 = 0x1f;
    /// `NextStep`, read-only.
    pub(crate) const NEXT_STEP: i32 = 0x20;
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1d;
    /// `Locals`, read-only.
    pub(crate) const LOCALS: i32 = 0x36;
    /// `Parameters`, read-only.
    pub(crate) const PARAMETERS: i32 = 0x37;
    /// `FileGlobals`, read-only, the run's own copy, not the file's defaults.
    pub(crate) const FILE_GLOBALS: i32 = 0x38;
    /// `StationGlobals`, read-only, outlives the execution.
    pub(crate) const STATION_GLOBALS: i32 = 0x39;
    /// `Execution`, read-only.
    pub(crate) const EXECUTION: i32 = 0x31;
    /// `Thread`, read-only.
    pub(crate) const THREAD: i32 = 0x30;
    /// `SequenceFile`, read-only, outlives the execution.
    pub(crate) const SEQUENCE_FILE: i32 = 0x22;
    /// `Sequence`, read-only.
    pub(crate) const SEQUENCE: i32 = 0x21;
    /// `Step`, read-only, present only while a step executes.
    pub(crate) const STEP: i32 = 0x1e;
    /// `StepIndex`, read-only.
    pub(crate) const STEP_INDEX: i32 = 0x23;
    /// `PreviousStepIndex`, read/write.
    pub(crate) const PREVIOUS_STEP_INDEX: i32 = 0x24;
    /// `NextStepIndex`, read/write.
    pub(crate) const NEXT_STEP_INDEX: i32 = 0x25;
    /// `NumStepsExecuted`, read-only.
    pub(crate) const NUM_STEPS_EXECUTED: i32 = 0x57;
    /// `CallStackDepth`, read-only.
    pub(crate) const CALL_STACK_DEPTH: i32 = 0x3a;
    /// `LoopIndex`, read/write.
    pub(crate) const LOOP_INDEX: i32 = 0x3b;
    /// `SequenceFailed`, read-only.
    pub(crate) const SEQUENCE_FAILED: i32 = 0x41;
    /// `ErrorReported`, read-only.
    pub(crate) const ERROR_REPORTED: i32 = 0x50;
    /// `RunTimeErrorMessage`, read-only.
    pub(crate) const RUN_TIME_ERROR_MESSAGE: i32 = 0x2a;
    /// `Root`, read-only, the context's own property tree root, which is what
    /// an expression means by `ThisContext`.
    pub(crate) const ROOT: i32 = 0x2e;
    /// `GetMultipleValues`, `[out]` byref, not bindable through the in-only
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

/// Members of `InteractiveArgs`.
pub(crate) mod interactive_args {
    /// `GetStepIndex`, one index.
    pub(crate) const GET_STEP_INDEX: i32 = 0x1;
    /// `AddStepIndex`, one index.
    pub(crate) const ADD_STEP_INDEX: i32 = 0x2;
    /// `ContainsStep`, one index.
    pub(crate) const CONTAINS_STEP: i32 = 0x3;
    /// `ClearStepList`, no parameters.
    pub(crate) const CLEAR_STEP_LIST: i32 = 0x4;
    /// `AsPropertyObject`, no parameters.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1d;
    /// `StepGroup`, read/write enum.
    pub(crate) const STEP_GROUP: i32 = 0x1e;
    /// `NumSteps`, read-only integer.
    pub(crate) const NUM_STEPS: i32 = 0x1f;
    /// `LoopCount`, read/write integer.
    pub(crate) const LOOP_COUNT: i32 = 0x20;
    /// `StopExpression`, read/write string.
    pub(crate) const STOP_EXPRESSION: i32 = 0x21;
    /// `Sequence`, read/write object reference.
    pub(crate) const SEQUENCE: i32 = 0x22;
}

/// Members of `Module`.
pub(crate) mod module {
    /// `AsPropertyObject`, no parameters.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1;
    /// `Adapter`, read-only object.
    pub(crate) const ADAPTER: i32 = 0x2;
    /// `Step`, read-only object.
    pub(crate) const STEP: i32 = 0x3;
    /// `Load`, options and an optional context.
    pub(crate) const LOAD: i32 = 0x4;
    /// `Unload`, no parameters.
    pub(crate) const UNLOAD: i32 = 0x5;
    /// `CanSpecify`, read-only boolean.
    pub(crate) const CAN_SPECIFY: i32 = 0x6;
    /// `Specify`, options.
    pub(crate) const SPECIFY: i32 = 0x7;
    /// `CanCreateCode`, read-only boolean.
    pub(crate) const CAN_CREATE_CODE: i32 = 0x8;
    /// `CreateCode`, no parameters.
    pub(crate) const CREATE_CODE: i32 = 0x9;
    /// `CanEditCode`, read-only boolean.
    pub(crate) const CAN_EDIT_CODE: i32 = 0xa;
    /// `EditCode`, no parameters.
    pub(crate) const EDIT_CODE: i32 = 0xb;
    /// `GetDescription`, options.
    pub(crate) const GET_DESCRIPTION: i32 = 0xc;
    /// `UnmappedArgumentValues`, read-only object.
    pub(crate) const UNMAPPED_ARGUMENT_VALUES: i32 = 0xc9;
    /// `LoadPrototype`, options.
    pub(crate) const LOAD_PROTOTYPE: i32 = 0xca;
    /// `IsPrototypeIncompatible`, read-only boolean.
    pub(crate) const IS_PROTOTYPE_INCOMPATIBLE: i32 = 0xcb;
    /// `ClearUnmappedArgumentValues`, no parameters (obsolete).
    pub(crate) const CLEAR_UNMAPPED_ARGUMENT_VALUES: i32 = 0xcc;
    /// `GetLastLoadWarnings`, no parameters, string array.
    pub(crate) const GET_LAST_LOAD_WARNINGS: i32 = 0x12d;
}

/// `StepType` dispinterface members.
pub(crate) mod step_type {
    /// `AsPropertyObject`, no parameters.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1;
    /// `Name`, read-only string.
    pub(crate) const NAME: i32 = 0x2;
    /// `NumSubsteps`, read-only integer.
    pub(crate) const NUM_SUBSTEPS: i32 = 0x9;
    /// `GetSubstep`, one index parameter.
    pub(crate) const GET_SUBSTEP: i32 = 0xa;
}

/// `Step` dispinterface members.
pub(crate) mod step {
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1;
    /// `SetBreakOnStepEx`: sets or clears the breakpoint, optionally for one run.
    /// `Step.SetRunModeEx`, which supersedes the obsolete `RunMode` property.
    ///
    /// Takes the mode and an optional execution.
    pub(crate) const SET_RUN_MODE_EX: i32 = 0x4;

    /// `Step.GetRunModeEx`, which supersedes the obsolete `RunMode` property.
    ///
    /// Takes an optional execution and returns a run mode string.
    pub(crate) const GET_RUN_MODE_EX: i32 = 0x5;

    pub(crate) const SET_BREAK_ON_STEP_EX: i32 = 0x6;
    /// `GetBreakOnStepEx`: reads it back, optionally for one run.
    pub(crate) const GET_BREAK_ON_STEP_EX: i32 = 0x7;
    /// `BreakOnStep`: whether the step itself carries a breakpoint.
    pub(crate) const BREAK_ON_STEP: i32 = 0x16;
    /// `SetBreakSettings`: pass count and condition alongside the breakpoint.
    pub(crate) const SET_BREAK_SETTINGS: i32 = 0x96;
    /// `AdapterKeyName`, read/write, a string, not a number.
    pub(crate) const ADAPTER_KEY_NAME: i32 = 0x15;
    /// `ChangeAdapter`, one key-name parameter.
    pub(crate) const CHANGE_ADAPTER: i32 = 0x98;
    /// `ChangeStepType`, one step-type object.
    pub(crate) const CHANGE_STEP_TYPE: i32 = 0x99;
    /// `CanChangeStepType`, one optional step-type object.
    pub(crate) const CAN_CHANGE_STEP_TYPE: i32 = 0x9b;
    /// `RunMode`, read/write, a string, not a number.
    pub(crate) const RUN_MODE: i32 = 0x17;
    /// `Precondition`, read/write.
    pub(crate) const PRECONDITION: i32 = 0x1a;
    /// `PreExpression`, read/write.
    pub(crate) const PRE_EXPRESSION: i32 = 0x52;
    /// `StatusExpression`, read/write.
    pub(crate) const STATUS_EXPRESSION: i32 = 0x50;
    /// `PassAction`, read/write.
    pub(crate) const PASS_ACTION: i32 = 0x28;
    /// `PassActionTarget`, read/write.
    pub(crate) const PASS_ACTION_TARGET: i32 = 0x29;
    /// `FailAction`, read/write.
    pub(crate) const FAIL_ACTION: i32 = 0x2a;
    /// `FailActionTarget`, read/write.
    pub(crate) const FAIL_ACTION_TARGET: i32 = 0x2b;
    /// `LoopType`, read/write.
    pub(crate) const LOOP_TYPE: i32 = 0x3c;
    /// `LoopWhileExpression`, read/write.
    pub(crate) const LOOP_WHILE_EXPRESSION: i32 = 0x3d;
    /// `LoopStatusExpression`, read/write.
    pub(crate) const LOOP_STATUS_EXPRESSION: i32 = 0x3e;
    /// `LoopInitExpression`, read/write.
    pub(crate) const LOOP_INIT_EXPRESSION: i32 = 0x3f;
    /// `LoopIncExpression`, read/write.
    pub(crate) const LOOP_INC_EXPRESSION: i32 = 0x40;
    /// `PostExpression`, read/write.
    pub(crate) const POST_EXPRESSION: i32 = 0x53;
    /// `CreateNewUniqueStepId`, no params.
    pub(crate) const CREATE_NEW_UNIQUE_STEP_ID: i32 = 0x9c;
    /// `StepType`, read-only.
    pub(crate) const STEP_TYPE: i32 = 0x22;
    /// `Module`, read-only property object.
    pub(crate) const MODULE: i32 = 0x9e;
    /// `IsSequenceCall`, read-only boolean.
    pub(crate) const IS_SEQUENCE_CALL: i32 = 0x14;
    /// `RecordResult`, read/write.
    pub(crate) const RECORD_RESULT: i32 = 0x54;
    /// `Name`, read/write.
    pub(crate) const NAME: i32 = 0x57;
    /// `ResultRecordingOption`, read/write.
    pub(crate) const RESULT_RECORDING_OPTION: i32 = 0xd2;
    /// `RecordLoopIterationResults`, read/write.
    pub(crate) const RECORD_LOOP_ITERATION_RESULTS: i32 = 0x92;
}

/// `Sequence` dispinterface members.
pub(crate) mod sequence {
    /// `EvalEntryPointNameExpression`, 1 parameter, returns BSTR.
    pub(crate) const EVAL_ENTRY_POINT_NAME_EXPRESSION: i32 = 0x8;
    /// `EvalEntryPointEnabledExpression`, 1 parameter, returns `VT_BOOL`.
    pub(crate) const EVAL_ENTRY_POINT_ENABLED_EXPRESSION: i32 = 0x9;
    /// `EvalEntryPointNameExpressionEx`, 1 parameter, returns BSTR.
    pub(crate) const EVAL_ENTRY_POINT_NAME_EXPRESSION_EX: i32 = 0x11;
    /// `EvalEntryPointEnabledExpressionEx`, 1 parameter, returns `VT_BOOL`.
    pub(crate) const EVAL_ENTRY_POINT_ENABLED_EXPRESSION_EX: i32 = 0x12;
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
    /// `DeleteStep`, 2 params.
    pub(crate) const DELETE_STEP: i32 = 0x6;
    /// `AsPropertyObject`, no params.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0xb;
    /// `CreateNewUniqueStepIds`, no params.
    pub(crate) const CREATE_NEW_UNIQUE_STEP_IDS: i32 = 0x46;
    /// `Type`, read/write enum.
    pub(crate) const TYPE: i32 = 0x38;
    /// `GotoCleanupOnFailure`, read/write boolean.
    pub(crate) const GOTO_CLEANUP_ON_FAILURE: i32 = 0x3a;
    /// `DisableResults`, read/write boolean.
    pub(crate) const DISABLE_RESULTS: i32 = 0x3b;
    /// `FailureAction`, read/write enum.
    pub(crate) const FAILURE_ACTION: i32 = 0x5a;
    /// `EntryPointNameExpression`, read/write string.
    pub(crate) const ENTRY_POINT_NAME_EXPRESSION: i32 = 0x43;
    /// `ShowEntryPointForAllWindows`, read/write boolean.
    pub(crate) const SHOW_ENTRY_POINT_FOR_ALL_WINDOWS: i32 = 0x3d;
    /// `ShowEntryPointForFileWindow`, read/write boolean.
    pub(crate) const SHOW_ENTRY_POINT_FOR_FILE_WINDOW: i32 = 0x35;
    /// `ShowEntryPointForExeWindow`, read/write boolean.
    pub(crate) const SHOW_ENTRY_POINT_FOR_EXE_WINDOW: i32 = 0x36;
    /// `ShowEntryPointForEditorOnly`, read/write boolean.
    pub(crate) const SHOW_ENTRY_POINT_FOR_EDITOR_ONLY: i32 = 0x3c;
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
    /// `IsExecuting`, read-only.
    pub(crate) const IS_EXECUTING: i32 = 0x39;
    /// `InsertSequence`, 1 param.
    pub(crate) const INSERT_SEQUENCE: i32 = 0x4;
    /// `FileGlobalsDefaultValues`, read-only.
    pub(crate) const FILE_GLOBALS_DEFAULT_VALUES: i32 = 0x33;
    /// `GetSequenceIndex`, 1 parameter.
    pub(crate) const GET_SEQUENCE_INDEX: i32 = 0xc;
}

/// Members of the `Expression` object.
pub(crate) mod expression {
    /// `Text`, read/write, the expression source.
    pub(crate) const TEXT: i32 = 0x1;
    /// `NumTokens`, read-only.
    pub(crate) const NUM_TOKENS: i32 = 0x2;
    /// `Evaluate`, 2 parameters, returns a property object.
    pub(crate) const EVALUATE: i32 = 0x6;
}

/// Members of the engine initialization settings object.
pub(crate) mod engine_initialization_settings {
    /// `SetEnvironmentPath`, one path parameter.
    pub(crate) const SET_ENVIRONMENT_PATH: i32 = 0x1;
    /// `CanInitializeEngine`, no parameters.
    pub(crate) const CAN_INITIALIZE_ENGINE: i32 = 0x2;
}

/// Members of the sequence analyzer object (`IAnalyzer`).
pub(crate) mod analyzer {
    /// `Initialize`, no parameters. Required before any other member.
    pub(crate) const INITIALIZE: i32 = 0x1;
    /// `Engine`, read-only. The engine the analyzer created.
    pub(crate) const ENGINE: i32 = 0x7;
    /// `NewProject`, no parameters.
    pub(crate) const NEW_PROJECT: i32 = 0x9;
    /// `OpenProject`, one path parameter.
    pub(crate) const OPEN_PROJECT: i32 = 0xa;
    /// `UnloadAllModules`, no parameters.
    pub(crate) const UNLOAD_ALL_MODULES: i32 = 0x15;
    /// `LoadRulesFiles`, no parameters.
    pub(crate) const LOAD_RULES_FILES: i32 = 0x17;
    /// `CacheSequenceFiles`, read/write boolean.
    pub(crate) const CACHE_SEQUENCE_FILES: i32 = 0x18;
    /// `NewRuleModule`, no parameters.
    pub(crate) const NEW_RULE_MODULE: i32 = 0xb;
    /// `NewRule`, one rule-id parameter.
    pub(crate) const NEW_RULE: i32 = 0xc;
    /// `RuleModules`, read-only collection.
    pub(crate) const RULE_MODULES: i32 = 0x3;
    /// `NewRuleSet`, one name parameter.
    pub(crate) const NEW_RULE_SET: i32 = 0x13;
    /// `Rules`, read-only collection.
    pub(crate) const RULES: i32 = 0x2;
    /// `RuleSets`, read-only collection.
    pub(crate) const RULE_SETS: i32 = 0x4;
    /// `NewDirectoryToAnalyze`, path and recursive flag.
    pub(crate) const NEW_DIRECTORY_TO_ANALYZE: i32 = 0xd;
}

/// Members of an analyzer project.
pub(crate) mod project {
    /// `Path`, read/write.
    pub(crate) const PATH: i32 = 0x1;
    /// `RuleSet`, read/write object.
    pub(crate) const RULE_SET: i32 = 0xd;
    /// `DirectoriesToAnalyze`, read-only collection.
    pub(crate) const DIRECTORIES_TO_ANALYZE: i32 = 0x3;
    /// `AnalyzeNITypePalettes`, read/write boolean.
    pub(crate) const ANALYZE_NI_TYPE_PALETTES: i32 = 0x5;
    /// `AnalyzePublicTypePalettes`, read/write boolean.
    pub(crate) const ANALYZE_PUBLIC_TYPE_PALETTES: i32 = 0x6;
    /// `AnalyzeUsersFile`, read/write boolean.
    pub(crate) const ANALYZE_USERS_FILE: i32 = 0x7;
    /// `AnalyzeStationGlobalsFile`, read/write boolean.
    pub(crate) const ANALYZE_STATION_GLOBALS_FILE: i32 = 0x8;
    /// `AnalyzeTemplatesFile`, read/write boolean.
    pub(crate) const ANALYZE_TEMPLATES_FILE: i32 = 0x9;
    /// `StartFileAnalysis`, one file parameter.
    pub(crate) const START_FILE_ANALYSIS: i32 = 0xf;
    /// `NewChildProject`, no parameters.
    pub(crate) const NEW_CHILD_PROJECT: i32 = 0x1b;
    /// `RemoveChildProject`, one project.
    pub(crate) const REMOVE_CHILD_PROJECT: i32 = 0x1c;
    /// `AddChildProject`, one project.
    pub(crate) const ADD_CHILD_PROJECT: i32 = 0x1d;
    /// `AnalyzeFilesCalledBySequenceCalls`, read/write boolean.
    pub(crate) const ANALYZE_FILES_CALLED_BY_SEQUENCE_CALLS: i32 = 0x13;
    /// `AnalyzeModelFiles`, read/write boolean.
    pub(crate) const ANALYZE_MODEL_FILES: i32 = 0x14;
    /// `AnalyzeSkippedSteps`, read/write boolean.
    pub(crate) const ANALYZE_SKIPPED_STEPS: i32 = 0x19;
    /// `WorkspaceFileToAnalyze`, read/write.
    pub(crate) const WORKSPACE_FILE_TO_ANALYZE: i32 = 0x2;
    /// `FilesToAnalyze`, read/write string array.
    pub(crate) const FILES_TO_ANALYZE: i32 = 0x4;
    /// `Messages`, read-only.
    pub(crate) const MESSAGES: i32 = 0xa;
    /// `Write`, no parameters.
    pub(crate) const WRITE: i32 = 0xb;
    /// `Read`, no parameters.
    pub(crate) const READ: i32 = 0xc;
    /// `StartAnalysis`, one integer parameter.
    pub(crate) const START_ANALYSIS: i32 = 0xe;
    /// `StopAnalysis`, no parameters.
    pub(crate) const STOP_ANALYSIS: i32 = 0x10;
    /// `IsModified`, read-only.
    pub(crate) const IS_MODIFIED: i32 = 0x11;
    /// `AnalysisStatus`, read-only.
    pub(crate) const ANALYSIS_STATUS: i32 = 0x12;
    /// `AnalyzedFiles`, read-only string array.
    pub(crate) const ANALYZED_FILES: i32 = 0x17;
    /// `AbortAnalysis`, no parameters.
    pub(crate) const ABORT_ANALYSIS: i32 = 0x18;
    /// `NumberOfStepsNotAnalyzed`, read-only.
    pub(crate) const NUMBER_OF_STEPS_NOT_ANALYZED: i32 = 0x1a;
    /// `AutoSave`, read/write boolean.
    pub(crate) const AUTO_SAVE: i32 = 0x1e;
    /// `GenerateReport`, no parameters, returns the report text.
    pub(crate) const GENERATE_REPORT: i32 = 0x15;
}

/// Members of `DirectoriesToAnalyze`.
pub(crate) mod directories_to_analyze {
    /// `Item`, one index parameter.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x2;
    /// `Insert`, index and directory.
    pub(crate) const INSERT: i32 = 0x3;
    /// `Remove`, one index parameter.
    pub(crate) const REMOVE: i32 = 0x4;
    /// `Clear`, no parameters.
    pub(crate) const CLEAR: i32 = 0x5;
}

/// Members of `DirectoryToAnalyze`.
pub(crate) mod directory_to_analyze {
    /// `Path`, read/write.
    pub(crate) const PATH: i32 = 0x1;
    /// `AnalyzeSubDirectories`, read/write boolean.
    pub(crate) const ANALYZE_SUB_DIRECTORIES: i32 = 0x2;
}

/// Members of `AnalysisStatus`.
pub(crate) mod analysis_status {
    /// `IsAnalyzing`, read-only boolean.
    pub(crate) const IS_ANALYZING: i32 = 0x1;
    /// `NumberOfFilesAnalyzed`, read-only integer.
    pub(crate) const NUMBER_OF_FILES_ANALYZED: i32 = 0x2;
    /// `NumberOfFilesToAnalyze`, read-only integer.
    pub(crate) const NUMBER_OF_FILES_TO_ANALYZE: i32 = 0x3;
    /// `LastStatusMessage`, read-only string.
    pub(crate) const LAST_STATUS_MESSAGE: i32 = 0x4;
    /// `NumberOfSequencesAnalyzed`, read-only integer.
    pub(crate) const NUMBER_OF_SEQUENCES_ANALYZED: i32 = 0x7;
    /// `NumberOfSequencesToAnalyze`, read-only integer.
    pub(crate) const NUMBER_OF_SEQUENCES_TO_ANALYZE: i32 = 0x8;
    /// `UserName`, read-only string.
    pub(crate) const USER_NAME: i32 = 0x9;
    /// `ComputerName`, read-only string.
    pub(crate) const COMPUTER_NAME: i32 = 0xa;
    /// `TestStandVersion`, read-only string.
    pub(crate) const TEST_STAND_VERSION: i32 = 0xb;
    /// `AnalysisStopped`, read-only boolean.
    pub(crate) const ANALYSIS_STOPPED: i32 = 0xc;
    /// `StartTime`, read-only date.
    pub(crate) const START_TIME: i32 = 0xd;
    /// `ElapsedTime`, read-only integer.
    pub(crate) const ELAPSED_TIME: i32 = 0xe;
}

/// Members of `AnalysisMessage`.
pub(crate) mod analysis_message {
    /// `RuleId`, read/write string.
    pub(crate) const RULE_ID: i32 = 0x1;
    /// `Text`, read/write string.
    pub(crate) const TEXT: i32 = 0x2;
    /// `Locations`, read-only.
    pub(crate) const LOCATIONS: i32 = 0x3;
    /// `Severity`, read-only enum.
    pub(crate) const SEVERITY: i32 = 0x4;
    /// `IsCurrent`, read-only boolean.
    pub(crate) const IS_CURRENT: i32 = 0x5;
    /// `Status`, read/write enum.
    pub(crate) const STATUS: i32 = 0x6;
    /// `JustificationForIgnore`, read/write string.
    pub(crate) const JUSTIFICATION_FOR_IGNORE: i32 = 0x7;
}

/// Members of `AnalysisMessages`.
pub(crate) mod analysis_messages {
    /// `Item`, one index parameter.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x2;
    /// `Remove`, one index parameter.
    pub(crate) const REMOVE: i32 = 0x3;
    /// `Clear`, no parameters.
    pub(crate) const CLEAR: i32 = 0x4;
}

/// Members of `Location`.
pub(crate) mod location {
    /// `Kind`, read-only enum.
    pub(crate) const KIND: i32 = 0x1;
    /// `FilePath`, read-only string.
    pub(crate) const FILE_PATH: i32 = 0x2;
    /// `FileDisplayName`, read-only string.
    pub(crate) const FILE_DISPLAY_NAME: i32 = 0x4;
    /// `PropertyPath`, read-only string.
    pub(crate) const PROPERTY_PATH: i32 = 0x5;
    /// `SequenceName`, read-only string.
    pub(crate) const SEQUENCE_NAME: i32 = 0x7;
    /// `StepGroup`, read-only enum.
    pub(crate) const STEP_GROUP: i32 = 0x8;
    /// `StepName`, read-only string.
    pub(crate) const STEP_NAME: i32 = 0x9;
    /// `StepId`, read-only string.
    pub(crate) const STEP_ID: i32 = 0xa;
    /// `StepIndex`, read-only integer.
    pub(crate) const STEP_INDEX: i32 = 0xb;
}

/// Members of `Locations`.
pub(crate) mod locations {
    /// `Item`, one index parameter.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x1;
    /// `Clear`, no parameters.
    pub(crate) const CLEAR: i32 = 0xa;
    /// `GotoLocation`, one index.
    pub(crate) const GOTO_LOCATION: i32 = 0xb;
    /// `ApplicationSite`, read/write integer.
    pub(crate) const APPLICATION_SITE: i32 = 0xd;
    /// `LocationFound`, read/write boolean.
    pub(crate) const LOCATION_FOUND: i32 = 0xe;
    /// `LocationNotFoundMessage`, read/write string.
    pub(crate) const LOCATION_NOT_FOUND_MESSAGE: i32 = 0xf;
    /// `CanDisplayDialog`, read/write boolean.
    pub(crate) const CAN_DISPLAY_DIALOG: i32 = 0x10;
    /// `ToPropertyObject`, no parameters.
    pub(crate) const TO_PROPERTY_OBJECT: i32 = 0x11;
}

/// Members of `WatchExpression`.
pub(crate) mod watch_expression {
    /// `Evaluate`, one context parameter and a reserved integer.
    pub(crate) const EVALUATE: i32 = 0x1;
    /// `ClientSequenceFile`, read/write string.
    pub(crate) const CLIENT_SEQUENCE_FILE: i32 = 0x2;
    /// `SequenceFileScope`, read/write string.
    pub(crate) const SEQUENCE_FILE_SCOPE: i32 = 0x3;
    /// `SequenceScope`, read/write string.
    pub(crate) const SEQUENCE_SCOPE: i32 = 0x4;
    /// `Expression`, read/write string.
    pub(crate) const EXPRESSION: i32 = 0x6;
    /// `NumericFormat`, read/write string.
    pub(crate) const NUMERIC_FORMAT: i32 = 0x7;
    /// `BreakpointType`, read/write enum.
    pub(crate) const BREAKPOINT_TYPE: i32 = 0x8;
    /// `UniqueWatchId`, read-only string.
    pub(crate) const UNIQUE_WATCH_ID: i32 = 0xa;
    /// `BreakpointTriggered`, one context parameter.
    pub(crate) const BREAKPOINT_TRIGGERED: i32 = 0xb;
    /// `UseScopingContext`, read/write boolean.
    pub(crate) const USE_SCOPING_CONTEXT: i32 = 0xd;
    /// `GetScopingContext`, one context parameter.
    pub(crate) const GET_SCOPING_CONTEXT: i32 = 0xe;
}

/// Members of `SearchResults`.
pub(crate) mod search_results {
    /// `Cancel`, two boolean parameters.
    pub(crate) const CANCEL: i32 = 0x1;
    /// `IsComplete`, two boolean parameters.
    pub(crate) const IS_COMPLETE: i32 = 0x2;
    /// `NumMatches`, read-only integer.
    pub(crate) const NUM_MATCHES: i32 = 0x3;
    /// `GetMatch`, one index parameter.
    pub(crate) const GET_MATCH: i32 = 0x4;
    /// `StatusMessage`, read-only string.
    pub(crate) const STATUS_MESSAGE: i32 = 0x5;
    /// `NumWarnings`, read-only integer.
    pub(crate) const NUM_WARNINGS: i32 = 0x6;
    /// `GetWarning`, one index parameter.
    pub(crate) const GET_WARNING: i32 = 0x7;
}

/// Members of `SearchMatch`.
pub(crate) mod search_match {
    /// `GetLocation`, three integer outputs.
    pub(crate) const GET_LOCATION: i32 = 0x1;
    /// `GetPropertyPath`, one boolean parameter.
    pub(crate) const GET_PROPERTY_PATH: i32 = 0x2;
    /// `MatchedText`, read-only string.
    pub(crate) const MATCHED_TEXT: i32 = 0x3;
    /// `FileDisplayName`, read-only string.
    pub(crate) const FILE_DISPLAY_NAME: i32 = 0x4;
    /// `FilePath`, read-only string.
    pub(crate) const FILE_PATH: i32 = 0x5;
    /// `FileId`, read-only integer.
    pub(crate) const FILE_ID: i32 = 0x6;
    /// `UpdateForReplace`, one string parameter.
    pub(crate) const UPDATE_FOR_REPLACE: i32 = 0x7;
    /// `TypeCategoryOfMatch`, read-only integer.
    pub(crate) const TYPE_CATEGORY_OF_MATCH: i32 = 0x8;
    /// `UserData`, read/write integer.
    pub(crate) const USER_DATA: i32 = 0x9;
    /// `PropertyValueAsString`, read-only string.
    pub(crate) const PROPERTY_VALUE_AS_STRING: i32 = 0xa;
    /// `PropertyValueType`, read-only integer.
    pub(crate) const PROPERTY_VALUE_TYPE: i32 = 0xb;
    /// `UpdateForReplaceValue`, one string parameter.
    pub(crate) const UPDATE_FOR_REPLACE_VALUE: i32 = 0xc;
    /// `MatchIsValid`, read-only boolean.
    pub(crate) const MATCH_IS_VALID: i32 = 0xd;
}

/// Members of `Report`.
pub(crate) mod report {
    /// `Append`, one string parameter, returns an integer.
    pub(crate) const APPEND: i32 = 0x1;
    /// `Reset`, one string parameter, returns an integer.
    pub(crate) const RESET: i32 = 0x2;
    /// `GetSection`, oldest and latest indices.
    pub(crate) const GET_SECTION: i32 = 0x3;
    /// `Save`, path, append flag, and conversion.
    pub(crate) const SAVE: i32 = 0x5;
    /// `Load`, path and conversion.
    pub(crate) const LOAD: i32 = 0x6;
    /// `GetTempFile`, conversion and optional extension.
    pub(crate) const GET_TEMP_FILE: i32 = 0x8;
    /// `LaunchViewer`, one linefeed-conversion parameter.
    pub(crate) const LAUNCH_VIEWER: i32 = 0x7;
    /// `NewReportSection`, no parameters.
    pub(crate) const NEW_REPORT_SECTION: i32 = 0x29;
    /// `ReportSection`, read/write object.
    pub(crate) const REPORT_SECTION: i32 = 0x28;
    /// `AsPropertyObject`, no parameters.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0x1d;
    /// `LatestAppendIndex`, read-only integer.
    pub(crate) const LATEST_APPEND_INDEX: i32 = 0x1e;
    /// `ResetCount`, read-only integer.
    pub(crate) const RESET_COUNT: i32 = 0x1f;
    /// `All`, read-only string.
    pub(crate) const ALL: i32 = 0x20;
    /// `Id`, read-only integer.
    pub(crate) const ID: i32 = 0x21;
    /// `Location`, read/write string.
    pub(crate) const LOCATION: i32 = 0x22;
    /// `Format`, read/write string.
    pub(crate) const FORMAT: i32 = 0x23;
    /// `IsEmpty`, read-only boolean.
    pub(crate) const IS_EMPTY: i32 = 0x24;
    /// `StyleSheetPath`, read/write string.
    pub(crate) const STYLE_SHEET_PATH: i32 = 0x25;
    /// `TempFileDirectory`, read/write string.
    pub(crate) const TEMP_FILE_DIRECTORY: i32 = 0x26;
    /// `SuspendReportRefresh`, read/write boolean.
    pub(crate) const SUSPEND_REPORT_REFRESH: i32 = 0x2a;
}

/// Members of `Batch`.
pub(crate) mod batch {
    /// `AddThread`, engine, thread, and order number.
    pub(crate) const ADD_THREAD: i32 = 0x1;
    /// `RemoveThread`, engine and thread.
    pub(crate) const REMOVE_THREAD: i32 = 0x2;
    /// `EnterSynchronizedSection`, section, sync type, timeout, context, etc.
    pub(crate) const ENTER_SYNCHRONIZED_SECTION: i32 = 0x3;
    /// `ExitSynchronizedSection`, section, timeout, context, and message flag.
    pub(crate) const EXIT_SYNCHRONIZED_SECTION: i32 = 0x4;
    /// `Name`, read-only string.
    pub(crate) const NAME: i32 = 0x6;
    /// `DefaultBatchSynchronization`, read/write enum.
    pub(crate) const DEFAULT_BATCH_SYNCHRONIZATION: i32 = 0x7;
    /// `GetInfo`, two integer outputs.
    pub(crate) const GET_INFO: i32 = 0x8;
    /// `GetBatchThreads`, no parameters, returns an object array.
    pub(crate) const GET_BATCH_THREADS: i32 = 0x9;
    /// `ReportRTEOccurred`, one context.
    pub(crate) const REPORT_RTE_OCCURRED: i32 = 0xa;
}

/// Members of `SequenceFileDiffer`.
pub(crate) mod sequence_file_differ {
    /// `DifferencesRoot`, read-only.
    pub(crate) const DIFFERENCES_ROOT: i32 = 0x1;
    /// `UpdateDifferences`, one section parameter, returns an integer.
    pub(crate) const UPDATE_DIFFERENCES: i32 = 0x2;
    /// `DiffSequenceFiles`, no parameters.
    pub(crate) const DIFF_SEQUENCE_FILES: i32 = 0x3;
}

/// Members of `SequenceFileDifferenceSection`.
pub(crate) mod sequence_file_difference_section {
    /// `Name`, read-only string.
    pub(crate) const NAME: i32 = 0x1;
    /// `SmallIconIndex`, read-only integer.
    pub(crate) const SMALL_ICON_INDEX: i32 = 0x2;
    /// `File1Section`, read-only property object.
    pub(crate) const FILE1_SECTION: i32 = 0x3;
    /// `File2Section`, read-only property object.
    pub(crate) const FILE2_SECTION: i32 = 0x4;
    /// `File1Location`, read-only string.
    pub(crate) const FILE1_LOCATION: i32 = 0x5;
    /// `File2Location`, read-only string.
    pub(crate) const FILE2_LOCATION: i32 = 0x6;
    /// `File1Differences`, read-only integer array.
    pub(crate) const FILE1_DIFFERENCES: i32 = 0x7;
    /// `File2Differences`, read-only integer array.
    pub(crate) const FILE2_DIFFERENCES: i32 = 0x8;
    /// `NumberOfDifferenceGroups`, read-only integer.
    pub(crate) const NUMBER_OF_DIFFERENCE_GROUPS: i32 = 0x9;
    /// `GetDifferenceGroup`, one index and two integer-array outputs.
    pub(crate) const GET_DIFFERENCE_GROUP: i32 = 0xa;
    /// `NumberOfDifferences`, read-only integer.
    pub(crate) const NUMBER_OF_DIFFERENCES: i32 = 0xb;
    /// `TotalNumberOfDifferences`, read-only integer.
    pub(crate) const TOTAL_NUMBER_OF_DIFFERENCES: i32 = 0xc;
    /// `SubSections`, read-only collection.
    pub(crate) const SUB_SECTIONS: i32 = 0xd;
}

/// Members of `SequenceFileDifferenceSections`.
pub(crate) mod sequence_file_difference_sections {
    /// `Item`, one index parameter.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x2;
}

/// Members of `SyncManager`.
pub(crate) mod sync_manager {
    /// `CreateSemaphore`, name, initial count, max count, and an existing flag.
    pub(crate) const CREATE_SEMAPHORE: i32 = 0x1;
    /// `GetSemaphore`, one name.
    pub(crate) const GET_SEMAPHORE: i32 = 0x2;
    /// `CreateMutex`, name and an existing flag.
    pub(crate) const CREATE_MUTEX: i32 = 0x3;
    /// `GetMutex`, one name.
    pub(crate) const GET_MUTEX: i32 = 0x4;
    /// `CreateRendezvous`, name, count, and an existing flag.
    pub(crate) const CREATE_RENDEZVOUS: i32 = 0x5;
    /// `GetRendezvous`, one name.
    pub(crate) const GET_RENDEZVOUS: i32 = 0x6;
    /// `CreateQueue`, name, size limit, and an existing flag.
    pub(crate) const CREATE_QUEUE: i32 = 0x7;
    /// `GetQueue`, one name.
    pub(crate) const GET_QUEUE: i32 = 0x8;
    /// `CreateNotification`, name and an existing flag.
    pub(crate) const CREATE_NOTIFICATION: i32 = 0x9;
    /// `GetNotification`, one name.
    pub(crate) const GET_NOTIFICATION: i32 = 0xa;
    /// `CreateBatch`, name and an existing flag.
    pub(crate) const CREATE_BATCH: i32 = 0xc;
    /// `GetBatch`, one name.
    pub(crate) const GET_BATCH: i32 = 0xd;
    /// `GetSyncObject`, name and kind.
    pub(crate) const GET_SYNC_OBJECT: i32 = 0xe;
    /// `IsAlive`, no parameters.
    pub(crate) const IS_ALIVE: i32 = 0xf;
    /// `MachineID`, read-only string.
    pub(crate) const MACHINE_ID: i32 = 0x14;
    /// `ProcessID`, read-only integer.
    pub(crate) const PROCESS_ID: i32 = 0x15;
    /// `LoggingEnabled`, read-only boolean.
    pub(crate) const LOGGING_ENABLED: i32 = 0x16;
    /// `LoggingOutputMessageCategoryName`, read-only string.
    pub(crate) const LOGGING_OUTPUT_MESSAGE_CATEGORY_NAME: i32 = 0x18;
    /// `BeginLogging`, no parameters.
    pub(crate) const BEGIN_LOGGING: i32 = 0x19;
    /// `EndLogging`, no parameters.
    pub(crate) const END_LOGGING: i32 = 0x1a;
    /// `Sleep`, one seconds parameter.
    pub(crate) const SLEEP: i32 = 0x1e;
}

/// Members of `Semaphore`.
pub(crate) mod semaphore {
    /// `AcquireSemaphore`, timeout, context, message flag, and result output.
    pub(crate) const ACQUIRE_SEMAPHORE: i32 = 0x1;
    /// `AcquireSemaphoreWithAutoReleaser`, same shape, returns an
    /// `AutoReleaser`.
    pub(crate) const ACQUIRE_SEMAPHORE_WITH_AUTO_RELEASER: i32 = 0x2;
    /// `ReleaseSemaphore`, no parameters.
    pub(crate) const RELEASE_SEMAPHORE: i32 = 0x3;
    /// `GetInfo`, four integer outputs.
    pub(crate) const GET_INFO: i32 = 0x4;
    /// `Name`, read-only string.
    pub(crate) const NAME: i32 = 0x5;
    /// `ReleaseSemaphoreEx`, context and thread id.
    pub(crate) const RELEASE_SEMAPHORE_EX: i32 = 0x6;
}

/// Members of `Mutex`.
pub(crate) mod mutex {
    /// `GetInfo`, one integer output.
    pub(crate) const GET_INFO: i32 = 0x1;
    /// `EarlyUnlockMutex`, thread id.
    pub(crate) const EARLY_UNLOCK_MUTEX: i32 = 0x2;
    /// `LockMutex`, seven inputs and two outputs.
    pub(crate) const LOCK_MUTEX: i32 = 0x3;
    /// `Name`, read-only string.
    pub(crate) const NAME: i32 = 0x4;
    /// `LockMutexGroup`, mutex array and two outputs.
    pub(crate) const LOCK_MUTEX_GROUP: i32 = 0x5;
    /// `GetSavedOwnerInfo`, three outputs.
    pub(crate) const GET_SAVED_OWNER_INFO: i32 = 0x6;
    /// `EarlyUnlockMutexEx`, context and thread id.
    pub(crate) const EARLY_UNLOCK_MUTEX_EX: i32 = 0x9;
}

/// Members of `Notification`.
pub(crate) mod notification {
    /// `Set`, data object, by-reference flag, and auto-clear flag.
    pub(crate) const SET: i32 = 0x1;
    /// `Clear`, no parameters.
    pub(crate) const CLEAR: i32 = 0x2;
    /// `Pulse`, data object, by-reference flag, and all-waiters flag.
    pub(crate) const PULSE: i32 = 0x3;
    /// `Wait`, destination, timeout, context, message flag, result output.
    pub(crate) const WAIT: i32 = 0x4;
    /// `GetInfo`, context, destination, and three outputs.
    pub(crate) const GET_INFO: i32 = 0x5;
    /// `Name`, read-only string.
    pub(crate) const NAME: i32 = 0x6;
    /// `WaitMultiple`, notification array and three outputs.
    pub(crate) const WAIT_MULTIPLE: i32 = 0x7;
    /// `SetEx`, context plus the three `Set` parameters.
    pub(crate) const SET_EX: i32 = 0xb;
    /// `ClearEx`, context.
    pub(crate) const CLEAR_EX: i32 = 0xc;
    /// `PulseEx`, context plus the three `Pulse` parameters.
    pub(crate) const PULSE_EX: i32 = 0xd;
}

/// Members of `Queue`.
pub(crate) mod queue {
    /// `Enqueue`, seven inputs and two outputs.
    pub(crate) const ENQUEUE: i32 = 0x1;
    /// `Dequeue`, six inputs and one output.
    pub(crate) const DEQUEUE: i32 = 0x2;
    /// `Flush`, context and destination array.
    pub(crate) const FLUSH: i32 = 0x3;
    /// `GetInfo`, context, destination, and four outputs.
    pub(crate) const GET_INFO: i32 = 0x4;
    /// `DequeueMultiple`, queue array and three outputs.
    pub(crate) const DEQUEUE_MULTIPLE: i32 = 0x5;
    /// `Name`, read-only string.
    pub(crate) const NAME: i32 = 0x9;
}

/// Members of `Rendezvous`.
pub(crate) mod rendezvous {
    /// `Rendezvous`, timeout, context, message flag, and result output.
    pub(crate) const RENDEZVOUS: i32 = 0x1;
    /// `GetInfo`, two integer outputs.
    pub(crate) const GET_INFO: i32 = 0x2;
    /// `Name`, read-only string.
    pub(crate) const NAME: i32 = 0x3;
}

/// Members of `AutoReleaser`.
pub(crate) mod auto_releaser {
    /// `EarlyRelease`, no parameters.
    pub(crate) const EARLY_RELEASE: i32 = 0x1;
    /// `ReleasePending`, no parameters.
    pub(crate) const RELEASE_PENDING: i32 = 0x2;
    /// `EarlyReleaseEx`, one context.
    pub(crate) const EARLY_RELEASE_EX: i32 = 0x3;
    /// `SetReleaserThreadId`, one thread id.
    pub(crate) const SET_RELEASER_THREAD_ID: i32 = 0x4;
}

/// Members of `Reports`.
pub(crate) mod reports {
    /// `Item`, one index.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x1;
    /// `Insert`, optional index.
    pub(crate) const INSERT: i32 = 0xa;
    /// `Remove`, one index.
    pub(crate) const REMOVE: i32 = 0xb;
    /// `Clear`, no parameters.
    pub(crate) const CLEAR: i32 = 0xc;
    /// `InsertExisting`, report and optional index.
    pub(crate) const INSERT_EXISTING: i32 = 0xd;
    /// `Replace`, old and new report.
    pub(crate) const REPLACE: i32 = 0xe;
    /// `ActiveReport`, read-only via property get.
    pub(crate) const ACTIVE_REPORT: i32 = 0xf;
}

/// Members of `ReportSection`.
pub(crate) mod report_section {
    /// `ReportSubsections`, read/write collection.
    pub(crate) const REPORT_SUBSECTIONS: i32 = 0x1;
    /// `Body`, read/write string.
    pub(crate) const BODY: i32 = 0x2;
    /// `IsEmpty`, read-only boolean.
    pub(crate) const IS_EMPTY: i32 = 0x3;
    /// `Id`, read-only integer.
    pub(crate) const ID: i32 = 0x4;
    /// `Header`, read/write string.
    pub(crate) const HEADER: i32 = 0x5;
    /// `Footer`, read/write string.
    pub(crate) const FOOTER: i32 = 0x6;
    /// `GetAllText`, no parameters.
    pub(crate) const GET_ALL_TEXT: i32 = 0x32;
}

/// Members of `ReportSections`.
pub(crate) mod report_sections {
    /// `Item`, one index.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x1;
    /// `Insert`, index, header, body, footer.
    pub(crate) const INSERT: i32 = 0xa;
    /// `Remove`, one index.
    pub(crate) const REMOVE: i32 = 0xb;
    /// `Clear`, no parameters.
    pub(crate) const CLEAR: i32 = 0xc;
    /// `InsertExisting`, section and index.
    pub(crate) const INSERT_EXISTING: i32 = 0xd;
}

/// Members of `OutputRecordStream` (the base Data Streams interface).
pub(crate) mod output_record_stream {
    /// `WriteRecord`, an object and an optional mapping string.
    pub(crate) const WRITE_RECORD: i32 = 0x1;
    /// `WriteRecordFrom`, a context object and a string array.
    pub(crate) const WRITE_RECORD_FROM: i32 = 0x2;
    /// `Flush`, no parameters.
    pub(crate) const FLUSH: i32 = 0x3;
    /// `Close`, no parameters.
    pub(crate) const CLOSE: i32 = 0x4;
    /// `GetStreamInfo`, one optional string output.
    pub(crate) const GET_STREAM_INFO: i32 = 0x5;
}

/// Members of `CsvFileInputRecordStream`.
pub(crate) mod csv_file_input_record_stream {
    /// `Open`, one path.
    pub(crate) const OPEN: i32 = 0x12d;
    /// `Path`, read-only string.
    pub(crate) const PATH: i32 = 0x12e;
    /// `ReadLine`, one text output.
    pub(crate) const READ_LINE: i32 = 0x12f;
    /// `SeparatorChar`, read/write string.
    pub(crate) const SEPARATOR_CHAR: i32 = 0x130;
    /// `ScanForTag`, tag and ignore-case flag.
    pub(crate) const SCAN_FOR_TAG: i32 = 0x131;
    /// `SkipLines`, one count.
    pub(crate) const SKIP_LINES: i32 = 0x132;
}

/// Members of `CsvFileOutputRecordStream`.
pub(crate) mod csv_file_output_record_stream {
    /// `Open`, path and open mode.
    pub(crate) const OPEN: i32 = 0xc9;
    /// `Path`, read-only string.
    pub(crate) const PATH: i32 = 0xca;
    /// `WriteFieldHeaders`, prototype and mapping.
    pub(crate) const WRITE_FIELD_HEADERS: i32 = 0xcb;
    /// `SeparatorChar`, read/write string.
    pub(crate) const SEPARATOR_CHAR: i32 = 0xcc;
    /// `WriteLine`, one string.
    pub(crate) const WRITE_LINE: i32 = 0xcd;
    /// `AutoFlush`, read/write boolean.
    pub(crate) const AUTO_FLUSH: i32 = 0xce;
    /// `WriteRecordPrototype`, prototype and mapping.
    pub(crate) const WRITE_RECORD_PROTOTYPE: i32 = 0xd2;
}

/// Members of `RuleModule`.
pub(crate) mod rule_module {
    /// `Scope`, read-only object.
    pub(crate) const SCOPE: i32 = 0x1;
    /// `Module`, read-only object.
    pub(crate) const MODULE: i32 = 0x2;
    /// `IsBuiltin`, read/write boolean.
    pub(crate) const IS_BUILTIN: i32 = 0x3;
    /// `Clone`, no parameters.
    pub(crate) const CLONE: i32 = 0x4;
}

/// Members of `AnalyzerModule`.
pub(crate) mod analyzer_module {
    /// `Adapter`, read/write integer.
    pub(crate) const ADAPTER: i32 = 0x1;
    /// `Path`, read/write string.
    pub(crate) const PATH: i32 = 0x2;
    /// `Function`, read/write string.
    pub(crate) const FUNCTION: i32 = 0x3;
    /// `VerifyPrototype`, one optional string output.
    pub(crate) const VERIFY_PROTOTYPE: i32 = 0x4;
    /// `Call`, one object argument.
    pub(crate) const CALL: i32 = 0x5;
    /// `Description`, read-only string.
    pub(crate) const DESCRIPTION: i32 = 0x6;
    /// `AbsolutePath`, read-only string.
    pub(crate) const ABSOLUTE_PATH: i32 = 0x7;
    /// `Type`, read/write integer.
    pub(crate) const TYPE: i32 = 0x8;
    /// `RunInLabVIEWRunTimeEngine`, read/write boolean.
    pub(crate) const RUN_IN_LABVIEW_RUN_TIME_ENGINE: i32 = 0x9;
    /// `ClassName`, read/write string.
    pub(crate) const CLASS_NAME: i32 = 0xa;
    /// `ModuleName`, read/write string.
    pub(crate) const MODULE_NAME: i32 = 0xb;
    /// `AllowDotNetModuleUnload`, read/write boolean (2026).
    pub(crate) const ALLOW_DOTNET_MODULE_UNLOAD: i32 = 0xd;
}

/// Members of `RuleModuleScope`.
pub(crate) mod rule_module_scope {
    /// `NamedTypes`, string array.
    pub(crate) const NAMED_TYPES: i32 = 0x1;
    /// `ArrayOfNamedTypes`, string array.
    pub(crate) const ARRAY_OF_NAMED_TYPES: i32 = 0x2;
    /// `ObjectKinds`, integer flags.
    pub(crate) const OBJECT_KINDS: i32 = 0x3;
    /// `Transitions`, integer array.
    pub(crate) const TRANSITIONS: i32 = 0x4;
    /// `Adapters`, string array.
    pub(crate) const ADAPTERS: i32 = 0x5;
    /// `PropertyValueTypeFlags`, integer flags.
    pub(crate) const PROPERTY_VALUE_TYPE_FLAGS: i32 = 0x6;
}

/// Members of `RuleModules`.
pub(crate) mod rule_modules {
    /// `Item`, one index.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x2;
    /// `Insert`, index and module.
    pub(crate) const INSERT: i32 = 0x3;
    /// `Remove`, one index.
    pub(crate) const REMOVE: i32 = 0x4;
    /// `Clear`, no parameters.
    pub(crate) const CLEAR: i32 = 0x5;
}

/// Members of `Rule`.
pub(crate) mod rule {
    /// `Id`, read/write string.
    pub(crate) const ID: i32 = 0x1;
    /// `Name`, read/write string.
    pub(crate) const NAME: i32 = 0x2;
    /// `Description`, read/write string.
    pub(crate) const DESCRIPTION: i32 = 0x3;
    /// `Category`, read/write string.
    pub(crate) const CATEGORY: i32 = 0x4;
    /// `StringResourceCategory`, read/write string.
    pub(crate) const STRING_RESOURCE_CATEGORY: i32 = 0x5;
    /// `Severity`, read/write enum.
    pub(crate) const SEVERITY: i32 = 0x6;
    /// `IsBuiltin`, read/write boolean.
    pub(crate) const IS_BUILTIN: i32 = 0x9;
    /// `RuleSettingDefinitions`, read-only collection.
    pub(crate) const RULE_SETTING_DEFINITIONS: i32 = 0xb;
    /// `Clone`, no parameters.
    pub(crate) const CLONE: i32 = 0xe;
    /// `DisplayName`, read-only string.
    pub(crate) const DISPLAY_NAME: i32 = 0xf;
    /// `GetDisplayDescription`, one parameter.
    pub(crate) const GET_DISPLAY_DESCRIPTION: i32 = 0x10;
}

/// Members of `Rules`.
pub(crate) mod rules {
    /// `Item`, one index.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x2;
    /// `Insert`, index and rule.
    pub(crate) const INSERT: i32 = 0x3;
    /// `Remove`, one index.
    pub(crate) const REMOVE: i32 = 0x4;
    /// `Clear`, no parameters.
    pub(crate) const CLEAR: i32 = 0x5;
}

/// Members of `RuleSet`.
pub(crate) mod rule_set {
    /// `Name`, read/write string.
    pub(crate) const NAME: i32 = 0x1;
    /// `GetRuleConfiguration`, one rule-id parameter.
    pub(crate) const GET_RULE_CONFIGURATION: i32 = 0x2;
    /// `ConformToRules`, no parameters, returns a string array.
    pub(crate) const CONFORM_TO_RULES: i32 = 0x3;
    /// `Clone`, no parameters.
    pub(crate) const CLONE: i32 = 0x4;
    /// `RenameRuleId`, old and new ids.
    pub(crate) const RENAME_RULE_ID: i32 = 0x5;
}

/// Members of `RuleSets`.
pub(crate) mod rule_sets {
    /// `Item`, one index.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x2;
    /// `Insert`, index and rule set.
    pub(crate) const INSERT: i32 = 0x3;
    /// `Remove`, one index.
    pub(crate) const REMOVE: i32 = 0x4;
    /// `Clear`, no parameters.
    pub(crate) const CLEAR: i32 = 0x5;
}

/// Members of `RuleConfiguration`.
pub(crate) mod rule_configuration {
    /// `RuleId`, read-only string.
    pub(crate) const RULE_ID: i32 = 0x1;
    /// `Enabled`, read/write boolean.
    pub(crate) const ENABLED: i32 = 0x2;
    /// `ConfigurationData`, read-only property object.
    pub(crate) const CONFIGURATION_DATA: i32 = 0x3;
    /// `RuleSettingValues`, read-only collection.
    pub(crate) const RULE_SETTING_VALUES: i32 = 0x4;
    /// `Severity`, read/write enum.
    pub(crate) const SEVERITY: i32 = 0x5;
    /// `Description`, read/write string.
    pub(crate) const DESCRIPTION: i32 = 0x6;
}

/// Members of `OutputMessage`.
pub(crate) mod output_message {
    /// `Category`, read/write string.
    pub(crate) const CATEGORY: i32 = 0x1;
    /// `Severity`, read/write enum.
    pub(crate) const SEVERITY: i32 = 0x2;
    /// `Message`, read/write string.
    pub(crate) const MESSAGE: i32 = 0x3;
    /// `TextColor`, read/write integer.
    pub(crate) const TEXT_COLOR: i32 = 0x4;
    /// `IconName`, read/write string.
    pub(crate) const ICON_NAME: i32 = 0x6;
    /// `TimeInSeconds`, read/write double.
    pub(crate) const TIME_IN_SECONDS: i32 = 0xa;
    /// `Id`, read-only integer.
    pub(crate) const ID: i32 = 0xb;
    /// `Post`, no parameters.
    pub(crate) const POST: i32 = 0x8;
    /// `AsPropertyObject`, no parameters.
    pub(crate) const AS_PROPERTY_OBJECT: i32 = 0xe;
}

/// Members of `OutputMessages`.
pub(crate) mod output_messages {
    /// `Item`, one index.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x2;
    /// `Add`, one message.
    pub(crate) const ADD: i32 = 0x3;
    /// `Remove`, one index.
    pub(crate) const REMOVE: i32 = 0x4;
    /// `Clear`, no parameters.
    pub(crate) const CLEAR: i32 = 0x5;
    /// `FindIndex`, one message.
    pub(crate) const FIND_INDEX: i32 = 0x8;
    /// `ToPropertyObject`, one parameter.
    pub(crate) const TO_PROPERTY_OBJECT: i32 = 0x9;
    /// `FromPropertyObject`, one parameter.
    pub(crate) const FROM_PROPERTY_OBJECT: i32 = 0xa;
}

/// Members of `RuleSettingValues`.
pub(crate) mod rule_setting_values {
    /// `Item`, one index or name.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x2;
    /// `GetBooleanValue`, one setting name.
    pub(crate) const GET_BOOLEAN_VALUE: i32 = 0x3;
    /// `GetStringValue`, one setting name.
    pub(crate) const GET_STRING_VALUE: i32 = 0x4;
    /// `GetNumberValue`, one setting name.
    pub(crate) const GET_NUMBER_VALUE: i32 = 0x5;
}

/// Members of `IUndoItem`.
pub(crate) mod undo_item {
    /// `EditedFile`, read-only object.
    pub(crate) const EDITED_FILE: i32 = 0x1;
    /// `EditedObjects`, read-only variant array.
    pub(crate) const EDITED_OBJECTS: i32 = 0x2;
    /// `UndoDescription`, read-only string.
    pub(crate) const UNDO_DESCRIPTION: i32 = 0x3;
    /// `RedoDescription`, read-only string.
    pub(crate) const REDO_DESCRIPTION: i32 = 0x4;
    /// `CanUndo`, read-only boolean.
    pub(crate) const CAN_UNDO: i32 = 0x5;
    /// `CanRedo`, read-only boolean.
    pub(crate) const CAN_REDO: i32 = 0x6;
    /// `UndoEditKind`, read-only integer.
    pub(crate) const UNDO_EDIT_KIND: i32 = 0x7;
    /// `RedoEditKind`, read-only integer.
    pub(crate) const REDO_EDIT_KIND: i32 = 0x8;
    /// `UndoStack`, read/write object.
    pub(crate) const UNDO_STACK: i32 = 0x9;
    /// `RefreshEnabled`, read/write boolean.
    pub(crate) const REFRESH_ENABLED: i32 = 0xa;
    /// `PreEditLocations`, read-only object.
    pub(crate) const PRE_EDIT_LOCATIONS: i32 = 0xb;
    /// `PostEditLocations`, read-only object.
    pub(crate) const POST_EDIT_LOCATIONS: i32 = 0xc;
    /// `PreEditChangeCount`, read-only integer.
    pub(crate) const PRE_EDIT_CHANGE_COUNT: i32 = 0xd;
    /// `PostEditChangeCount`, read-only integer.
    pub(crate) const POST_EDIT_CHANGE_COUNT: i32 = 0xe;
    /// `Undo`, no parameters.
    pub(crate) const UNDO: i32 = 0xf;
    /// `Redo`, no parameters.
    pub(crate) const REDO: i32 = 0x10;
    /// `TopObjects`, read-only variant array (`UndoItem2`).
    pub(crate) const TOP_OBJECTS: i32 = 0x14;
    /// `LookupStrings`, read-only string array (`UndoItem2`).
    pub(crate) const LOOKUP_STRINGS: i32 = 0x15;
}

/// Members of `UndoItemCreator`.
pub(crate) mod undo_item_creator {
    /// `HaveEditsBeenMade`, read-only boolean.
    pub(crate) const HAVE_EDITS_BEEN_MADE: i32 = 0x1;
    /// `EndEdit`, no parameters.
    pub(crate) const END_EDIT: i32 = 0x3;
    /// `BeginBatchEdit`, one object array.
    pub(crate) const BEGIN_BATCH_EDIT: i32 = 0x4;
    /// `EndBatchEdit`, no parameters.
    pub(crate) const END_BATCH_EDIT: i32 = 0x5;
    /// `CreateAndPostUndoItem`, options and optional locations.
    pub(crate) const CREATE_AND_POST_UNDO_ITEM: i32 = 0x6;
    /// `BeginEditEx`, object and lookup string.
    pub(crate) const BEGIN_EDIT_EX: i32 = 0x7;
}

/// Members of `UndoItems`.
pub(crate) mod undo_items {
    /// `Item`, one index.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Remove`, one index.
    pub(crate) const REMOVE: i32 = 0x1;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x2;
    /// `TopItem`, read-only object.
    pub(crate) const TOP_ITEM: i32 = 0x3;
}

/// Members of `UndoStack`.
pub(crate) mod undo_stack {
    /// `GetUndoDescription`, one accelerator prefix.
    pub(crate) const GET_UNDO_DESCRIPTION: i32 = 0x1;
    /// `GetRedoDescription`, one accelerator prefix.
    pub(crate) const GET_REDO_DESCRIPTION: i32 = 0x2;
    /// `CanUndo`, read-only boolean.
    pub(crate) const CAN_UNDO: i32 = 0x3;
    /// `CanRedo`, read-only boolean.
    pub(crate) const CAN_REDO: i32 = 0x4;
    /// `Undo`, no parameters.
    pub(crate) const UNDO: i32 = 0x5;
    /// `Redo`, no parameters.
    pub(crate) const REDO: i32 = 0x6;
    /// `Push`, one undo item.
    pub(crate) const PUSH: i32 = 0x7;
    /// `Clear`, no parameters.
    pub(crate) const CLEAR: i32 = 0x8;
    /// `AggregateTopUndoItems`, count and optional locations.
    pub(crate) const AGGREGATE_TOP_UNDO_ITEMS: i32 = 0x9;
    /// `UndoItems`, read-only collection.
    pub(crate) const UNDO_ITEMS: i32 = 0xa;
    /// `RedoItems`, read-only collection.
    pub(crate) const REDO_ITEMS: i32 = 0xb;
}

/// Members of `WatchExpressions`.
pub(crate) mod watch_expressions {
    /// `Item`, one index parameter.
    pub(crate) const ITEM: i32 = 0x0;
    /// `Count`, read-only integer.
    pub(crate) const COUNT: i32 = 0x2;
    /// `Insert`, one position and two optional parameters.
    pub(crate) const INSERT: i32 = 0x3;
    /// `Remove`, one watch-expression parameter.
    pub(crate) const REMOVE: i32 = 0x4;
    /// `Clear`, no parameters.
    pub(crate) const CLEAR: i32 = 0x5;
    /// `Clone`, one optional boolean.
    pub(crate) const CLONE: i32 = 0x6;
}
