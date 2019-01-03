//use crate::database::Database;
use crate::processor::ProcessingStep;
use crate::processor::ProcessorData;
//use cpp_to_rust_common::log;
use crate::common::errors::{bail, Result};
use crate::cpp_type::CppClassType;
use crate::cpp_type::CppType;
use crate::database::CppItemData;
use crate::database::DatabaseItem;
use cpp_to_rust_common::log;

fn check_type(all_items: &[&DatabaseItem], cpp_type: &CppType) -> Result<()> {
    match cpp_type {
        CppType::Class(CppClassType {
            ref name,
            ref template_arguments,
        }) => {
            if !all_items
                .iter()
                .filter_map(|item| item.cpp_data.as_type_ref())
                .any(|t| &t.name == name && t.kind.is_class())
            {
                bail!("class not found: {}", name);
            }

            if let Some(ref args) = *template_arguments {
                for arg in args {
                    check_type(all_items, arg)?;
                }
            }
        }
        CppType::Enum { name } => {
            if !all_items
                .iter()
                .filter_map(|item| item.cpp_data.as_type_ref())
                .any(|t| &t.name == name && t.kind.is_enum())
            {
                bail!("enum not found: {}", name);
            }
        }
        CppType::PointerLike { ref target, .. } => {
            check_type(all_items, target)?;
        }
        CppType::FunctionPointer(t) => {
            check_type(all_items, &t.return_type)?;

            for arg in &t.arguments {
                check_type(all_items, arg)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn is_cpp_item_resolvable(all_items: &[&DatabaseItem], item: &CppItemData) -> Result<()> {
    for cpp_type in &item.all_involved_types() {
        check_type(&all_items, cpp_type)?;
    }
    Ok(())
}

/// Runs the parser on specified data.
fn run(data: &mut ProcessorData) -> Result<()> {
    let all_items = data.all_items();
    for item in &data.current_database.items {
        if item.rust_item.is_some() {
            continue;
        }
        match is_cpp_item_resolvable(&all_items, &item.cpp_data) {
            Ok(_) => unimplemented!(),
            Err(err) => {
                log::error(format!("skipping item: {}: {}", &item.cpp_data, err));
            }
        }
    }
    // TODO: everything
    Ok(())
}

pub fn rust_name_resolver_step() -> ProcessingStep {
    // TODO: set dependencies
    ProcessingStep::new("rust_name_resolver", Vec::new(), run)
}

#[test]
fn it_should_check_functions() {
    use crate::cpp_data::CppName;
    use crate::cpp_data::CppTypeData;
    use crate::cpp_data::CppTypeDataKind;
    use crate::cpp_function::CppFunction;
    use crate::cpp_function::CppFunctionArgument;
    use crate::database::DatabaseItemSource;

    let func = CppFunction {
        name: CppName::from_one_part("foo"),
        member: None,
        operator: None,
        return_type: CppType::Void,
        arguments: vec![],
        allows_variadic_arguments: false,
        template_arguments: None,
        declaration_code: None,
        doc: None,
    };
    let func_item = DatabaseItem {
        cpp_data: CppItemData::Function(func.clone()),
        source: DatabaseItemSource::ImplicitDestructor,
        ffi_items: None,
        rust_item: None,
    };

    let func2_item = DatabaseItem {
        cpp_data: CppItemData::Function(CppFunction {
            arguments: vec![CppFunctionArgument {
                name: "a".to_string(),
                argument_type: CppType::Class(CppClassType {
                    name: CppName::from_one_part("C1"),
                    template_arguments: None,
                }),
                has_default_value: false,
            }],
            ..func
        }),
        source: DatabaseItemSource::ImplicitDestructor,
        ffi_items: None,
        rust_item: None,
    };
    let all_items = &[&func_item, &func2_item];
    assert!(is_cpp_item_resolvable(all_items, &func_item.cpp_data).is_ok());
    assert!(is_cpp_item_resolvable(all_items, &func2_item.cpp_data).is_err());

    let class_item = DatabaseItem {
        cpp_data: CppItemData::Type(CppTypeData {
            name: CppName::from_one_part("C1"),
            kind: CppTypeDataKind::Class {
                type_base: CppClassType {
                    name: CppName::from_one_part("C1"),
                    template_arguments: None,
                },
            },
            doc: None,
            is_movable: false,
        }),
        source: DatabaseItemSource::ImplicitDestructor,
        ffi_items: None,
        rust_item: None,
    };
    let all_items = &[&func_item, &func2_item, &class_item];
    assert!(is_cpp_item_resolvable(all_items, &func_item.cpp_data).is_ok());
    assert!(is_cpp_item_resolvable(all_items, &func2_item.cpp_data).is_ok());
}