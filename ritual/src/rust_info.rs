//! Types holding information about generates Rust API.

use crate::cpp_data::{CppPath, CppTypeDoc};
use crate::cpp_ffi_data::CppFfiFunction;
use crate::cpp_function::CppFunctionExternalDoc;
use crate::cpp_type::CppType;
use crate::rust_code_generator::rust_type_to_code;
use crate::rust_type::{RustFinalType, RustPath, RustPointerLikeTypeKind, RustType};
use ritual_common::errors::{bail, format_err, Result};
use ritual_common::string_utils::ends_with_digit;
use serde_derive::{Deserialize, Serialize};

/// One variant of a Rust enum
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustEnumValue {
    pub path: RustPath,
    /// Corresponding value
    pub value: i64,
    /// Documentation of corresponding C++ variants
    pub doc: RustEnumValueDoc,
    pub cpp_item_index: usize,
}

/// C++ documentation data for a enum variant
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustEnumValueDoc {
    pub extra_doc: Option<String>,
    /// C++ path of the variant
    pub cpp_path: CppPath,
    /// HTML content
    pub cpp_doc: Option<String>,
}

/// Information about a Qt slot wrapper on Rust side
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustQtSlotWrapper {
    /// Argument types of the slot
    pub arguments: Vec<RustFinalType>,
    pub signal_arguments: Vec<CppType>,
    pub raw_slot_wrapper: RustPath,
    pub cpp_item_index: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum RustWrapperTypeKind {
    EnumWrapper,
    ImmovableClassWrapper,
    MovableClassWrapper { sized_type_path: RustPath },
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustRawQtSlotWrapperDocData {
    pub public_wrapper_path: RustPath,
    pub rust_arguments: Vec<RustFinalType>,
    pub cpp_arguments: Vec<CppType>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustWrapperTypeDocData {
    /// Corresponding C++ type (for generating docs).
    pub cpp_path: CppPath,
    /// C++ documentation for this type
    pub cpp_doc: Option<CppTypeDoc>,

    pub raw_qt_slot_wrapper: Option<RustRawQtSlotWrapperDocData>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustWrapperType {
    pub doc_data: RustWrapperTypeDocData,
    pub kind: RustWrapperTypeKind,
    pub cpp_item_index: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustFfiClassTypeDoc {
    pub cpp_path: CppPath,
    pub public_rust_path: RustPath,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustSizedType {
    pub cpp_path: CppPath,
    pub cpp_item_index: usize,
}

/// Information about a Rust type wrapper
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum RustStructKind {
    WrapperType(RustWrapperType),
    QtSlotWrapper(RustQtSlotWrapper),
    SizedType(RustSizedType),
}

impl RustStructKind {
    pub fn is_wrapper_type(&self) -> bool {
        if let RustStructKind::WrapperType(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_sized_type(&self) -> bool {
        match *self {
            RustStructKind::SizedType(_) => true,
            _ => false,
        }
    }

    pub fn has_same_source(&self, other: &Self) -> bool {
        match self {
            RustStructKind::WrapperType(data) => {
                if let RustStructKind::WrapperType(other) = other {
                    data.cpp_item_index == other.cpp_item_index
                } else {
                    false
                }
            }
            RustStructKind::QtSlotWrapper(data) => {
                if let RustStructKind::QtSlotWrapper(other) = other {
                    data.cpp_item_index == other.cpp_item_index
                } else {
                    false
                }
            }
            RustStructKind::SizedType(data) => {
                if let RustStructKind::SizedType(other) = other {
                    data.cpp_item_index == other.cpp_item_index
                } else {
                    false
                }
            }
        }
    }
}

/// Exported information about a Rust wrapper type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustStruct {
    /// Additional documentation content that will appear before C++ documentation or any other
    /// automatically generated content.
    pub extra_doc: Option<String>,
    pub path: RustPath,
    /// Kind of the type and additional information.
    pub kind: RustStructKind,
    /// Indicates whether this type is public
    pub is_public: bool,
}

/// Location of a Rust method.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum RustFunctionScope {
    /// Inside `impl T {}`, where `T` is `target_type`.
    Impl { target_type: RustType },
    /// Inside a trait implementation.
    TraitImpl,
    /// A free function.
    Free,
}

/// Information about a Rust method argument.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustFunctionArgument {
    /// C++ and Rust types corresponding to this argument at all levels.
    pub argument_type: RustFinalType,
    /// Rust argument name.
    pub name: String,
    /// Index of the corresponding argument of the FFI function.
    pub ffi_index: usize,
}

/// Type of a receiver in Qt connection system.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum RustQtReceiverType {
    Signal,
    Slot,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustFfiWrapperData {
    /// C++ method corresponding to this variant.
    pub cpp_ffi_function: CppFfiFunction,

    pub ffi_function_path: RustPath,
    /// Index of the FFI function argument used for acquiring the return value,
    /// if any. `None` if the return value is passed normally (as the return value
    /// of the FFI function).
    pub return_type_ffi_index: Option<usize>, // TODO: why needed here?
    pub ffi_item_index: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustSignalOrSlotGetter {
    /// C++ name of the signal or slot
    pub cpp_path: CppPath,
    /// Type of the receiver.
    pub receiver_type: RustQtReceiverType,
    /// Identifier of the signal or slot for passing to `QObject::connect`.
    pub receiver_id: String,

    pub cpp_doc: Option<CppFunctionExternalDoc>,
    pub cpp_item_index: usize,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum RustFunctionKind {
    FfiWrapper(RustFfiWrapperData),
    SignalOrSlotGetter(RustSignalOrSlotGetter),
}

impl RustFunctionKind {
    pub fn short_text(&self) -> String {
        match self {
            RustFunctionKind::FfiWrapper(data) => data.cpp_ffi_function.short_text(),
            RustFunctionKind::SignalOrSlotGetter(getter) => format!(
                "SignalOrSlotGetter({}",
                getter.cpp_path.to_cpp_pseudo_code()
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnnamedRustFunction {
    pub is_public: bool,
    pub is_unsafe: bool,
    pub kind: RustFunctionKind,
    pub arguments: Vec<RustFunctionArgument>,
    pub return_type: RustFinalType,
    pub extra_doc: Option<String>,
}

impl UnnamedRustFunction {
    pub fn with_path(self, path: RustPath) -> RustFunction {
        RustFunction {
            path,
            is_public: self.is_public,
            is_unsafe: self.is_unsafe,
            kind: self.kind,
            arguments: self.arguments,
            return_type: self.return_type,
            extra_doc: self.extra_doc,
        }
    }

    /// Returns information about `self` argument of this method.
    pub fn self_arg_kind(&self) -> Result<RustFunctionSelfArgKind> {
        if let Some(arg) = self.arguments.get(0) {
            if arg.name == "self" {
                match arg.argument_type.api_type() {
                    RustType::PointerLike { kind, is_const, .. } => match *kind {
                        RustPointerLikeTypeKind::Pointer => {
                            bail!("pointer self arg is not supported")
                        }
                        RustPointerLikeTypeKind::Reference { .. } => {
                            if *is_const {
                                return Ok(RustFunctionSelfArgKind::ConstRef);
                            } else {
                                return Ok(RustFunctionSelfArgKind::MutRef);
                            }
                        }
                    },
                    RustType::Common { .. } => {
                        return Ok(RustFunctionSelfArgKind::Value);
                    }
                    _ => {
                        bail!("invalid self argument type: {:?}", self);
                    }
                }
            }
        }
        Ok(RustFunctionSelfArgKind::None)
    }

    /*/// Generates name suffix for this function using `caption_strategy`.
    /// `all_self_args` should contain all kinds of arguments found in
    /// the functions that have to be disambiguated using the name suffix.
    /// `index` is number of the function used in `RustFunctionCaptionStrategy::Index`.
    #[allow(dead_code)]
    fn name_suffix(
        &self,
        context: &RustPath,
        caption_strategy: &RustFunctionCaptionStrategy,
        all_self_args: &HashSet<RustFunctionSelfArgKind>,
        index: usize,
    ) -> Result<Option<String>> {
        if caption_strategy == &RustFunctionCaptionStrategy::UnsafeOnly {
            return Ok(if self.is_unsafe {
                Some("unsafe".to_string())
            } else {
                None
            });
        }
        let result = {
            let self_arg_kind = self.self_arg_kind()?;
            let self_arg_kind_caption =
                if all_self_args.len() == 1 || self_arg_kind == RustFunctionSelfArgKind::ConstRef {
                    None
                } else if self_arg_kind == RustFunctionSelfArgKind::None {
                    Some("static")
                } else if self_arg_kind == RustFunctionSelfArgKind::MutRef {
                    if all_self_args.contains(&RustFunctionSelfArgKind::ConstRef) {
                        Some("mut")
                    } else {
                        None
                    }
                } else {
                    bail!("unsupported self arg kinds combination");
                };
            let other_caption = match *caption_strategy {
                RustFunctionCaptionStrategy::SelfOnly => None,
                RustFunctionCaptionStrategy::UnsafeOnly => unreachable!(),
                RustFunctionCaptionStrategy::SelfAndIndex => Some(index.to_string()),
                RustFunctionCaptionStrategy::SelfAndArgNames => {
                    if self.arguments.is_empty() {
                        Some("no_args".to_string())
                    } else {
                        Some(self.arguments.iter().map(|a| &a.name).join("_"))
                    }
                }
                RustFunctionCaptionStrategy::SelfAndArgTypes => {
                    if self.arguments.is_empty() {
                        Some("no_args".to_string())
                    } else {
                        Some(
                            self.arguments
                                .iter()
                                .filter(|t| &t.name != "self")
                                .map_if_ok(|t| t.argument_type.api_type().caption(context))?
                                .join("_"),
                        )
                    }
                }
            };
            let mut key_caption_items = Vec::new();
            if let Some(c) = self_arg_kind_caption {
                key_caption_items.push(c.to_string());
            }
            if let Some(c) = other_caption {
                key_caption_items.push(c);
            }
            if key_caption_items.is_empty() {
                None
            } else {
                Some(key_caption_items.join("_"))
            }
        };
        Ok(result)
    }*/
}

/// Information about a public API function.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustFunction {
    pub is_public: bool,

    /// True if the function is `unsafe`.
    pub is_unsafe: bool,
    /// Full name of the function.
    pub path: RustPath,

    pub kind: RustFunctionKind,

    /// List of arguments. For an overloaded function, only the arguments
    /// involved in the overloading are listed in this field.
    /// There can also be arguments shared by all variants (typically the
    /// `self` argument), and they are not listed in this field.
    pub arguments: Vec<RustFunctionArgument>,
    /// C++ and Rust return types at all levels.
    pub return_type: RustFinalType,

    /// Documentation data.
    pub extra_doc: Option<String>,
}

/// Information about type of `self` argument of the function.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum RustFunctionSelfArgKind {
    /// No `self` argument (static function or a free function).
    None,
    /// `&self` argument.
    ConstRef,
    /// `&mut self` argument.
    MutRef,
    /// `self` argument.
    Value,
}

/// Information about an associated type value
/// within a trait implementation.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustTraitAssociatedType {
    /// Name of the associated type.
    pub name: String,
    /// Value of the associated type.
    pub value: RustType,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum RustTraitImplSourceKind {
    Normal,
    Deref,
    DerefMut,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustTraitImplSource {
    pub ffi_item_index: usize,
    pub kind: RustTraitImplSourceKind,
}

/// Information about a trait implementation.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustTraitImpl {
    pub parent_path: RustPath,
    /// Type the trait is implemented for.
    pub target_type: RustType,
    /// Type of the trait.
    pub trait_type: RustType, // TODO: RustCommonType?
    /// Values of associated types of the trait.
    pub associated_types: Vec<RustTraitAssociatedType>,
    /// Functions that implement the trait.
    pub functions: Vec<RustFunction>,
    pub source: RustTraitImplSource,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustModuleDoc {
    pub extra_doc: Option<String>,
    pub cpp_path: Option<CppPath>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum RustSpecialModuleKind {
    CrateRoot,
    Ffi,
    Ops,
    SizedTypes,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum RustModuleKind {
    Special(RustSpecialModuleKind),
    CppNamespace { cpp_item_index: usize },
    CppNestedTypes { cpp_item_index: usize },
}

impl RustModuleKind {
    pub fn is_in_separate_file(self) -> bool {
        match self {
            RustModuleKind::Special(kind) => match kind {
                RustSpecialModuleKind::CrateRoot => true,
                RustSpecialModuleKind::Ffi => false,
                RustSpecialModuleKind::Ops => true,
                RustSpecialModuleKind::SizedTypes => false,
            },
            RustModuleKind::CppNamespace { .. } => true,
            RustModuleKind::CppNestedTypes { .. } => false,
        }
    }

    pub fn is_cpp_nested_types(self) -> bool {
        if let RustModuleKind::CppNestedTypes { .. } = self {
            true
        } else {
            false
        }
    }
}

/// Information about a Rust module.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RustModule {
    pub is_public: bool,

    /// Path to the module.
    pub path: RustPath,
    /// Markdown content of Rust documentation for this module.
    pub doc: RustModuleDoc,

    pub kind: RustModuleKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RustTypeCaptionStrategy {
    LastName,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RustFunctionCaptionStrategy {
    pub mut_: bool,
    pub args_count: bool,
    pub arg_names: bool,
    pub arg_types: Option<RustTypeCaptionStrategy>,
    pub static_: bool,
}

impl RustFunctionCaptionStrategy {
    /// Returns list of all available strategies sorted by priority
    /// (more preferred strategies go first).
    #[allow(dead_code)]
    pub fn all() -> Vec<Self> {
        use self::RustFunctionCaptionStrategy as S;

        let mut all = Vec::new();
        all.push(S {
            mut_: true,
            ..S::default()
        });

        let other = &[
            S {
                args_count: true,
                ..S::default()
            },
            S {
                static_: true,
                ..S::default()
            },
            S {
                arg_types: Some(RustTypeCaptionStrategy::LastName),
                ..S::default()
            },
            S {
                arg_types: Some(RustTypeCaptionStrategy::LastName),
                static_: true,
                ..S::default()
            },
            S {
                arg_types: Some(RustTypeCaptionStrategy::Full),
                ..S::default()
            },
            S {
                arg_types: Some(RustTypeCaptionStrategy::Full),
                static_: true,
                ..S::default()
            },
        ];

        for item in other {
            all.push(item.clone());
            all.push(S {
                mut_: true,
                ..item.clone()
            });
        }

        all
    }
}

/// Information about an argument of a Rust FFI function.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RustFFIArgument {
    /// Name of the argument.
    pub name: String,
    /// Type of the argument.
    pub argument_type: RustType,
}

/// Information about a Rust FFI function.
/// Name and signature of this function must be the same
/// as the corresponding C++ function on the other side of FFI.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RustFFIFunction {
    /// Return type of the function.
    pub return_type: RustType,
    /// Name of the function.
    pub path: RustPath,
    /// Arguments of the function.
    pub arguments: Vec<RustFFIArgument>,
    pub ffi_item_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustRawSlotReceiver {
    pub target_path: RustPath,
    pub arguments: RustType,
    pub receiver_id: String,
    pub cpp_item_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustFlagEnumImpl {
    pub enum_path: RustPath,
    pub cpp_item_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RustExtraImplKind {
    FlagEnum(RustFlagEnumImpl),
    RawSlotReceiver(RustRawSlotReceiver),
}

impl RustExtraImplKind {
    pub fn has_same_source(&self, other: &Self) -> bool {
        match self {
            RustExtraImplKind::FlagEnum(data) => {
                if let RustExtraImplKind::FlagEnum(other) = other {
                    data.cpp_item_index == other.cpp_item_index
                } else {
                    false
                }
            }
            RustExtraImplKind::RawSlotReceiver(data) => {
                if let RustExtraImplKind::RawSlotReceiver(other) = other {
                    data.cpp_item_index == other.cpp_item_index
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustExtraImpl {
    pub parent_path: RustPath,
    pub kind: RustExtraImplKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RustReexportSource {
    DependencyCrate { crate_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustReexport {
    pub path: RustPath,
    pub target: RustPath,
    pub source: RustReexportSource,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RustItem {
    Module(RustModule),
    Struct(RustStruct),
    EnumValue(RustEnumValue),
    TraitImpl(RustTraitImpl),
    ExtraImpl(RustExtraImpl),
    FfiFunction(RustFFIFunction), // TODO: merge FfiFunction and Function
    Function(RustFunction),
    Reexport(RustReexport),
}

impl RustItem {
    pub fn has_same_source(&self, other: &Self) -> bool {
        match self {
            RustItem::Module(data) => {
                if let RustItem::Module(other) = other {
                    data.kind == other.kind
                } else {
                    false
                }
            }
            RustItem::Struct(data) => {
                if let RustItem::Struct(other) = other {
                    data.kind.has_same_source(&other.kind)
                } else {
                    false
                }
            }
            RustItem::EnumValue(data) => {
                if let RustItem::EnumValue(other) = other {
                    data.cpp_item_index == other.cpp_item_index
                } else {
                    false
                }
            }
            RustItem::TraitImpl(data) => match other {
                RustItem::TraitImpl(other) => data.source == other.source,
                RustItem::Function(other) => {
                    if let RustFunctionKind::FfiWrapper(other) = &other.kind {
                        data.source.ffi_item_index == other.ffi_item_index
                            && data.source.kind == RustTraitImplSourceKind::Normal
                    } else {
                        false
                    }
                }
                _ => false,
            },
            RustItem::ExtraImpl(data) => {
                if let RustItem::ExtraImpl(other) = other {
                    data.kind.has_same_source(&other.kind)
                } else {
                    false
                }
            }
            RustItem::FfiFunction(data) => {
                if let RustItem::FfiFunction(other) = other {
                    data.ffi_item_index == other.ffi_item_index
                } else {
                    false
                }
            }
            RustItem::Function(data) => match &data.kind {
                RustFunctionKind::FfiWrapper(data) => match other {
                    RustItem::TraitImpl(other) => {
                        data.ffi_item_index == other.source.ffi_item_index
                            && other.source.kind == RustTraitImplSourceKind::Normal
                    }
                    RustItem::Function(other) => {
                        if let RustFunctionKind::FfiWrapper(other) = &other.kind {
                            data.ffi_item_index == other.ffi_item_index
                        } else {
                            false
                        }
                    }
                    _ => false,
                },
                RustFunctionKind::SignalOrSlotGetter(data) => {
                    if let RustItem::Function(other) = other {
                        if let RustFunctionKind::SignalOrSlotGetter(other) = &other.kind {
                            data.cpp_item_index == other.cpp_item_index
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            },
            RustItem::Reexport(data) => {
                if let RustItem::Reexport(other) = other {
                    data.source == other.source
                } else {
                    false
                }
            }
        }
    }

    pub fn is_ffi_function(&self) -> bool {
        if let RustItem::FfiFunction(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_wrapper_type(&self) -> bool {
        if let RustItem::Struct(data) = self {
            data.kind.is_wrapper_type()
        } else {
            false
        }
    }

    pub fn is_module(&self) -> bool {
        if let RustItem::Module(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_module_for_nested(&self) -> bool {
        if let RustItem::Module(module) = self {
            module.kind.is_cpp_nested_types()
        } else {
            false
        }
    }

    pub fn is_crate_root(&self) -> bool {
        if let RustItem::Module(module) = self {
            module.kind == RustModuleKind::Special(RustSpecialModuleKind::CrateRoot)
        } else {
            false
        }
    }

    pub fn short_text(&self) -> String {
        match self {
            RustItem::Module(data) => format!("mod {}", data.path.full_name(None)),
            RustItem::Struct(data) => format!("struct {}", data.path.full_name(None)),
            RustItem::EnumValue(data) => format!("enum value {}", data.path.full_name(None)),
            RustItem::TraitImpl(data) => format!(
                "impl {} for {}",
                rust_type_to_code(&data.trait_type, None),
                rust_type_to_code(&data.target_type, None)
            ),
            RustItem::ExtraImpl(data) => format!("extra impl {:?}", data.kind),
            RustItem::FfiFunction(data) => format!("ffi fn {}", data.path.full_name(None)),
            RustItem::Function(data) => format!("fn {}", data.path.full_name(None)),
            RustItem::Reexport(data) => format!(
                "use {} as {}",
                data.path.full_name(None),
                data.target.last()
            ),
        }
    }

    pub fn as_reexport_ref(&self) -> Option<&RustReexport> {
        if let RustItem::Reexport(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_module_ref(&self) -> Option<&RustModule> {
        if let RustItem::Module(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustDatabaseItem {
    pub item: RustItem,

    // TODO: remove this
    pub cpp_item_index: Option<usize>,
    pub ffi_item_index: Option<usize>,
}

impl RustDatabaseItem {
    pub fn path(&self) -> Option<&RustPath> {
        match &self.item {
            RustItem::Module(data) => Some(&data.path),
            RustItem::Struct(data) => Some(&data.path),
            RustItem::EnumValue(data) => Some(&data.path),
            RustItem::Function(data) => Some(&data.path),
            RustItem::FfiFunction(data) => Some(&data.path),
            RustItem::Reexport(data) => Some(&data.path),
            RustItem::TraitImpl(_) | RustItem::ExtraImpl(_) => None,
        }
    }

    pub fn parent_path(&self) -> Result<RustPath> {
        match &self.item {
            RustItem::TraitImpl(trait_impl) => Ok(trait_impl.parent_path.clone()),
            RustItem::ExtraImpl(data) => Ok(data.parent_path.clone()),
            _ => self
                .path()
                .expect("item must have path because it's not a trait impl")
                .parent(),
        }
    }

    pub fn is_child_of(&self, parent: &RustPath) -> bool {
        self.parent_path().ok().as_ref() == Some(parent)
    }

    pub fn as_module_ref(&self) -> Option<&RustModule> {
        if let RustItem::Module(data) = &self.item {
            Some(data)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustDatabase {
    crate_name: String,
    items: Vec<RustDatabaseItem>,
}

impl RustDatabase {
    pub fn new(crate_name: String) -> Self {
        Self {
            crate_name,
            items: Vec::new(),
        }
    }

    pub fn find(&self, path: &RustPath) -> Option<&RustDatabaseItem> {
        self.items.iter().find(|item| item.path() == Some(path))
    }

    pub fn children<'a>(
        &'a self,
        path: &'a RustPath,
    ) -> impl Iterator<Item = &'a RustDatabaseItem> {
        self.items.iter().filter(move |item| item.is_child_of(path))
    }

    pub fn items(&self) -> &[RustDatabaseItem] {
        &self.items
    }

    pub fn add_item(&mut self, item: RustDatabaseItem) -> Result<()> {
        if item.item.is_crate_root() {
            let item_path = item.path().expect("crate root must have path");
            let crate_name = item_path
                .crate_name()
                .expect("rust item path must have crate name");
            if crate_name != self.crate_name {
                bail!("can't add rust item with different crate name: {:?}", item);
            }
        } else {
            let mut path = item
                .parent_path()
                .map_err(|_| format_err!("path has no parent for rust item: {:?}", item))?;
            let crate_name = path
                .crate_name()
                .expect("rust item path must have crate name");
            if crate_name != self.crate_name {
                bail!("can't add rust item with different crate name: {:?}", item);
            }
            while path.parts.len() > 1 {
                if self.find(&path).is_none() {
                    bail!("unreachable path {:?} for rust item: {:?}", path, item);
                }
                path.parts.pop();
            }
        }

        self.items.push(item);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn make_unique_path(&self, path: &RustPath) -> RustPath {
        let mut number = None;
        let mut path_try = path.clone();
        loop {
            if let Some(number) = number {
                *path_try.last_mut() = format!(
                    "{}{}{}",
                    path.last(),
                    if ends_with_digit(path.last()) {
                        "_"
                    } else {
                        ""
                    },
                    number
                );
            }
            if self.find(&path_try).is_none() {
                return path_try;
            }

            number = Some(number.unwrap_or(1) + 1);
        }
        // TODO: check for conflicts with types from crate template (how?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustPathScope {
    pub path: RustPath,
    pub prefix: Option<String>,
}

impl RustPathScope {
    pub fn apply(&self, name: &str) -> RustPath {
        let full_name = if let Some(prefix) = &self.prefix {
            format!("{}{}", prefix, name)
        } else {
            name.to_string()
        };
        self.path.join(full_name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameType<'a> {
    Type,
    EnumValue,
    Module,
    FfiFunction,
    ApiFunction(&'a CppFfiFunction),
    ReceiverFunction {
        receiver_type: RustQtReceiverType,
    },
    SizedItem,
    QtSlotWrapper {
        signal_arguments: &'a [CppType],
        is_public: bool,
    },
}

impl NameType<'_> {
    pub fn is_api_function(&self) -> bool {
        match self {
            NameType::ApiFunction(_) => true,
            _ => false,
        }
    }
}
