#[cfg(feature = "riscv-cva6")]
pub use cva6::*;
#[cfg(feature = "riscv-cva6")]
mod cva6 {
    use crate::{
        analyze::Analysis as CodegenAnalysis,
        codegen::util,
        syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
    };

    use proc_macro2::{Span, TokenStream as TokenStream2};
    use quote::quote;
    use std::collections::HashSet;
    use syn::{parse, Attribute, Ident};

    pub fn impl_mutex(
        _app: &App,
        _analysis: &CodegenAnalysis,
        _cfgs: &[Attribute],
        _resources_prefix: bool,
        _name: &Ident,
        _ty: &TokenStream2,
        _ceiling: u8,
        _ptr: &TokenStream2,
    ) -> TokenStream2 {
        quote!()
    }

    pub fn extra_assertions(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
        vec![]
    }

    pub fn pre_init_checks(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
        vec![]
    }

    pub fn pre_init_enable_interrupts(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
        let mut stmts = vec![];
        let mut curr_cpu_id: u8 = 1; //cpu interrupt id 0 is reserved
        let rt_err = util::rt_err_ident();
        let max_prio: usize = 15; //check this for cva6
        let interrupt_ids = analysis.interrupts.iter().map(|(p, (id, _))| (p, id));

        // Unmask interrupts and set their priorities
        for (&priority, name) in interrupt_ids.chain(
            app.hardware_tasks
                .values()
                .filter_map(|task| Some((&task.args.priority, &task.args.binds))),
        ) {
            let es = format!(
                "Maximum priority used by interrupt vector '{name}' is more than supported by hardware"
            );
            // Compile time assert that this priority is supported by the device
            stmts.push(quote!(
                const _: () =  if (#max_prio) <= #priority as usize { ::core::panic!(#es); };
            ));
            stmts.push(quote!(
                rtic::export::enable(
                    #rt_err::Interrupt::#name,
                    #priority,
                    #curr_cpu_id,
                );
            ));
            curr_cpu_id += 1;
        }
        stmts
    }

    pub fn architecture_specific_analysis(
        app: &App,
        _analysis: &SyntaxAnalysis,
    ) -> parse::Result<()> {
        //check if the dispatchers are supported
        for name in app.args.dispatchers.keys() {
            let name_s = name.to_string();
            match &*name_s {
                "Soft1" | "Soft2" | "Soft3" => {}

                _ => {
                    return Err(parse::Error::new(
                        name.span(),
                        "Only SoftX are supported as dispatchers",
                    ));
                }
            }
        }

        // Check that there are enough external interrupts to dispatch the software tasks and the timer
        // queue handler
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

            // If not enough tasks and first still is None, may cause
            // "custom attribute panicked" due to unwrap on None
            return Err(parse::Error::new(first.unwrap().span(), s));
        }
        Ok(())
    }

    pub fn interrupt_entry(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
        // TODO: set PLICs thershold if needed
        // PLIC::ctx0().threshold().get_register().write();
        // TODO: store old theshold

        //enable global interrupts
        // unsafe {
        //     riscv::interrupt::enable();
        // }
        vec![]
    }

    pub fn interrupt_exit(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
        vec![]
    }

    pub fn interrupt_ident() -> Ident {
        let span = Span::call_site();
        Ident::new("Interrupt", span)
    }

    pub fn async_entry(
        _app: &App,
        _analysis: &CodegenAnalysis,
        dispatcher_name: Ident,
    ) -> Vec<TokenStream2> {
        let mut stmts = vec![];
        stmts.push(quote!(
            rtic::export::unpend(rtic::export::Interrupt::#dispatcher_name); //simulate cortex-m behavior by unpending the interrupt on entry.
        ));
        stmts
    }

    pub fn async_prio_limit(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
        let max = if let Some(max) = analysis.max_async_prio {
            quote!(#max)
        } else {
            // No limit
            let device = &app.args.device;
            // althought this is related to Cortex-M NVIC, it is available
            // in the PAC as it is required by the CMSISS-SVD format and it is
            // in fact match the number of bit used for priority representation.
            quote!(1 << #device::NVIC_PRIO_BITS)
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
}
