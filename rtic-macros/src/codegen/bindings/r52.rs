use crate::{
    analyze::Analysis as CodegenAnalysis,
    syntax::{
        analyze::Analysis as SyntaxAnalysis,
        ast::App,
    },
};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::collections::HashSet;
use syn::{parse, Attribute, Ident};

pub fn interrupt_ident() -> Ident {
    let span = Span::call_site();
    Ident::new("Interrupt", span)
}

pub fn interrupt_mod(app: &App) -> TokenStream2 {
    let device = &app.args.device;
    let interrupt = interrupt_ident();
    quote!(#device::#interrupt)
}

/// Minimal `Mutex` implementation for R52 using a global critical section.
#[allow(clippy::too_many_arguments)]
pub fn impl_mutex(
    _app: &App,
    _analysis: &CodegenAnalysis,
    cfgs: &[Attribute],
    resources_prefix: bool,
    name: &Ident,
    ty: &TokenStream2,
    ceiling: u8,
    ptr: &TokenStream2,
) -> TokenStream2 {
    let path = if resources_prefix {
        quote!(shared_resources::#name)
    } else {
        quote!(#name)
    };

    quote!(
        #(#cfgs)*
        impl<'a> rtic::Mutex for #path<'a> {
            type T = #ty;

            #[inline(always)]
            fn lock<RTIC_INTERNAL_R>(&mut self, f: impl FnOnce(&mut #ty) -> RTIC_INTERNAL_R) -> RTIC_INTERNAL_R {
                const CEILING: u8 = #ceiling;
                unsafe { rtic::export::lock(#ptr, CEILING, f) }
            }
        }
    )
}

pub fn extra_assertions(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn pre_init_preprocessing(app: &mut App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
    // R52 backend does not provide cortex-m core peripherals.
    app.args.core = false;
    Ok(())
}

pub fn pre_init_checks(app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    let mut stmts: Vec<TokenStream2> = vec![];
    let int_mod = interrupt_mod(app);

    for name in app.args.dispatchers.keys() {
        stmts.push(quote!(let _ = #int_mod::#name;));
    }

    for task in app.hardware_tasks.values() {
        let name = &task.args.binds;
        stmts.push(quote!(let _ = #int_mod::#name;));
    }

    stmts
}

pub fn pre_init_enable_interrupts(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn architecture_specific_analysis(app: &App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
    let mut first = None;
    let priorities = app
        .software_tasks
        .iter()
        .map(|(name, task)| {
            first = Some(name);
            task.args.priority
        })
        .filter(|prio| *prio > 0)
        .collect::<HashSet<_>>();

    let need = priorities.len();
    let given = app.args.dispatchers.len();
    if need > given {
        let s = {
            format!(
                "not enough interrupts to dispatch \
                    all software tasks (need: {need}; given: {given})"
            )
        };

        return Err(parse::Error::new(first.unwrap().span(), s));
    }

    Ok(())
}

pub fn interrupt_entry(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn interrupt_exit(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn check_stack_overflow_before_init(
    _app: &App,
    _analysis: &CodegenAnalysis,
) -> Vec<TokenStream2> {
    vec![]
}

pub fn async_entry(
    _app: &App,
    _analysis: &CodegenAnalysis,
    _dispatcher_name: Ident,
) -> Vec<TokenStream2> {
    vec![]
}

pub fn async_prio_limit(_app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    let max = if let Some(max) = analysis.max_async_prio {
        quote!(#max)
    } else {
        quote!(u8::MAX)
    };

    vec![quote!(
        /// Holds the maximum priority level for use by async HAL drivers.
        #[no_mangle]
        static RTIC_ASYNC_MAX_LOGICAL_PRIO: u8 = #max;
    )]
}

pub fn handler_config(
    _app: &App,
    _analysis: &CodegenAnalysis,
    _dispatcher_name: Ident,
) -> Vec<TokenStream2> {
    vec![]
}

pub fn extra_modules(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}
