use crate::plugin::*;
use crate::{def_package, FnPtr, ImmutableString, NativeCallContext};

def_package!(crate:BasicFnPackage:"Basic Fn functions.", lib, {
    combine_with_exported_module!(lib, "FnPtr", fn_ptr_functions);
});

#[export_module]
mod fn_ptr_functions {
    #[rhai_fn(name = "name", get = "name", pure)]
    pub fn name(f: &mut FnPtr) -> ImmutableString {
        f.get_fn_name().clone()
    }

    #[cfg(not(feature = "no_function"))]
    pub mod functions {
        #[rhai_fn(name = "is_anonymous", get = "is_anonymous", pure)]
        pub fn is_anonymous(f: &mut FnPtr) -> bool {
            f.is_anonymous()
        }
    }

    #[cfg(not(feature = "no_function"))]
    #[cfg(not(feature = "no_index"))]
    #[cfg(not(feature = "no_object"))]
    pub mod functions_and_maps {
        pub fn get_fn_metadata_list(ctx: NativeCallContext) -> crate::Array {
            collect_fn_metadata(ctx)
        }
    }
}

#[cfg(not(feature = "no_function"))]
#[cfg(not(feature = "no_index"))]
#[cfg(not(feature = "no_object"))]
fn collect_fn_metadata(ctx: NativeCallContext) -> crate::Array {
    use crate::{ast::ScriptFnDef, stdlib::collections::HashMap, Array, Map};

    // Create a metadata record for a function.
    fn make_metadata(
        dict: &HashMap<&str, ImmutableString>,
        namespace: Option<ImmutableString>,
        f: &ScriptFnDef,
    ) -> Map {
        let mut map = Map::with_capacity(6);

        if let Some(ns) = namespace {
            map.insert(dict["namespace"].clone(), ns.into());
        }
        map.insert(dict["name"].clone(), f.name.clone().into());
        map.insert(
            dict["access"].clone(),
            match f.access {
                FnAccess::Public => dict["public"].clone(),
                FnAccess::Private => dict["private"].clone(),
            }
            .into(),
        );
        map.insert(
            dict["is_anonymous"].clone(),
            f.name.starts_with(crate::engine::FN_ANONYMOUS).into(),
        );
        map.insert(
            dict["params"].clone(),
            f.params
                .iter()
                .cloned()
                .map(Into::<Dynamic>::into)
                .collect::<Array>()
                .into(),
        );

        map.into()
    }

    // Intern strings
    let mut dict = HashMap::<&str, ImmutableString>::with_capacity(8);
    [
        "namespace",
        "name",
        "access",
        "public",
        "private",
        "is_anonymous",
        "params",
    ]
    .iter()
    .for_each(|&s| {
        dict.insert(s, s.into());
    });

    let mut list: Array = Default::default();

    ctx.iter_namespaces()
        .flat_map(|m| m.iter_script_fn())
        .for_each(|(_, _, _, _, f)| list.push(make_metadata(&dict, None, f).into()));

    #[cfg(not(feature = "no_module"))]
    {
        // Recursively scan modules for script-defined functions.
        fn scan_module(
            list: &mut Array,
            dict: &HashMap<&str, ImmutableString>,
            namespace: ImmutableString,
            module: &Module,
        ) {
            module.iter_script_fn().for_each(|(_, _, _, _, f)| {
                list.push(make_metadata(dict, Some(namespace.clone()), f).into())
            });
            module.iter_sub_modules().for_each(|(ns, m)| {
                let ns: ImmutableString = format!("{}::{}", namespace, ns).into();
                scan_module(list, dict, ns, m.as_ref())
            });
        }

        ctx.iter_imports_raw()
            .for_each(|(ns, m)| scan_module(&mut list, &dict, ns.clone(), m.as_ref()));
    }

    list
}
